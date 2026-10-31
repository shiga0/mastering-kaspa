# Chapter 25 — Exploring the rusty-kaspa Codebase（rusty-kaspaのコードベースを読む）

> **原稿ステータス:** v0.1（技術レビュー前）  
> **対象読者:** Rustの基礎を理解し、Kaspaの実装を追跡したい開発者  
> **この章の目的:** `rusty-kaspa` を「ファイルの集合」ではなく、責務・依存関係・データフローから読み解く。

## 25.1 本章の到達目標

本章を読み終えたら、次のことができるようになる。

1. リポジトリの構造とCargoワークスペースを調査できる。
2. P2P、コンセンサス、ストレージ、RPC、ウォレットの境界を説明できる。
3. 「トランザクション送信」「ブロック受信」などのユースケースを入口から追跡できる。
4. 型、トレイト、非同期処理、エラー、テストから実装上の不変条件を見つけられる。
5. 調査結果を再現可能な形で記録し、小さな変更とテストに結び付けられる。

> **バージョンについて:** 本章ではディレクトリ名や関数名を固定的な仕様として扱わない。`rusty-kaspa` は更新されるため、手元のコミットで確認した事実と、概念的なアーキテクチャを区別する。

## 25.2 なぜソースコードを読むのか

論文はアルゴリズムの目的や性質を説明し、KIPは変更提案や設計意図を説明する。一方、実装には、入力検査、キャッシュ、永続化、エラー処理、並行実行、テストといった運用上の詳細が存在する。

| 資料 | 主に分かること | 注意点 |
|---|---|---|
| 論文 | モデル・前提・安全性の議論 | 実装の細部を必ずしも規定しない |
| KIP | 変更の動機・提案仕様 | 提案と導入済みの機能は異なる |
| ソースコード | 現在の実装・依存関係 | ブランチとコミットに依存する |
| テスト | 実装が保証しようとする振る舞い | テストにない性質まで保証しない |
| 実行ログ | 実環境で起きたこと | 観測だけでは原因を確定できない |

重要なのは、これらを相互参照することである。

## 25.3 調査環境の準備

公式リポジトリを取得する。

```bash
 git clone https://github.com/kaspanet/rusty-kaspa.git
 cd rusty-kaspa
 git rev-parse HEAD
 git status --short
 rustc --version
 cargo --version
```

最初の行の先頭スペースはシェルに影響しないが、実行時には省いてよい。調査対象のコミットを必ずメモする。再現性を高めるなら、タグや特定コミットをチェックアウトする。

```bash
# ワークスペースのメンバーと依存関係を把握する
sed -n '1,220p' Cargo.toml
cargo metadata --no-deps --format-version 1 > /tmp/kaspa-metadata.json

# 主要なRustファイルの一覧
find . -name '*.rs' -not -path './target/*' | head -80
```

`cargo metadata` は、複数クレートを持つ大規模なRustプロジェクトの入口として便利である。

### 25.3.1 調査の再現性

調査メモには、最低限以下を残す。

```text
Repository: kaspanet/rusty-kaspa
Commit: <git rev-parse HEAD の結果>
Toolchain: <rustc --version の結果>
Question: ブロックを受信した後、どこで検証されるか？
Entry point: <確認した型・関数・ファイル>
Evidence: <コード位置とテスト>
Open questions: <未確認事項>
```

## 25.4 全体像をC4モデルで把握する

C4モデルの観点から、最初は詳細な関数ではなく「何が誰に依存するか」を把握する。

### Context：外部との関係

```text
                   ┌─────────────────┐
                   │ Wallet / App    │
                   └────────┬────────┘
                            │ RPC / SDK
                            ▼
  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐
  │ Peer Node A │◀───▶│ Kaspa Node   │◀───▶│ Peer Node B  │
  └─────────────┘ P2P └──────┬───────┘ P2P └──────────────┘
                             │
                             ▼
                      ┌──────────────┐
                      │ Local Storage│
                      └──────────────┘
```

### Container / Component：ノード内部

```text
RPC Server ────────┐
                   ▼
              Node Services
                   │
P2P Networking ────┤
                   ▼
          Consensus Interfaces
                   │
        ┌──────────┼───────────┐
        ▼          ▼           ▼
   BlockDAG     UTXO State   Difficulty
        └──────────┼───────────┘
                   ▼
             Storage Layer
```

これは**責務を理解するための概念図**であり、実際のクレート間依存や呼び出し順序を正確に表したものではない。具体的な依存方向は `Cargo.toml` とコードから検証する。

