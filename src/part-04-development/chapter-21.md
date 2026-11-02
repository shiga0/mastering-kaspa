# Chapter 21 — Pruning, Storage & Node Synchronization（Pruning・ストレージ・ノード同期）

> **Part IV — Kaspa Development & Implementation**  
> 目次順の本文レビュー：2026-11-01。主要経路は基準版と照合。本文の確認と実Pruning・Pruning後同期の検証は区別する。\
> 対象：Kaspaの仕組みを理解し、`rusty-kaspa` のコード・運用・テストに進む開発者

## 照合した実装と実測範囲

2026年10月31日、[本書の基準版](../baseline.md) v1.0.1、コミット `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` のPruningパラメータ、point進行、削減、証明検証、UTXOインポートの主要経路をソースで確認した（Source-checked）。今回の目次順レビューで本文と教材を確認したが、実データ削減・Pruning後の新規ノード同期は未検証。

既存実験のDevnetは5475ブロック、sinkのblue scoreも5475で、Pruning pointはGenesisのままだった。短い実験の再起動・分岐復旧をPruning後の検証へ読み替えない。記録は `experiments/devnet-pruning-review-2026-10-31.md`。

## この章で学ぶこと

この章のテーマは、**「全履歴を保持しなくても、新しいノードはどうやって現在の状態を検証できるのか」**である。KaspaはBlockDAGを採用し、多数のブロックを扱う。そのため、過去のデータを無制限に保存する設計では、ディスク・ネットワーク・同期時間の負担が増え続ける。

学習後には、次の点を説明できるようになる。

1. Pruningと単なるデータ削除の違い
2. BlockDAGの履歴と現在のUTXO集合の役割
3. Pruning Point、Pruning Proof、UTXO commitmentの関係
4. IBD（Initial Block Download）の大まかな処理と信頼境界
5. ストレージ層・同期層・コンセンサス層の責務分担
6. Rustで検証可能な最小スナップショット・インポート処理
7. 実ノードの障害調査と実験の進め方

---

## 21.1 なぜPruningが必要なのか

分散台帳では、過去のデータが時間とともに増加する。特に高いブロック生成レートを持つネットワークでは、ブロックヘッダー・トランザクション本体・DAGの関係情報・インデックスなどの保存コストが重要になる。

ただし、**過去の全トランザクションデータを保持すること**と、**現在のトランザクションの正当性を検証できること**は同義ではない。

UTXO方式では、すでに消費された出力は現在の未使用出力の集合に残らない。現在の状態を表すために必要なのは、基本的には「まだ消費されていない出力の集合」である。

```text
過去のトランザクション履歴
    │
    ├── すでに消費された出力
    ├── 現在も未使用の出力 ───────┐
    └── 監査・解析のための履歴    │
                                 ▼
                         現在のUTXO集合
```

Pruningは、コンセンサスで安全に不要と判断できる古い情報を削減し、検証に必要な状態と証拠を維持するための仕組みである。**「過去を何も検証せずに捨てること」ではない。**

## 21.2 Full Node、Pruned Node、Archiveの違い

| 観点 | Pruningを行う検証ノード | 履歴保持を重視するノード |
|---|---|---|
| 新規トランザクション・ブロックの検証 | 行う | 行う |
| 現在のUTXO状態 | 保持・検証 | 保持・検証 |
| 古いブロック本体 | 規則に従い削減可能 | より長く保持 |
| 過去の詳細な照会 | 制限がある | 保持範囲に応じて可能 |
| 主な用途 | 通常のネットワーク参加・検証 | インデックス作成・研究・履歴分析 |

**注意：**「フルノード」は検証能力を表す文脈でも使われる。全履歴を永久に保存することだけがフルノードの条件ではない。実際の保存範囲や起動オプションはバージョン・設定に依存する。

## 21.3 何を保存するのか：ストレージの論理モデル

Kaspaノードの永続化対象を概念的に分けると、次のようになる。

| データ | 用途 | 保存・削減の考え方 |
|---|---|---|
| ブロックヘッダー | PoW、親参照、コンセンサス情報 | 検証とPruning Proofに必要な範囲を保持 |
| ブロック本体 | トランザクション・Coinbaseなど | 古い本体はPruning対象になり得る |
| DAG関係情報 | 親子関係・到達可能性・順序付け | 現在の計算と証明に必要な情報を維持 |
| UTXO集合 | 現在の未使用出力（成熟等の使用条件は別に検査） | 正当性検証に必須 |
| コンセンサスメタデータ | blue work、DAAなど | プロトコルの規則に従い保持 |
| Mempool | 未確定のトランザクション候補 | 通常は永続コンセンサス履歴とは別 |
| アプリケーション用インデックス | アドレス検索・履歴照会など | 用途に応じて追加の保存コストが必要 |