## 25.5 Cargoワークスペースとクレートの読み方

Rustでは、ひとつのリポジトリに複数のクレートが存在し得る。各クレートは公開APIと内部実装の境界を持つ。

最初に確認する項目は次のとおり。

- ルート `Cargo.toml` の `[workspace]` と `members`
- 各クレートの `Cargo.toml` と `dependencies`
- `src/lib.rs` または `src/main.rs`
- `pub`、`pub(crate)`、非公開の境界
- `trait` とその実装箇所
- `#[cfg(feature = "...")]` とフィーチャーフラグ

```bash
# 利用できる場合、依存グラフを表示
cargo tree --depth 2

# 機能別に入口を探す
rg 'trait .*Consensus|struct .*Consensus' --glob '*.rs'
rg 'submit_block|validate_block|process_block' --glob '*.rs'
rg 'get_block_template|submit_transaction' --glob '*.rs'
```

検索語は**探索用の候補**であり、現行実装に必ず同名の識別子が存在するという意味ではない。`rg` が未導入なら `grep -R` を使用する。

## 25.6 主要な責務を分解する

以下は調査のための分類であり、実際のモジュール名と一対一に対応するとは限らない。

| 領域 | 主な責務 | 読むときの質問 |
|---|---|---|
| P2P | ピア接続、メッセージ受信・送信 | 未信頼入力をどこで検査するか |
| Consensus | DAG、順序付け、状態計算 | どの条件で状態が変わるか |
| Storage | ブロック・索引・状態の永続化 | 更新の原子性をどう確保するか |
| Mempool | 未確定トランザクションの検査と管理 | 競合入力をどう扱うか |
| Mining | ブロックテンプレート作成 | DAGとMempoolの整合をどう取るか |
| RPC | 外部APIとイベント配信 | 内部エラーをどう公開するか |
| Wallet / SDK | 鍵、トランザクション構築、クライアント接続 | 秘密鍵が通る境界はどこか |

## 25.7 ユースケースからコードを追跡する

巨大なリポジトリを先頭から順番に読む必要はない。**ひとつの出来事を選び、入口から出口まで追う**。

### 例A：ブロック受信

```text
Peer
  │ message
  ▼
P2P message handler
  │ decode / basic checks
  ▼
Block processing entry
  │ consensus checks
  ▼
BlockDAG / consensus state
  │ persistence / notifications
  ▼
Storage + downstream consumers
```

この図も概念図である。実装では並列化、キューイング、早期リターン、キャッシュなどにより順序が変わり得る。

調査手順：

1. ブロックを表す型とP2Pメッセージ型を探す。
2. 受信ハンドラーから呼ばれる処理を辿る。
3. `Result` のエラー型を確認する。
4. 検証に失敗した場合、永続状態が変わらないかを確認する。
5. 成功時のイベント通知・キャッシュ更新・永続化を確認する。
6. 対応するテストを探す。

### 例B：トランザクション送信

```text
Application
   │ RPC request
   ▼
RPC boundary
   │ parse / authorize / route
   ▼
Transaction processing
   │ checks / UTXO dependencies
   ▼
Mempool
   │ accepted locally
   ▼
P2P propagation
```

**RPC成功 ≠ コンセンサスでの最終的な受理**。この違いを、返り値と状態追跡APIの両方から確認する。

### 例C：ブロックテンプレート作成

```text
Miner / Mining Client
         │
         ▼
  Template Request
         │
    ┌────┴────┐
    ▼         ▼
 BlockDAG   Mempool
    └────┬────┘
         ▼
  Block Candidate
         │
         ▼
      PoW Work
```

ここでは、親ブロックの選択、トランザクション選択、Coinbase、難易度関連フィールドの出所を調べる。

## 25.8 型を読む：データモデルと不変条件

関数の中身より先に、入出力の型を読むと理解が速い。

```rust
// 以下は教育用の例であり、rusty-kaspaの実際の型ではない。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockId([u8; 32]);

#[derive(Clone, Debug)]
pub struct BlockHeader {
    pub parents: Vec<BlockId>,
    pub timestamp_ms: u64,
    pub nonce: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    MissingParent,
    InvalidProofOfWork,
    InvalidTimestamp,
}

pub trait BlockValidator {
    fn validate(&self, header: &BlockHeader) -> Result<(), ValidationError>;
}
```

この例から読み取れるのは、`BlockValidator` が入力とエラーの契約を公開しているという点である。ただし、`parents` が空でよいか、時刻がどの範囲なら有効か、PoWがどう計算されるかは、型定義だけでは分からない。呼び出し側とテストを確認する必要がある。