ここで重要なのは、**コンセンサス上必要なデータ**と**アプリケーションが便利に検索するためのデータ**を混同しないことだ。

例えば「あるアドレスの過去の全トランザクションを即座に検索したい」という要件は、現在のUTXO集合を検証するだけでは満たせない。別途インデクサーや履歴保存戦略が必要になる。

## 21.4 Pruning Pointとは

Pruning Pointは、ノードが古い履歴を削減する際の基準となる、コンセンサス上の特別な地点である。

```text
古い履歴                              新しい履歴
─────────────────────┬────────────────────────▶
                     │
                Pruning Point
                     │
        過去の不要データを削減
                     │
        検証可能な状態・証拠を維持
```

図は**線形の時間軸に投影した概念図**であり、Kaspaの実データはBlockDAGである。Pruning Pointを「単純にこの高さ以前をすべて削除する境界」と理解してはいけない。証明に必要なヘッダーやコンセンサス情報の保持規則が存在する。

Pruning PointとFinalityの関係も重要だが、両者は同じ概念ではない。Finalityは過去のコンセンサス判断の変更を制限する規則であり、Pruningは保存データを削減するための仕組みである。

### 実装パラメータを読む際の注意

`rusty-kaspa` では、`NEW_PRUNING_DURATION` などの時間ベースの定数が定義されている。ただし、**ソース中の定数をそのまま「全ネットワークで必ず何時間後にすべて削除」と解釈しない**。ネットワーク別設定・バージョン・活性化済みアップグレード・実際の削除条件を合わせて確認する。

参考：[`consensus/core/src/config/constants.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)、[`params.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)。

### 21.4.1 基準版Devnetの進行条件

[DEVNET_PARAMS](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)はCrescendoを `never` とし、従来のpruning depthを185798、finality depthを86400、目標ブロック時間を1000msとする。Mainnetや後発コミットの値と区別する。

[next_pruning_points_v1_inner](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning.rs)は、現在pointと候補のblue score、sinkのblue score、pruning depth、finality scoreの区間を使って次のpointを選ぶ。DAA scoreや単純なブロック数を深さ判定へそのまま代入しない。185798という数に到達しただけで必ずpointが移動するわけでもない。

目標1 BPSを使った深さの単純換算は約51.6時間分だが、これを実験の所要時間や全データ削除時刻の保証とは扱わない。並行ブロックや難易度、point選択の条件がある。

[PruningProcessor](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/pruning_processor/processor.rs)ではpoint更新、Pruning用UTXO集合の更新、データのpruneを別段階で行う。`is_archival` の場合はデータpruneを実行しない。したがって、RPCでpointが動いたことだけでは、古い本体が削減されたことや、Pruning Proofで新規ノードが同期したことの証拠にはならない。

## 21.5 Pruning Proofが必要な理由

新しいノードが同期するとき、ネットワーク上の既存ノードは古い履歴をすでに削減しているかもしれない。新規ノードは、単に相手から「現在のUTXOはこれだ」と渡されただけでは、それを信用できない。同じ相手がUTXO集合とそのハッシュを提示して一致しても、比較先のヘッダー・履歴の正当性を確立したことにはならない。

そこで重要になるのが、**PoWで裏付けられた履歴の証拠と、現在状態のコミットメントを組み合わせて検証すること**である。

```text
未信頼のピア
  ├── Pruning Pointに関する証拠
  ├── 必要なヘッダー・DAG情報
  └── Pruning Point時点のUTXO集合
                │
                ▼
          ローカル検証
  ├── ヘッダー・PoW・コンセンサス条件
  ├── 証明の整合性
  └── UTXO集合とコミットメントの一致
                │
                ▼
         検証済みの同期基盤
```

**Pruning Proofは単なるチェックサムではない。** 履歴の妥当性を評価するための証拠であり、UTXO commitmentとは役割が異なる。両者を組み合わせることが重要である。

証明の厳密なアルゴリズムや信頼前提は、実装と論文を確認する必要がある。本章の図はプロトコルの逐語的な実装手順ではない。

## 21.6 UTXO CommitmentとMuHash

対象ヘッダーのUTXO commitmentは、そのブロックの過去の受理文脈に対応する。対象ブロック自身の通常取引をすべて適用した後の集合、と単純に解釈しない。

Pruning Pointに対応するUTXO集合をダウンロードする際、ノードはその内容がヘッダーにコミットされた状態と一致するか検証する。

概念的には次の関係になる。