### 25.8.1 不変条件の例

- 参照する親は、必要な文脈で解決可能である。
- 不正なPoWを持つブロックは受理されない。
- 検証失敗時に、受理済みのコンセンサス状態を不正に変更しない。
- UTXOは、同じコンセンサス状態の中で二重消費されない。
- 同じ入力を再処理しても、二重適用されない設計になっている。

これらは**調査時に確認すべき性質**であり、個別の処理段階に対して無条件に成立すると断定してはいけない。

## 25.9 非同期処理と並行性を読む

ノードはP2P通信、RPC、コンセンサス処理、ストレージI/Oを並行して扱う。Rustコードでは以下に注目する。

- `async fn` と `.await` の境界
- `tokio::spawn` 等によるタスク起動
- `mpsc`、`oneshot` などのチャネル
- `Arc`、`Mutex`、`RwLock` 等の共有状態
- キャンセル・タイムアウト・シャットダウン処理
- キューの容量とバックプレッシャー

```text
Inbound Messages
       │
       ▼
  Bounded Queue ── full ──▶ backpressure / reject
       │
       ▼
   Worker Tasks
       │
       ▼
 Consensus Service
       │
       ▼
  Storage / Events
```

**キューを無制限に増やさないこと**は、メモリ枯渇やDoSを避けるうえで重要である。ただし、具体的な制限値は実装と設定に依存する。

## 25.10 エラー処理を読む

エラーを単にログへ出すだけでは不十分である。次の分類で調査する。

| エラー種別 | 例 | 調査の焦点 |
|---|---|---|
| 入力エラー | 不正なメッセージ形式 | ピア切断・拒否の条件 |
| コンセンサスエラー | 無効なブロック | 永続化や通知が起きないか |
| 一時的エラー | 親ブロック未取得 | 後で再試行できるか |
| ストレージエラー | 書き込み失敗 | 状態が部分更新されないか |
| 通信エラー | ピア切断 | 再接続と同期回復 |
| 内部不整合 | 想定外の状態 | ログ・メトリクス・停止条件 |

`?` 演算子で伝播したエラーが最終的にどこで処理されるかを追う。エラーを返す関数だけでなく、その呼び出し元が重要である。

## 25.11 テストから仕様を発見する

実装を読む際、対応するテストは強力な手掛かりとなる。

```bash
# テスト候補を探す
rg '#\[test\]|#\[tokio::test\]' --glob '*.rs' | head -80

# 特定のクレートだけテストする（名前はCargo.tomlで確認）
cargo test -p <package-name>

# 名前でテストを絞る
cargo test <test-name-fragment> -- --nocapture
```

**注意:** `<package-name>` と `<test-name-fragment>` はプレースホルダーであり、そのまま実行しない。

テストを読むときは、次の4点を記録する。

1. 初期状態は何か。
2. どの入力を与えるか。
3. 何が変化するか。
4. 何が**変化してはいけない**か。

成功ケースだけでなく、失敗時の不変条件を見ることが重要である。

## 25.12 Rustによる小さな実験：状態更新の原子性

以下は `rusty-kaspa` のコードではなく、状態変更を安全に行うための教育用モデルである。**検証してから状態を変更する**という基本原則を確認する。

```rust
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
enum Error {
    AlreadyKnown,
    MissingParent,
}

#[derive(Default)]
struct DagStore {
    known: HashSet<u64>,
}

impl DagStore {
    fn insert(&mut self, id: u64, parents: &[u64]) -> Result<(), Error> {
        if self.known.contains(&id) {
            return Err(Error::AlreadyKnown);
        }
        if parents.iter().any(|parent| !self.known.contains(parent)) {
            return Err(Error::MissingParent);
        }
        self.known.insert(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_block_after_parent() {
        let mut store = DagStore::default();
        assert_eq!(store.insert(1, &[]), Ok(()));
        assert_eq!(store.insert(2, &[1]), Ok(()));
        assert!(store.known.contains(&2));
    }

    #[test]
    fn invalid_block_does_not_mutate_state() {
        let mut store = DagStore::default();
        let before = store.known.clone();
        assert_eq!(store.insert(2, &[99]), Err(Error::MissingParent));
        assert_eq!(store.known, before);
    }

    #[test]
    fn duplicate_is_rejected() {
        let mut store = DagStore::default();
        assert_eq!(store.insert(1, &[]), Ok(()));
        assert_eq!(store.insert(1, &[]), Err(Error::AlreadyKnown));
    }
}
```

実行手順：

```bash
cargo new dag-store-lab --lib
cd dag-store-lab
# 上のコードを src/lib.rs に貼り付ける
cargo test
```

このモデルにはPoW、GHOSTDAG、永続化、ネットワーク、UTXO処理は含まれない。また、メモリ上の単一操作の性質を示すだけであり、実際のノードのトランザクション性を保証するものではない。

## 25.13 DDDの視点：境界づけられたコンテキスト

`rusty-kaspa` をDDDで読むなら、まず**概念的な境界**を整理する。

```text
[Networking] ── block / tx messages ──▶ [Consensus]
      │                                    │
      │                                    ▼
      └──────────────▶ [Mempool]        [Storage]
                           │
                           ▼
                        [Mining]

[RPC] ── requests / subscriptions ──▶ Node Services
```

これはドメイン境界を考えるための図であり、コードの依存方向や実際の呼び出し関係の断定ではない。

各領域について次の問いを立てる。

- **何を知っているべきか:** 例えばMempoolは未確定トランザクションの状態を管理する。
- **何を知らなくてよいか:** 例えばウォレットUIはGHOSTDAG内部の実装を知らなくてよい。
- **どの変更に影響されるか:** RPC仕様変更がコンセンサス内部に波及していないか。
- **どこで変換するか:** ネットワーク表現と内部ドメイン型の変換地点はどこか。

## 25.14 依存関係を可視化する

実装調査では「AがBを呼ぶ」だけでなく、「Bの変更がAに影響するか」を考える。

```text
┌─────────────────────────────────────────────┐
│ Application-facing interfaces               │
├─────────────────────────────────────────────┤
│ Node orchestration / services               │
├─────────────────────────────────────────────┤
│ Consensus domain / transaction rules        │
├─────────────────────────────────────────────┤
│ Persistence and external adapters           │
└─────────────────────────────────────────────┘
```

この積層図は説明用であり、実際の依存関係は必ずしも一方向ではない。循環依存を避けるための抽象化や、パフォーマンス上の理由で導入された構造を見つけたら、**設計上の理由**も記録する。

## 25.15 調査対象を小さくする技術

巨大なコードベースでは、次の順番が有効である。

1. **質問を1つに絞る:** 「ブロック受信後、PoW検証はどこで行われるか」。
2. **入口を見つける:** メッセージ型、RPCメソッド、CLI引数。
3. **型の流れを追う:** 変換、所有権、参照、戻り値。
4. **境界を記録する:** P2PからConsensusへ、ConsensusからStorageへ。
5. **テストを読む:** 正常系と異常系。
6. **実験する:** ログを増やす、テストを1つ追加する。
7. **結論と未確認事項を分ける。**

「全部理解してから実装する」のではなく、局所的な仮説と検証を積み重ねる。

## 25.16 実践課題A：ブロック処理の経路を調べる

**課題:** 手元のコミットで、ブロック受信からコンセンサス処理への経路を追跡する。

成果物の例：

```markdown
### 調査: ブロック受信
- Commit: ...
- 入口: ...
- 関連型: ...
- 呼び出し経路: ...
- 検証に失敗した場合: ...
- 参照したテスト: ...
- 未確認事項: ...
```

完了条件は、**最低3つの具体的なコード位置**と、**少なくとも1つのテスト**を根拠として示せること。

## 25.17 実践課題B：RPCからMempoolまでを追う

**課題:** トランザクション送信APIの入口を探し、Mempoolでの受理・拒否までを追う。

確認したい問い：

- APIはどの型を受け取るか。
- ネットワーク形式から内部形式への変換はどこか。
- 重複・競合トランザクションはどこで判断されるか。
- RPCエラーへどう変換されるか。
- ローカル受理後、P2P伝播はどこで開始されるか。

## 25.18 実践課題C：小さなテストを追加する

大規模な改修を行う前に、既存テストの近くへ**一つだけ**境界ケースを追加する。

例：

- 同じブロックの再受信
- 親がまだ到着していないブロック
- 同じUTXOを参照する競合トランザクション
- 未同期ノードに対するRPC要求

まず失敗するテストを作る必要があるとは限らない。既存の仕様を確認する**回帰テスト**でもよい。重要なのは、テストの意図と期待結果を説明できること。

## 25.19 ログ・メトリクス・障害調査

ノード実行時のログは、コード読解と対応付けて扱う。