```text
受信したUTXOエントリ群
       │
       ▼
  集合コミットメント計算
       │
       ▼
計算結果 == ヘッダーのコミットメント ?
       │                       │
      Yes                      No
       │                       │
       ▼                       ▼
次の検証段階へ               インポート拒否
```

Kaspaの実装では、UTXO集合のコミットメントに**MuHash**が用いられる。これは、集合の要素を順不同で扱うための暗号学的な集合ハッシュ構成である。通常の `SHA256(UTXOを文字列連結)` とは異なる。

基準版の[import_pruning_point_utxo_set](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/pipeline/virtual_processor/processor.rs)は、インポートしたMuHashのfinalize結果を対象ヘッダーの `utxo_commitment` と比較し、不一致なら `ImportedMultisetHashMismatch` を返す。これは固定ソースの読み取りによる確認であり、不正な集合を実際に注入した実行試験ではない。

> **セキュリティ上の注意**：一致するコミットメントがあるからといって、ネットワーク全体の正しいチェーン／DAG選択まで自動的に保証されるわけではない。どのヘッダーを信頼できるものとして採用するかについて、PoWとコンセンサス検証が別途必要である。

証明の構造・各レベルのPoWや仕事量等を調べる経路は [validate_pruning_point_proof](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning_proof/validate.rs)、証明をローカル状態へ取り込む経路は [apply_proof](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/src/processes/pruning_proof/apply.rs) にある。証明の検証・適用と、別途受信するUTXO集合のコミットメント検査を同じ検査として説明しない。

## 21.7 IBD（Initial Block Download）とは

IBDは、新しいノードがネットワークへ参加し、検証可能な状態へ到達するための初期同期である。

教育用に整理すると、次の段階に分けられる。

1. ピアを発見し、プロトコル互換性を確認する。
2. 相手の同期状況と自分のローカル状態を比較する。
3. 必要なPruning Proof・ヘッダーなどを受信して検証する。
4. Pruning Pointに対応するUTXO集合を受信し、コミットメントを検証する。
5. Pruning Point以降のブロック・状態変化を追いつくまで処理する。
6. 通常のブロック伝播・検証モードへ移行する。

**実際の順序・フェーズ・例外処理はバージョンによって異なる。** 上記は理解のための論理的な分解であり、実装の固定的なステートマシンを意味しない。

基準版の [`ibd/flow.rs`](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/protocol/flows/src/v5/ibd/flow.rs) は、既知の共有履歴からヘッダーを同期する経路と、headers proofを使う経路を分ける。すべてのIBDが毎回Pruning ProofとUTXO集合を取り直すわけではない。`v5/` は配置名であり、接続で合意した番号とは区別する。

### IBDと通常同期の違い

| 項目 | IBD | 通常同期 |
|---|---|---|
| 主目的 | 大きな履歴差を埋める | 新しいブロックへ追従する |
| 主なデータ | 証明、ヘッダー、UTXO、追いつき用ブロック | 新着ブロック・トランザクション |
| リスク | 大容量転送・長時間処理・不正な同期データ | 遅延・ピア切断・競合ブロック |
| 監視 | 進捗、受信量、検証失敗、ディスク | Tip差分、接続数、処理遅延 |

## 21.8 ネットワーク越しの同期は信頼できない

同期の入力は、すべて未信頼として扱うべきである。

考慮すべき例：

- 不正なヘッダーや存在しない親を送るピア
- コミットメントと一致しないUTXOスナップショット
- 異常に大きいメッセージや大量のエントリ
- 途中切断・再送・順序の入れ替わり
- 古いバージョンとの互換性問題
- 同期中にPruning Pointが進むことによる再試行
- 複数ピアが矛盾した状態を提示するケース

したがって、**サイズ制限・タイムアウト・検証前のリソース制限・インポートの原子性・再開可能性**が重要になる。

基準版のheaders proof経路はstaging consensusを作り、証明・ヘッダー・Pruning point・時刻・UTXO集合の検証へ進む。成功するとstagingをcommitし、エラーならcancelする。通信の分割単位や上限・タイムアウトは対象版のflow/stream実装で確認し、別バージョンのリリース説明を基準版へ読み替えない。

`import_pruning_point_utxo_set` 単体は、MuHashの一致後にPruning用UTXOの位置を保存し、VirtualのUTXO集合をクリアして分割コピーし、その後に対象ブロックの取引も検証する。単一WriteBatchで全部を差し替える関数ではない。外側のstaging境界と関数内部の更新を分け、ソース確認だけで電源断時の耐久性を保証しない。

## 21.9 ストレージとコンセンサスの責務を分離する

```text
┌────────────────────────────────────┐
│ P2P / Sync                         │
│ 取得・再試行・転送・ピア管理         │
└────────────────┬───────────────────┘
                 │ 未信頼データ
                 ▼
┌────────────────────────────────────┐
│ Consensus Validation               │
│ 証明・PoW・DAG・UTXO整合性の検証    │
└────────────────┬───────────────────┘
                 │ 検証済み状態
                 ▼
┌────────────────────────────────────┐
│ Storage                            │
│ 原子的な保存・索引・削除・復旧      │
└────────────────────────────────────┘
```

責務の境界を明確にする。

- **P2P層**：データを運ぶ。正当性の最終判断をしない。
- **コンセンサス層**：受信内容がルールに適合するか判定する。
- **ストレージ層**：整合した状態を安全に保持し、再起動後も復元できるようにする。
- **RPC／インデックス層**：利用者の問い合わせを処理する。コンセンサスの真実を勝手に変更しない。

この分離はDDDでいう境界づけられたコンテキストや、C4モデルのコンポーネント図で表現できる。

## 21.10 同期の状態遷移と不変条件

次は**教育用の簡略ステートマシン**である。

```text
Disconnected
    │ connect
    ▼
Discovering
    │ choose peer
    ▼
DownloadingProof
    │ verify
    ▼
DownloadingSnapshot
    │ verify commitment
    ▼
CatchingUp
    │ reach network tip
    ▼
Synced
    │ disconnect / large gap
    └───────────────▶ Recovering ──▶ Discovering
```

実装上、少なくとも次の不変条件を検討したい。

1. **未検証のスナップショットは有効なUTXO状態として公開しない。**
2. **検証失敗時に公開状態を保つ境界を明示する。関数内・staging・永続化の各範囲を区別する。**
3. **受信件数・バイト数・処理時間には上限を設ける。**
4. **同期元ピアの発言だけで最終的なコンセンサス状態を確定しない。**
5. **Pruning後も、新規ブロック検証に必要なデータを保持する。**
6. **再起動や途中切断によって中途半端な状態を有効化しない。**

## 21.11 Rust実装：最小スナップショット検証モデル

実際のKaspaはMuHashなどを使用するが、まずは「**コミットメントを検証してから状態を差し替える**」という設計原則を小さなRustコードで学ぶ。

このサンプルは教育用であり、KaspaのUTXO形式・MuHash・Pruning Proofを再現しない。

本書とは別の作業用ディレクトリで `cargo new --lib --edition 2021 snapshot-lab` を実行し、次のCargo.tomlとsrc/lib.rsへ置き換える。Rust 1.90.0、sha2 0.10.9で確認した。

### Cargo.toml

```toml
[package]
name = "snapshot-lab"
version = "0.1.0"
edition = "2021"

[dependencies]
sha2 = "=0.10.9"
```

### src/lib.rs