| 観測 | 考えられる要因 | 次に確認するもの |
|---|---|---|
| ピア数が増えない | 接続先・ネットワーク設定・通信制限 | 起動引数とP2Pログ |
| 同期が進まない | ピアの状態・ディスク・検証負荷 | 同期状態とエラーログ |
| RPC応答が遅い | 処理待ち・I/O・高負荷 | メトリクスと呼び出し経路 |
| ブロックが拒否される | 検証条件不一致 | 検証エラー型とテスト |
| メモリが増え続ける | キャッシュ・キュー・リーク | キュー容量と保持オブジェクト |

一つの症状に原因は複数あり得る。ログの文字列だけで原因を断定しない。

## 25.20 ソースコード変更時の注意

- 作業前に対象コミットを固定する。
- 変更対象の責務を説明する。
- コンセンサス関連の変更は、ネットワーク互換性に影響する可能性がある。
- 公開APIの変更は、SDKや外部ツールへの影響を確認する。
- フォーマット・Lint・単体テストを実行する。
- 可能ならDevnetなど隔離環境で統合テストする。
- 実ネットワークへの適用は、レビューと互換性確認を経て行う。

```bash
cargo fmt --all -- --check
cargo test -p <package-name>
# lint対象・feature構成はリポジトリのCI設定に合わせる
```

## 25.21 よくある誤解

**Q. ディレクトリ名がそのままアーキテクチャなのか？**  
A. いいえ。クレート・モジュールの構造は重要な手掛かりだが、実際の責務は型・呼び出し・テストから確認する。

**Q. 論文にある概念と同名の関数があるはずか？**  
A. 必ずしもない。実装では処理が分割・統合・最適化される。

**Q. テストが通れば安全性は証明されたのか？**  
A. いいえ。テストは指定した条件下の振る舞いを検証する。形式的な証明や広範な脅威分析とは異なる。

**Q. blueブロックなら全トランザクションが有効か？**\
A. いいえ。ブロックのコンセンサス上の分類と、トランザクションの状態遷移上の受理は異なる。

**Q. 最初からすべてのクレートを理解する必要があるか？**  
A. ない。目的に沿った一つの経路を追い、周辺へ広げる方が効率的である。

## 25.22 理解度チェック

1. Cargoワークスペースの依存関係を確認する方法を2つ挙げよ。
2. P2Pメッセージ受信からブロック検証までの境界を説明せよ。
3. `Result` のエラー型を追うだけでは不十分なのはなぜか。
4. テストを読むとき、状態が「変わらないこと」を確認する理由は何か。
5. `Arc<Mutex<T>>` を見つけたら、どのような並行性の問題を調査するか。
6. RPCの成功とコンセンサス上の受理はなぜ異なるか。
7. ソースコード調査でコミットハッシュを残す理由を説明せよ。
8. KIPの提案と現在の実装をどう区別するか。

### 解答の要点

1. `Cargo.toml`、`cargo metadata`、`cargo tree` など。
2. 外部の未信頼データが内部の検証処理へ渡る境界。変換・入力検査・エラー処理が重要。
3. 呼び出し元が再試行・破棄・ログ・切断などを決めるため。
4. 不正入力による部分更新や二重適用を防ぐ設計を確認するため。
5. ロック競合、保持時間、デッドロック、共有状態の整合性など。
6. RPCはローカル処理の結果であり、DAGでの最終的なトランザクション受理を保証しないため。
7. コードは変更されるため、同じ実装を再調査できるようにするため。
8. KIPのステータス、対象バージョン、該当コード・テスト・リリース情報を照合する。

## 25.23 まとめ

`rusty-kaspa` を読むうえで重要なのは、個々の関数を暗記することではない。

- **責務:** どのコンポーネントが何を決めるか。
- **境界:** 外部入力がどこで検証されるか。
- **依存関係:** 変更がどこまで波及するか。
- **状態遷移:** 成功・失敗時に何が変化するか。
- **根拠:** コード、テスト、論文、KIPのどれに基づく結論か。

この5つを意識して、まずはブロック受信かトランザクション送信の経路を一つ追ってみよう。次章では、ここで学んだ読み方を使い、コンセンサス実装の構造にさらに踏み込む。

## 25.24 参考資料

- [Kaspa公式 `rusty-kaspa` リポジトリ](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [The Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [The Cargo Book — cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [C4 model](https://c4model.com/)

> **編集メモ:** 出版前に、対象とする `rusty-kaspa` のリリースまたはコミットを固定し、実際のクレート名・エントリーポイント・テスト名を検証して本文へ追記すること。