```rust
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub type UtxoSet = BTreeMap<String, u64>;

#[derive(Debug, PartialEq, Eq)]
pub enum ImportError {
    EmptyOutpoint,
    KeyTooLong,
    DuplicateOutpoint,
    CommitmentMismatch,
    TooManyEntries,
}

/// 教育用の決定的なコミットメント。KaspaのMuHashではない。
pub fn commitment(utxos: &UtxoSet) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"mastering-kaspa/snapshot-lab/v1\0");
    for (outpoint, amount) in utxos {
        let key = outpoint.as_bytes();
        hasher.update((key.len() as u64).to_le_bytes());
        hasher.update(key);
        hasher.update(amount.to_le_bytes());
    }
    hasher.finalize().into()
}

/// 全件検証後にのみ既存状態を置き換える。
pub fn import_snapshot(
    current: &mut UtxoSet,
    entries: Vec<(String, u64)>,
    expected: [u8; 32],
    max_entries: usize,
    max_key_bytes: usize,
) -> Result<(), ImportError> {
    if entries.len() > max_entries {
        return Err(ImportError::TooManyEntries);
    }

    let mut candidate = UtxoSet::new();
    for (outpoint, amount) in entries {
        if outpoint.is_empty() {
            return Err(ImportError::EmptyOutpoint);
        }
        if outpoint.len() > max_key_bytes {
            return Err(ImportError::KeyTooLong);
        }
        if candidate.insert(outpoint, amount).is_some() {
            return Err(ImportError::DuplicateOutpoint);
        }
    }

    if commitment(&candidate) != expected {
        return Err(ImportError::CommitmentMismatch);
    }

    *current = candidate;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_snapshot() {
        let mut expected_set = UtxoSet::new();
        expected_set.insert("tx-a:0".into(), 100);
        let expected = commitment(&expected_set);

        let mut current = UtxoSet::new();
        import_snapshot(&mut current, vec![("tx-a:0".into(), 100)], expected, 10, 64).unwrap();
        assert_eq!(current, expected_set);
    }

    #[test]
    fn mismatch_does_not_modify_current_state() {
        let mut current = UtxoSet::new();
        current.insert("old:0".into(), 50);
        let original = current.clone();

        let error = import_snapshot(&mut current, vec![("new:0".into(), 100)], [0; 32], 10, 64)
            .unwrap_err();

        assert_eq!(error, ImportError::CommitmentMismatch);
        assert_eq!(current, original);
    }

    #[test]
    fn rejects_duplicate_outpoints() {
        let mut current = UtxoSet::new();
        let error = import_snapshot(
            &mut current,
            vec![("tx-a:0".into(), 1), ("tx-a:0".into(), 2)],
            [0; 32],
            10,
            64,
        )
        .unwrap_err();
        assert_eq!(error, ImportError::DuplicateOutpoint);
        assert!(current.is_empty());
    }

    #[test]
    fn commitment_matches_independent_vector() {
        let snapshot = UtxoSet::from([("tx-a:0".into(), 100)]);
        // 固定のバイト列をPython hashlibへ渡して得た期待値。
        assert_eq!(
            commitment(&snapshot),
            [
                0x14, 0x5c, 0x04, 0xec, 0xfe, 0x64, 0x7a, 0x47, 0x60, 0x54, 0x39, 0xee, 0x76, 0x33,
                0xcf, 0x31, 0xcc, 0x92, 0x74, 0x2e, 0x6d, 0xe8, 0x3a, 0x12, 0x89, 0xef, 0x36, 0x65,
                0xdf, 0xaf, 0x2a, 0xa2
            ]
        );
    }

    #[test]
    fn input_order_and_reimport_preserve_the_snapshot() {
        let expected_set = UtxoSet::from([("a:0".into(), 1), ("b:1".into(), 2)]);
        let expected = commitment(&expected_set);
        let mut current = UtxoSet::new();
        for entries in [
            vec![("b:1".into(), 2), ("a:0".into(), 1)],
            vec![("a:0".into(), 1), ("b:1".into(), 2)],
        ] {
            import_snapshot(&mut current, entries, expected, 2, 3).unwrap();
            assert_eq!(current, expected_set);
        }
    }

    #[test]
    fn every_rejection_preserves_existing_state() {
        let before = UtxoSet::from([("old:0".into(), 50)]);
        let mut current = before.clone();
        for (entries, max_entries, max_key_bytes, error) in [
            (vec![("".into(), 1)], 10, 64, ImportError::EmptyOutpoint),
            (vec![("long".into(), 1)], 10, 3, ImportError::KeyTooLong),
            (
                vec![("x:0".into(), 1), ("x:0".into(), 2)],
                10,
                64,
                ImportError::DuplicateOutpoint,
            ),
            (vec![("x:0".into(), 1)], 0, 64, ImportError::TooManyEntries),
            (
                vec![("x:0".into(), 1)],
                10,
                64,
                ImportError::CommitmentMismatch,
            ),
        ] {
            assert_eq!(
                import_snapshot(&mut current, entries, [0; 32], max_entries, max_key_bytes),
                Err(error)
            );
            assert_eq!(current, before);
        }
    }

    #[test]
    fn byte_limits_and_empty_snapshot_are_explicit() {
        let expected_set = UtxoSet::from([("あ".into(), 1)]);
        let expected = commitment(&expected_set);
        let mut current = UtxoSet::new();
        assert_eq!(
            import_snapshot(&mut current, vec![("あ".into(), 1)], expected, 1, 2),
            Err(ImportError::KeyTooLong)
        );
        import_snapshot(&mut current, vec![("あ".into(), 1)], expected, 1, 3).unwrap();
        assert_eq!(current, expected_set);
        let empty = UtxoSet::new();
        import_snapshot(&mut current, vec![], commitment(&empty), 0, 0).unwrap();
        assert!(current.is_empty());
    }

    #[test]
    fn rejects_oversized_snapshot() {
        let mut current = UtxoSet::new();
        let error =
            import_snapshot(&mut current, vec![("tx-a:0".into(), 1)], [0; 32], 0, 64).unwrap_err();
        assert_eq!(error, ImportError::TooManyEntries);
    }
}
```

実行例：

```bash
cargo test
```

本文から抽出した教材は8テスト成功（`experiments/chapter-21-review-2026-11-01.md`）。固定のdomain/versionタグ、BTreeMapのキー順、キー長・キー・金額のエンコードを用いる。Python hashlibから得た既知値、受信順の反転、再インポート、全拒否理由で旧集合の維持、UTF-8のバイト上限と空集合を確認した。

`expected` の由来は呼び出し元の責任である。同じ未信頼の相手から届いた値との一致だけでは、正しいスナップショットの採用を保証しない。

> **実装上の限界**：このコードはメモリ上での原子的な置き換えを示すだけであり、ディスク書き込み・クラッシュ耐性・チャンク転送・ヘッダーの正当性検証・MuHashは含まない。キーは単なる文字列で本物のOutpointではなく、金額と文字列だけをハッシュするためScript・Coinbaseフラグ・DAA score等も含まない。件数とキー長を確認する時点でVecは受信・確保済みであり、受信前のメモリ上限・時間制限の代用ではない。実ノードの永続化・stagingの境界は別途確認する。

## 21.12 ディスク障害・再起動・ロールバック

ノードの状態が壊れる典型的な経路を考える。

```text
スナップショット受信
       │
       ▼
一時領域へ書き込み
       │
       ▼
完全性・コミットメント検証
       │
       ▼
検証済み状態としてコミット
       │
       ▼
旧状態の安全な解放
```

これは**望ましい設計原則**を示すものであり、`rusty-kaspa` が必ずこの順序で実装されているという断定ではない。

クラッシュ耐性を検証するなら、次のタイミングでプロセスを強制終了する実験が有効である。

- 受信中
- 一時保存中
- 検証完了直前
- 永続化のコミット中
- 古いデータの削除中

再起動後、状態が「以前の正常状態」または「新しい正常状態」のどちらかに復元できるか確認する。

## 21.13 ストレージ肥大化とインデックス

「Pruningしているのにディスクが増える」という現象は、必ずしも不具合ではない。

原因の候補は次のとおり。

- UTXO集合自体が増加している
- 必要なヘッダーやDAGメタデータが増えている
- DBの内部領域が再利用されてもファイルサイズがすぐ縮まない
- インデクサー・ログ・バックアップが別に保存されている
- ノード設定やバージョンによって保存対象が異なる

**論理的に削除したデータ量**と**OSから見えるディスク使用量**は一致しない場合がある。ディスク調査では、DB本体・ログ・インデックス・一時ファイルを分けて測る。

## 21.14 ノード同期の観測ポイント

| 観測対象 | 見たいこと | 疑う問題 |
|---|---|---|
| ピア接続数 | 安定して接続できているか | ネットワーク設定・到達性 |
| 同期進捗 | 進んでいるか・停滞しているか | データ取得・検証・ディスク |
| Tip差分 | ネットワークとの差が縮むか | ピア品質・処理能力 |
| 受信帯域 | 取得速度は十分か | 帯域制限・ピア切断 |
| ディスクI/O | 書き込みが詰まっていないか | ストレージ性能 |
| メモリ | 大きなデータの受信で急増しないか | バッファ管理・リーク |
| 検証エラー | どの段階で失敗するか | 不正データ・互換性 |

同期進捗が「100%に見える」のにノードが利用可能とは限らない。**同期状態、Virtual Stateの追従、RPCの応答、UTXOインデックスの準備状況**を分けて確認する。

## 21.15 `rusty-kaspa` を読むための調査手順

公式リポジトリ：[`kaspanet/rusty-kaspa`](https://github.com/kaspanet/rusty-kaspa)

最初から全コードを読むのではなく、**入口→検証→保存→公開**の順に追う。

```bash
# 第1章で取得した固定版の親ディレクトリから実行する
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
grep -n -E 'import_pruning_point_utxo_set|ImportedMultisetHashMismatch' consensus/src/pipeline/virtual_processor/processor.rs
grep -n -E 'validate_pruning_point_proof' consensus/src/processes/pruning_proof/validate.rs
grep -n -E 'DownloadHeadersProof|staging|sync_pruning_point_utxoset' protocol/flows/src/v5/ibd/flow.rs
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)の手順を使い、HEADを本章冒頭の完全SHAと照合する。検索語の一致だけでなく、検証・更新・公開の境界を追う。

### 調査ノートのテンプレート

```markdown
## 調査対象
- Repository:
- Commit SHA:
- Network:
- Node version:

## 入力
- どのP2Pメッセージ/RPC/内部イベントが起点か

## 検証
- どの関数が何を検証するか
- 失敗時のエラーは何か

## 状態更新
- どのストアを書き換えるか
- 原子的な境界はどこか

## 復旧
- 中断・再起動時の動作

## 未確認事項
- 推測せず保留する項目
```

**ソースコードの関数名だけで仕様を断定しないこと。** 呼び出し元、テスト、パラメータ、アップグレード条件まで追う。

## 21.16 Dockerによる観察実験

[第28章28.6](chapter-28.md#286-ローカル環境の準備)で環境を準備し、[28.7](chapter-28.md#287-dockerで単一ノードを起動する)の収録済みCompose構成で`node`サービスを起動する。起動・終了は同節の手順に従い、以下の観察コマンドは本書リポジトリのルートで、ノードが起動している間に実行する。

```bash
# 収録Compose構成のnodeサービスのログ
docker compose -f experiments/devnet/compose.yaml logs --tail=200 node

# CPU・メモリ等の現在値を一回表示
docker compose -f experiments/devnet/compose.yaml stats --no-stream node

# 稼働中のnodeコンテナを取得してイメージIDを表示
KASPA_NODE_ID=$(docker compose -f experiments/devnet/compose.yaml ps -q node)
docker inspect "$KASPA_NODE_ID" --format '{{.Image}}'
```

ログから、同期フェーズの変化・ピア切断・Pruning関連メッセージを探す。

再起動時の状態保持は[既存Devnet実習の比較手順](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#短時間採掘と再起動後の状態比較)、2ノードの停止・復帰後の追従は[28.8](chapter-28.md#288-2ノード構成とピア接続)の実習へ進む。これらは実際のPruning後IBDを再現する試験ではない。Pruning pointがGenesisのままの短い実習と、Pruning Proof・UTXO転送を使った同期は区別して記録する。

**本番ノードのデータディレクトリを削除したり、異なるネットワークのデータを使い回したりしないこと。** 初期同期実験は、独立したデータディレクトリまたはDevnetで行う。

### 計測表

| 試行 | ノード版 | ネットワーク | ストレージ | 同期所要時間 | 最大ディスク使用量 | 備考 |
|---|---|---|---|---|---|---|
| A | 記録 | Devnet | SSD | 計測 | 計測 | 初回同期 |
| B | 記録 | Devnet | SSD | 計測 | 計測 | 再起動後 |
| C | 記録 | Devnet | SSD | 計測 | 計測 | ピア切断 |

実測値を記録し、推測値と区別する。ログやinspectの生出力には内部情報が含まれ得るため、そのまま公開せず必要な集計・版・イメージ識別情報へ整理する。

## 21.17 障害シナリオと原因の切り分け

### ケースA：同期が途中で止まる

1. ピア接続が維持されているか。
2. ログに証明検証・UTXOインポート・タイムアウトのエラーがないか。
3. ディスク容量・I/O・メモリ不足がないか。
4. 使用バージョンに既知の同期不具合がないか。
5. 同じ条件で再現するか。

### ケースB：スナップショットのコミットメント不一致

- 入力データの破損・欠落・重複を疑う。
- ヘッダー側の参照点が一致しているか調べる。
- 検証失敗後にローカル状態が変更されていないか確認する。
- 異常なピアへの対処は実装のポリシーに従う。

### ケースC：同期後もアプリケーションからトランザクションが見えない

- ノード同期とアドレス用インデックスの同期を区別する。
- RPCで提供されるデータ範囲を確認する。
- 履歴検索が必要なら専用インデクサーの要否を判断する。

## 21.18 Bitcoinとの比較

| 観点 | Bitcoin | Kaspa |
|---|---|---|
| 基本構造 | 競合する枝から採用チェーンを選ぶ | BlockDAG |
| 状態 | UTXO集合 | UTXO集合 |
| 古いデータの削減 | ブロック本体のPruningが可能 | Pruning Pointを用いたDAG履歴削減 |
| 新規同期の基礎 | ヘッダー・ブロック検証など | Pruning Proof・UTXO commitment・追いつき処理など |
| 実装上の難しさ | 線形チェーンの再編成と状態管理 | DAG関係・コンセンサス順序・状態管理 |

両者とも「現在状態の検証」が重要だが、**KaspaのPruning Proofによる同期をBitcoinの標準IBDと同じものとして説明してはいけない**。

## 21.19 よくある誤解

**Q1. Pruningするとノードは検証しなくなる？**  
A. いいえ。Pruningは保存履歴の削減であり、検証能力を放棄することではない。

**Q2. UTXO集合だけダウンロードすれば安全？**  
A. いいえ。コミットメントが一致することと、そのコミットメントを含むヘッダーが正しいコンセンサス履歴に属することは別の検証である。

**Q3. Pruning Point以前のデータは一切残らない？**  
A. いいえ。証明や検証に必要な情報は保持される。削除対象はデータの種類と規則によって異なる。

**Q4. Archiveノードであればすべてのアドレス履歴を即座に検索できる？**  
A. 必ずしもそうではない。データの保持と検索用インデックスの構築は別の要件である。

**Q5. FinalityとPruningは同じ？**  
A. いいえ。前者はコンセンサス上の履歴変更制約、後者はストレージ削減の仕組みである。

## 21.20 実践課題

**課題1：責務図**  
IBDで受信したUTXO集合がローカルの有効な状態になるまでを、P2P・コンセンサス・ストレージの3つの境界で描く。

**課題2：障害注入**  
21.11の既存テストを読み、重複Outpoint・コミットメント不一致・件数超過が、それぞれどの検査で拒否されるか確認する。そのうえで、複数エントリからなる正しい候補集合のコミットメントを計算し、受け渡すエントリのうち1件の金額だけを変更するテストを追加する。期待値には改ざん前のコミットメントを渡し、`CommitmentMismatch` が返ることと、空ではない既存UTXO集合が全体として維持されることを確認する。

**課題3：永続化**  
候補スナップショットを一時ファイルへ保存し、検証成功時だけ正式ファイルへ置き換える処理を設計する。ファイルの原子的置換とディレクトリ同期の必要性も調査する。

**課題4：ソースコード追跡**  
`rusty-kaspa` の特定コミットで `import_pruning_point_utxo_set` の呼び出し元、失敗時の挙動、永続化境界を追跡し、根拠となる行番号を記録する。

**課題5：同期の性能評価**  
Devnet上で初期同期・再起動・一時的なネットワーク切断を比較し、時間・メモリ・ディスク・ログを測定する。

## 21.21 理解度チェック

1. Pruningと全履歴保存の違いは何か。
2. UTXO集合だけでは過去の全トランザクション履歴を検索できないのはなぜか。
3. Pruning PointとFinalityはどう異なるか。
4. Pruning ProofとUTXO commitmentは何をそれぞれ検証するか。
5. 同期元ピアを無条件に信頼できないのはなぜか。
6. UTXOインポートで原子性が重要なのはなぜか。
7. ノードが同期済みでもアプリケーションのインデックスが未完成の場合、何が起きるか。
8. Rustの教育用SHA-256コミットメントをKaspaのMuHashと同一視できないのはなぜか。

## 21.22 まとめ

この章の中心は、**「データを削減すること」と「検証可能性を保つこと」の両立**である。

KaspaのPruningは、BlockDAGの保存コストを抑えながら、新しいノードが検証可能な状態へ同期するための仕組みと結びついている。そのために、Pruning Point、Pruning Proof、UTXO commitment、IBDがそれぞれ異なる責務を担う。

開発者として特に重要なのは、**P2Pで受け取ったデータを信用せず、コンセンサスで検証し、ストレージへ安全にコミットする**という境界を明確にすることである。

次章では、このノード内部の状態を外部アプリケーションから利用するための **[Chapter 22 — Kaspa RPC, wRPC & Application Integration](chapter-22.md)** に進む。

---

## 21.23 参考資料・一次情報

1. [kaspanet/rusty-kaspa — 公式Rust実装](https://github.com/kaspanet/rusty-kaspa)
2. [rusty-kaspa Releases — IBD・Pruningの更新履歴](https://github.com/kaspanet/rusty-kaspa/releases)
3. [Consensus constants — Finality / Pruningの定数](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/constants.rs)
4. [Consensus parameters — ネットワーク別設定](https://github.com/kaspanet/rusty-kaspa/blob/fcd9c28f9b211b9872a0c90a8c94f79466ffb063/consensus/core/src/config/params.rs)
5. [Kaspa KIPs — 改善提案一覧](https://github.com/kaspanet/kips)
6. [KIP-0006 — Proof of Chain Membership](https://github.com/kaspanet/kips/blob/master/kip-0006.md)（履歴・取引の所属証明に関する文書。Pruning Proofそのものと同一視しない）
7. [KIP-0015 — Canonical Transaction Ordering and SelectedParent Accepted Transactions Commitment](https://github.com/kaspanet/kips/blob/master/kip-0015.md)
8. [Issue #1100 — Pruning Point UTXO snapshotの敵対的テスト](https://github.com/kaspanet/rusty-kaspa/issues/1100)（追加試験・防御の提案。固定版で試験が完了した証拠ではない）
9. [Bitcoin Developer Reference](https://developer.bitcoin.org/reference/)

> **技術レビュー時の確認事項**：本章の概念図・教育用実装と、`rusty-kaspa` の現行コードは厳密に区別すること。同期フェーズ・メッセージ名・Pruning保持規則・時間定数は、出版時に固定したGitコミットとネットワーク設定に基づいて再確認する。

目次順の本文レビューと教材8テストは `experiments/chapter-21-review-2026-11-01.md`。KIPの文書ステータス・Issueの議論と、基準版の有効規則・実行済みの検証結果を区別する。実Pruning・Pruning後IBD・敵対的インポート・クラッシュ耐久性は今回実行していない。
