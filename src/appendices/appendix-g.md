# Appendix G — 実験結果・テストベクトル

> **Mastering Kaspa / 付録G**  
> 対象：Rustによる教育用実装、Kaspa Devnet、コンセンサス関連の再現実験

本付録では、Kaspaの仕様を「理解したつもり」で終わらせず、**入力・期待値・実測値・実行環境を第三者が追跡できる形で残す**ための方法を示す。本付録は実行済みの記録と今後の計画を併記する。既存記録の照合を今回の再実行として扱わない。実際に実行していない試験を「成功」と記録してはならない。

対象版は[本書の対象バージョン](../baseline.md)を参照してください。

## G.1 目的と適用範囲

実験結果（experimental result）は、特定の環境で実際に観測した結果である。一方、テストベクトル（test vector）は、入力と期待出力を固定した検証用データである。この二つを分けて管理する。

| 種別 | 記録するもの | 例 |
|---|---|---|
| 仕様由来ベクトル | 仕様から導かれる入力・期待出力 | ハッシュの既知値 |
| 実装由来ベクトル | 参照実装で得た出力 | 特定コミットの関数出力 |
| 実験結果 | 実測値・環境・ログ | Devnet同期時間 |
| 性質テスト | 調べる入力範囲で成り立つべき条件 | UTXOの二重消費を拒否 |
| 差分テスト | 二つの実装の比較結果 | 教育用実装と参照実装 |

**注意：** 参照実装の出力は必ずしも独立した正しさの証明ではない。仕様との一致、実装間一致、ネットワークでの受理はそれぞれ別の検証対象となる。

## G.2 再現性の最低条件

実験には次の情報を添付する。

1. 実験ID、目的、仮説、判定基準。
2. 使用したOS・CPU・アーキテクチャ・Rust・Dockerのバージョン。
3. `rusty-kaspa`、SDK、実験コードの**GitコミットSHA**。未コミットの教材はその旨と掲載ソースのhash・変更内容も残す。
4. ネットワーク種別（Mainnet / Testnet / Devnet）、設定ファイル、起動引数。
5. 入力データ、期待出力、実測出力、標準出力・標準エラー。
6. 実行手順、所要時間、繰り返し回数、失敗の扱い。
7. 日時、実施者、結論、未解決事項。

生のstdout/stderrや設定は私有パス・peer識別子・鍵等を含み得る。保存は公開を意味せず、公開用には必要な集計・版・条件・hashを抽出する。公開しない生データの保存先をGit管理下に強制しない。

「動いた」ではなく、**何が、どの条件で、どの程度確認できたか**を書く。

## G.3 本書の関連ファイルと教材例

```text
mastering-kaspa/
├── examples/
│   ├── dag-relations/
│   ├── minimal-node/
│   ├── sdk-monitor/
│   └── payment-candidates/
└── experiments/
    ├── ghostdag/
    ├── daa/
    ├── daa-window/
    ├── daa-transition/
    ├── devnet/
    └── *-2026-10-31.md 等の検証記録
```

第35章の入金候補サンプルは `examples/payment-candidates/` に収録している。SDKの現在UTXOをRustへ取り込む手順は[サンプルREADME](https://github.com/shiga0/mastering-kaspa/blob/main/examples/payment-candidates/README.md)を参照する。完成した支払い監視アプリ全体を収録したものではない。

本付録のJSON/manifest/結果テンプレートは保存形式の提案で、experiments/vectors、experiments/rust、experiments/resultsやexperiments.ymlを本書に追加したものではない。G.6の独自教材も本書外の一時プロジェクトで確認した。ログやDB・鍵を含む新しいフォルダーを公開用に作らない。

## G.4 実験IDとステータス

第27章の最小ノードについては、2026年10月31日に6件の単体テスト・書式検査・Clippy・実行結果を確認した。収録コードは `examples/minimal-node/`、検証条件と結果は `experiments/minimal-node-2026-10-31.md` に記録している。以下のG-001〜G-010とは別の試験である。

2026-11-01の目次順レビューでは改訂後の16テストと実行を確認した。最新記録は `experiments/chapter-27-review-2026-11-01.md`。関数呼び出しによる伝達試験は実P2Pの検証ではなく、実験IDの完了へ読み替えない。

| ID | テーマ | 種別 | ステータス |
|---|---|---|---|
| G-001 | ハッシュ関数の既知値 | テストベクトル | PASS：今回、空入力/abcの2値を確認 |
| G-002 | UTXOの保存則 | 教育用の固定ケース | PASS：今回のG.6教材の範囲 |
| G-003 | 二重消費の拒否 | 教育用の固定ケース | PASS：入力重複と再消費等。G.6の範囲 |
| G-004 | DAGのPast・Anticone | 教育用モデル | PASS：今回、G.9のJSON固定DAGを計算 |
| G-005 | GHOSTDAG分類 | 参照実装比較 | **実行済み：24ケースで一致（G.10の範囲）** |
| G-006 | 難易度調整 | 参照実装比較 | **算術27・Full窓120・Sampled/Dual288ケース一致（G.11の範囲）** |
| G-007 | Devnetのノード同期 | 統合実験 | 限定PASS：既存2ノード伝播/停止復帰。Pruning後IBD等は未実施 |
| G-008 | RPC・wRPC再接続 | 障害注入 | 限定PASS：既存購読終了/再購読と現在UTXO補完。全履歴再生ではない |
| G-009 | トランザクション通知の重複 | アプリケーションテスト | 限定PASS：既存実通知のアプリ側二重適用/SQLite照合 |
| G-010 | Pruning・再起動 | 統合実験 | INCONCLUSIVE：前提検査でpoint未進行。通常再起動のみ別途実測 |

個々の実行ステータスは `NOT_RUN / RUNNING / PASS / FAIL / BLOCKED / INCONCLUSIVE` から選ぶ。上表の「限定PASS」は記録ごとの条件付き成功の要約で、実験テーマ全体の完了を表さない。**FAILを削除しない**。再試験は別の実行IDで記録する。

## G.5 テストベクトルの設計原則

テストベクトルは以下を満たすことが望ましい。

- 入力と期待出力を機械的に読み取れる。
- 値の単位、バイト順、文字エンコーディングを明記する。
- 正常系だけでなく境界値、異常系、反例を含む。
- 期待値の出典（仕様、論文、実装コミット、手計算）を記録する。
- 将来のプロトコル変更時にも古い期待値を上書きしない。

JSONの例：

```json
{
  "id": "G-001-sha256-empty",
  "algorithm": "SHA-256",
  "input_encoding": "utf8",
  "input": "",
  "expected_hex": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "source": "SHA-256 known-answer test",
  "status": "NOT_RUN"
}
```

上のstatusは未実行テンプレートの初期値。今回の実測結果はG.7と検証記録へ分けて保存する。これは**一般的なSHA-256の既知値**であり、KaspaのPoWハッシュそのものを検証するベクトルではない。

## G.6 Rustで再現できる最小テスト

以下はKaspaのコンセンサス実装ではなく、**UTXO台帳の保存則と二重消費防止を理解するための教育用コード**である。Rust標準ライブラリだけを使用する。

本書外でcargo +1.90.0 new --lib --edition 2021 utxo-lab等により作る独立プロジェクトの `Cargo.toml`：

```toml
[package]
name = "utxo-lab"
version = "0.1.0"
edition = "2021"
```

そのプロジェクトの `src/lib.rs`：

<details class="code-details">
<summary>UTXOモデルの実装とテストを見る</summary>

```rust
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub tx_id: u64,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Output {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    EmptyInputs,
    DuplicateInput,
    MissingInput,
    ValueOverflow,
    OutputsExceedInputs,
    OutputCollision,
}

pub fn apply_transaction(
    utxos: &mut HashMap<Outpoint, Output>,
    inputs: &[Outpoint],
    new_tx_id: u64,
    outputs: &[Output],
) -> Result<u64, ApplyError> {
    if inputs.is_empty() {
        return Err(ApplyError::EmptyInputs);
    }
    let mut seen = HashSet::new();
    let mut input_total = 0u64;
    for input in inputs {
        if !seen.insert(*input) {
            return Err(ApplyError::DuplicateInput);
        }
        let value = utxos.get(input).ok_or(ApplyError::MissingInput)?.value;
        input_total = input_total
            .checked_add(value)
            .ok_or(ApplyError::ValueOverflow)?;
    }
    let mut output_total = 0u64;
    for (index, output) in outputs.iter().enumerate() {
        output_total = output_total
            .checked_add(output.value)
            .ok_or(ApplyError::ValueOverflow)?;
        let index = u32::try_from(index).map_err(|_| ApplyError::ValueOverflow)?;
        if utxos.contains_key(&Outpoint {
            tx_id: new_tx_id,
            index,
        }) {
            return Err(ApplyError::OutputCollision);
        }
    }
    if output_total > input_total {
        return Err(ApplyError::OutputsExceedInputs);
    }
    // すべての検証が完了した後で状態を更新する。
    for input in inputs {
        utxos.remove(input);
    }
    for (index, output) in outputs.iter().enumerate() {
        utxos.insert(
            Outpoint {
                tx_id: new_tx_id,
                index: index as u32,
            },
            *output,
        );
    }
    Ok(input_total - output_total) // 教育用の「手数料」モデル
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial() -> (HashMap<Outpoint, Output>, Outpoint) {
        let op = Outpoint { tx_id: 1, index: 0 };
        (HashMap::from([(op, Output { value: 100 })]), op)
    }

    #[test]
    fn preserves_value_minus_fee() {
        let (mut utxos, input) = initial();
        let fee = apply_transaction(
            &mut utxos,
            &[input],
            2,
            &[Output { value: 60 }, Output { value: 35 }],
        )
        .unwrap();
        assert_eq!(fee, 5);
        assert_eq!(utxos.values().map(|v| v.value).sum::<u64>(), 95);
    }

    #[test]
    fn rejects_duplicate_input_without_mutation() {
        let (mut utxos, input) = initial();
        let before = utxos.clone();
        let err = apply_transaction(&mut utxos, &[input, input], 2, &[Output { value: 100 }])
            .unwrap_err();
        assert_eq!(err, ApplyError::DuplicateInput);
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_overspending_without_mutation() {
        let (mut utxos, input) = initial();
        let before = utxos.clone();
        let err = apply_transaction(&mut utxos, &[input], 2, &[Output { value: 101 }]).unwrap_err();
        assert_eq!(err, ApplyError::OutputsExceedInputs);
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_missing_input() {
        let (mut utxos, _) = initial();
        let before = utxos.clone();
        let missing = Outpoint {
            tx_id: 99,
            index: 0,
        };
        assert_eq!(
            apply_transaction(&mut utxos, &[missing], 2, &[]),
            Err(ApplyError::MissingInput)
        );
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_spending_consumed_output() {
        let (mut utxos, input) = initial();
        apply_transaction(&mut utxos, &[input], 2, &[Output { value: 100 }]).unwrap();
        let before = utxos.clone();
        assert_eq!(
            apply_transaction(&mut utxos, &[input], 3, &[]),
            Err(ApplyError::MissingInput)
        );
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_output_collision_without_mutation() {
        let (mut utxos, input) = initial();
        let before = utxos.clone();
        assert_eq!(
            apply_transaction(&mut utxos, &[input], 1, &[Output { value: 100 }]),
            Err(ApplyError::OutputCollision)
        );
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_empty_inputs_without_mutation() {
        let (mut utxos, _) = initial();
        let before = utxos.clone();
        assert_eq!(
            apply_transaction(&mut utxos, &[], 2, &[]),
            Err(ApplyError::EmptyInputs)
        );
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_totals_overflow_without_mutation() {
        let (mut utxos, input) = initial();
        let other = Outpoint { tx_id: 9, index: 0 };
        utxos.insert(other, Output { value: u64::MAX });
        let before = utxos.clone();
        assert_eq!(
            apply_transaction(&mut utxos, &[input, other], 2, &[]),
            Err(ApplyError::ValueOverflow)
        );
        assert_eq!(utxos, before);
        assert_eq!(
            apply_transaction(
                &mut utxos,
                &[input],
                2,
                &[Output { value: u64::MAX }, Output { value: 1 }]
            ),
            Err(ApplyError::ValueOverflow)
        );
        assert_eq!(utxos, before);
    }
}
```

</details>

実行コマンド：

```bash
cd utxo-lab
cargo +1.90.0 test --offline
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 clippy --offline --all-targets -- -D warnings
```

**確認結果：** 既存Rust 1.90.0 / edition 2021で8テスト・書式/Clippyが成功。新規toolchain導入はしていない。これは固定した入力例の確認で、全入力に対する性質の証明やfuzzingではない。また、このモデルは署名検証、Script、Mass、Coinbase、DAGの順序付けを実装していない。tx_idは簡略な整数で、呼出者が一意に決める前提。衝突検査は現在のUTXOに対するもので、消費済みIDの全履歴を保存・検証するものではない。

## G.7 G-001：ハッシュ関数の既知値

**目的：** ハッシュ値のバイト列と16進文字列表現を混同せず、固定入力から同じ出力が得られることを確認する。

**手順：**

```bash
printf '' | shasum -a 256
printf 'abc' | shasum -a 256
```

**期待値：**

| 入力 | SHA-256の期待値 |
|---|---|
| 空文字列 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `abc` | `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` |

**今回の結果：** macOSのshasum -a 256とPython標準hashlibで、改行を付けない空入力/abcの2値が一致。実行条件は本付録のレビュー記録へ掲載。Kaspa固有ハッシュの検証ではない。

## G.8 G-002〜003：UTXO保存則と二重消費

教育用モデルで検証する不変条件は以下の通り。

\[
\sum \text{input values} = \sum \text{output values} + \text{fee}
\]

ただしこれは通常の価値保存モデルの説明であり、実際のKaspaのトランザクション有効性は署名、Script、Massなど追加条件を含む。

| ケース | 入力 | 出力 | 期待結果 |
|---|---|---|---|
| 正常な移転 | 100 | 60, 35 | 成功・差額5 |
| 過剰支出 | 100 | 101 | 拒否 |
| 同じOutpointを二度参照 | 100を2回 | 100 | 拒否 |
| 存在しないOutpoint | 不明 | 0 | 拒否 |

失敗時には台帳が**一切変更されないこと**も確認する。

## G.9 G-004：DAGのPast・Future・Anticone

次の教育用DAGを用いる。矢印は**新しいブロックから参照する親への向き**とする。

```text
D → B → A
D → C → A
E → C
```

曖昧な図の解釈を避けるため、辺を機械可読な形式でも固定する。

```json
{
  "edges_child_to_parent": {
    "A": [],
    "B": ["A"],
    "C": ["A"],
    "D": ["B", "C"],
    "E": ["C"]
  }
}
```

この定義では、`Past(D) = {A, B, C}`、`Past(E) = {A, C}`、`D` と `E` は互いに祖先でも子孫でもないため、互いのAnticoneに属する。Pastに自身を含めるかは定義上明示し、ここでは**含めない**。

**今回の確認：** このJSONをPython集合モデルで読み、親の存在・閉路なし・Past(D/E)・Anticone(D)={E}を確認。Rust祖先例の実行は[付録C](appendix-c.md)の既存記録を参照し、今回再実行したとは扱わない。

## G.10 G-005：GHOSTDAGの分類比較

GHOSTDAGは単純な「DAGのトポロジカルソート」ではない。比較には少なくとも次を固定する。

- ブロックIDと親集合。
- 適用する `k` とネットワーク・プロトコル条件。
- 同点処理・選択規則を含む参照実装のコミット。
- Selected Parent、blue/red分類、blue score、blue workなどの期待値。

**手順：** 小規模DAGを生成 → 参照実装に与える → 出力をJSON化 → 教育用実装と差分比較 → 不一致時に最小反例を保存する。

**実行結果（2026-10-31）：** 基準版v1.0.1の `GhostdagManager::ghostdag` と全blue集合のanticoneを直接数えるモデルを、24ケース・各親配列順で712ブロック比較し、selected parent・順序付きblue/red・score/workが一致した。親配列の通常順と反転順の出力も一致。収録コードは `experiments/ghostdag/`、公開集計は `experiments/ghostdag/observed-2026-10-31.json`、条件と経緯は [ghostdag-comparisonの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/ghostdag-comparison-2026-10-31.md) を参照する。

固定k、ハッシュ同点処理、異なるbits、フォーク境界、上位レベルのwork下限を含む。到達可能性は共通DFSアダプター、ヘッダーは合成値であり、公式Reachability・PoW・DAA計算・ノード全体の検証ではない。任意のDAGや登録順を網羅した結果として扱わない。

## G.11 G-006：難易度調整の比較

難易度調整の実験では、単なる「時間平均の逆数」ではなく、**対象バージョンのKaspaコンセンサス規則**を確認する。

| 項目 | 固定する条件 |
|---|---|
| 入力 | 過去ブロックのタイムスタンプ、DAA関連データ |
| 出力 | 対象ブロックの難易度関連値 |
| 参照 | `rusty-kaspa`の特定コミット |
| 変化点 | ハードフォーク・パラメータ変更の境界 |
| 異常系 | 同一時刻、極端な時刻差、境界値 |

**実行結果（2026-10-31）：** 基準版v1.0.1のFull/Sampled DAA計算器に明示した窓を与え、27ケース・各2通りの挿入順でPython整数モデルとbitsが一致した。入力と期待値は `experiments/daa/vectors.json`、結果は `experiments/daa/observed-2026-10-31.json`、条件は [daa-comparisonの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/daa-comparison-2026-10-31.md) を参照する。履歴不足・同一時刻・整数丸め・target制限・Sampledの標本不足時の分岐を含む。別検査 `experiments/daa-window/` ではFull窓・non-DAA集合・DAA scoreを120ケースで比較し一致した。キャッシュ3条件・親配列2順で計11700回の公式ブロック比較を行った。GHOSTDAGメタデータは公式から供給し、到達可能性は共通DFSで提供する。さらに `experiments/daa-transition/` でSampled窓・Dual切り替えを288ケースで比較し、窓・non-DAA・DAA score・bitsが一致した。選択親DAAのactivation直前・一致・直後を含み、キャッシュ3条件・親配列2順でDualと直接計算器の出力も一致。PoW・全ネットワーク・全sample rate・中央値時刻窓・実際のPruning後IBDは未検証。

## G.12 G-007：Devnetノード同期

**目的：** 二つのノードを起動し、接続と同期の状態を確認する。

**前提：** 使用するDockerイメージ、起動引数、ポート、RPCインターフェースはリリースによって異なる。付録A・Dと対象バージョンのヘルプを照合する。

```bash
# 実験環境の識別情報を保存
uname -s
uname -m
rustc --version
cargo --version
docker --version
docker compose version

git -C rusty-kaspa rev-parse HEAD

# 2ノード専用手順。実行前にREADMEの条件と終了処理を確認する
# python3 experiments/devnet/two-node.py
# base compose.yamlの1ノード起動だけで2ノード試験の代替にしない
```

**測定するもの：** ピア数、同期状態、DAG情報、再起動後の復帰、エラー数。同期時間の測定には起点と終点の定義が必要。

**既存の結果：** [devnet-two-nodeの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-two-node-2026-10-31.md) に、別データ領域の2ノードで伝播・停止復帰後の27ブロック/DAA score・指定1アドレスの26 UTXOが一致した記録がある。イメージdigest/ARM64・構成は固定済み。「未固定によるBLOCKED」は現状と合わないため撤回する。全UTXOやPruning後IBD・同期時間ベンチマークは未検証。今回は記録の照合のみ。

## G.13 G-008：RPC・wRPC再接続

**目的：** 一時切断後にイベントを取りこぼさないアプリケーション設計を確認する。

1. チェーン監視を開始し、観測位置（カーソル相当）を保存する。
2. 接続を意図的に切断する。
3. 切断中にテスト用イベントを発生させる。
4. 再接続後、RPCによる照合と不足分の補完を実施する。
5. イベント重複、欠落、順序の差を記録する。

**注意：** wRPC通知だけで完全な履歴再生が保証されると仮定しない。必要な照合方法は実装とAPI仕様に依存する。

**既存の結果：** [devnet-live-reconciliationの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-live-reconciliation-2026-10-31.md) で、購読の終了/再購読・現在UTXOの再照会・古い応答の拒否を確認した記録がある。単一アドレス/単一書込主体の限定試験で、切断中の全イベント履歴や自動再接続サービスの継続運用を検証したものではない。今回は再実行していない。

## G.14 G-009：重複通知と冪等性

支払い監視アプリの例では、同じイベントを複数回受け取っても入金額が二重計上されないことを検証する。

```text
通知A ─┐
通知A ─┼─> イベント正規化 ─> 重複判定 ─> 台帳反映（最大1回）
通知B ─┘
```

検証対象は「イベントID」だけではない。トランザクションID、出力インデックス、チェーン状態、再編成の扱いも設計する。データベースの一意制約とトランザクション境界を併用する。added→removed→addedを永久的なoutpoint/kind重複排除で潰さない。現在状態への冪等置換とイベント履歴の同一性を区別する。

**既存の結果：** [devnet-notificationsの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-notifications-2026-10-31.md) で、実際に受けた通知をアプリ側で2回適用し、SQLiteの件数/金額が増えないことを確認した記録がある。ネットワークが同じ通知を2回送った試験ではない。再編成のremoved観測は別の [devnet-reorg-observerの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-reorg-observer-2026-10-31.md) を参照。会計取消や複数ワーカーの試験まで完了したとしない。

## G.15 G-010：Pruning・再起動

**目的：** 再起動や保存データの整理後に、ノードとアプリケーションが整合性を回復できるかを確認する。

**観測項目：** 起動時間、ディスク使用量、同期状態、RPC応答、保持データの範囲、復旧に必要な手順。Pruningの挙動はコンセンサス・ストレージ設定・ノードバージョンに依存する。

**既存の結果：** 通常のプロセス再起動の保持は [devnet-environmentの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-environment-2026-10-31.md) に記録。一方、[devnet-pruning-reviewの検証記録](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet-pruning-review-2026-10-31.md) の前提検査はPruning pointがGenesisのままで終了コード2。実削減・削減後再起動/新規ノードIBDは未実施で、Pruning成功として扱わない。今回はこれらの記録を照合しただけである。

### G.15.1 G-005〜010を再現する入口

本書リポジトリを取得し、各READMEの対象版・依存・起動条件・終了処理を確認してから実行する。結果の記録だけを読む場合と、自分で再実行した場合を分ける。

| 実験 | 実行手順 |
|---|---|
| G-005：公式GHOSTDAGとの比較 | [GHOSTDAG比較README](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/ghostdag/README.md) |
| G-006：DAA算術・窓・切り替え | [算術](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/daa/README.md)・[Full窓](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/daa-window/README.md)・[Sampled/Dual切り替え](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/daa-transition/README.md) |
| G-007：2ノードの伝播・復帰 | [2ノードの伝播と停止復帰](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#2ノードの伝播と停止復帰) |
| G-008：再購読と現在UTXOの補完 | [稼働中の再照会と再購読](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#稼働中の再照会と再購読) |
| G-009：通知の二重適用 | [通知欠落・重複とSQLiteの再読込](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#通知欠落重複とsqliteの再読込) |
| G-010：通常再起動とPruningの前提検査 | [短時間採掘と再起動後の状態比較](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#短時間採掘と再起動後の状態比較)・[Pruningの前提検査](https://github.com/shiga0/mastering-kaspa/blob/main/experiments/devnet/README.md#pruningの前提検査未達を成功扱いにしない) |

G-010のリンク先は通常再起動と前提検査の手順であり、Pruning後IBDの完成した実習ではない。各実験の確認範囲はG.10〜15と検証記録に従う。

## G.16 実験メタデータのテンプレート

本書には追加しない保存形式の例（生ログの公開を必須にしない）：

<details class="code-details">
<summary>実験メタデータのYAMLテンプレートを見る</summary>

```yaml
experiment_id: G-001
run_id: G-001-YYYYMMDD-001
status: NOT_RUN
purpose: "SHA-256の既知値を確認する"
hypothesis: "固定入力は期待値と一致する"
started_at_utc: null
finished_at_utc: null
operator: null
environment:
  os: null
  architecture: null
  rust_version: null
  docker_version: null
  node_version: null
source:
  repository: null
  commit_sha: null
  working_tree_clean: null
network:
  name: null
  config_hash: null
inputs:
  - "empty UTF-8 string"
expected:
  sha256_hex: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
observed: null
artifacts:
  public_summary: null
  private_raw_log_digest: null
conclusion: null
limitations: []
```

</details>

`NOT_RUN`を`PASS`に変えるには、実測値と証拠を添える。

## G.17 結果レポートのテンプレート

<details class="code-details">
<summary>結果レポートのテンプレートを見る</summary>

```markdown
# G-XXX 実験名

- Run ID:
- 実施日時（UTC）:
- ステータス:
- 参照仕様:
- コミットSHA:
- 環境:

## 仮説

## 実験手順

## 入力と期待結果

## 実測結果

## 期待値との差分

## 考察・代替仮説

## 制約・未確認事項

## 再現コマンド

## 添付ログ・ハッシュ
```

</details>

## G.18 実験結果の信頼度

| レベル | 内容 | 記載例 |
|---|---|---|
| L0 | 実行していない | 未実行 |
| L1 | 手元で一度成功 | ローカル単発PASS |
| L2 | 複数回成功、環境固定 | 再実行PASS |
| L3 | CIで自動検証 | CI PASS |
| L4 | 独立環境・別実装で検証 | クロス検証PASS |

L0〜L4は本書の整理用ラベルで、公式の認証基準ではない。CIであるだけで独立性や検査範囲が増えるわけではなく、実行回数・環境・独立性は別軸で記録する。**L4でも数学的な完全証明ではない**。また、ネットワークの安全性を単一のテストで保証できるわけではない。

## G.19 差分テストと失敗の最小化

参照実装との比較で差異が見つかった場合、すぐに「参照実装のバグ」と結論づけない。入力条件、バージョン、パラメータ、符号化、丸め、並び順を確認する。

```text
差分発見
  ↓
入力とバージョンの固定
  ↓
再現性確認
  ↓
最小反例へ縮小
  ↓
仕様・実装・テストの照合
  ↓
Issue / PR / 文書修正
```

失敗ケースを回帰テストとして残すと、将来の変更時に同じ問題を検出できる。

## G.20 プロパティベーステストとファジング

固定ベクトルでは検証できない入力空間を調べるため、プロパティベーステストやファジングを利用する。

- UTXOの入力集合に重複があれば拒否する。
- 入力合計・出力合計の計算でオーバーフローしない。
- 不正入力による失敗で状態が部分更新されない。
- DAGの祖先関係に閉路がない。
- シリアライズ→デシリアライズで情報が失われない。

実際のKaspaコンセンサスでは、上記に加えてプロトコル固有の不変条件を確認する。ランダムテストでは乱数シードを保存する。

## G.21 性能測定の落とし穴

- DebugビルドとReleaseビルドを混ぜない。
- ウォームアップ、試行回数、中央値、分散を記録する。
- CPU負荷、温度、ディスクI/O、ネットワーク状態を記録する。
- DAGサイズ、UTXO数、入力データの分布を固定する。
- 速い実装が正しい実装とは限らない。

性能比較では**正しさの検証を先に**行う。

## G.22 セキュリティと秘密情報

実験は原則としてDevnet・隔離環境で行う。Mainnetの資金を使った試験は避け、秘密鍵をGitにコミットしない。RPCポートを不用意にインターネットへ公開しない。ログの公開前にシークレットやIPアドレスなどを確認する。

## G.23 CIによる継続的検証

次は独立した教育用Rustプロジェクト向けの参考例。本書にはexperiments/rustやこのworkflowを追加していない。実際の本書CIは.github/workflows/deploy.ymlで、付録教材すべてを自動検査するものではない。

`.github/workflows/experiments.yml`：

<details class="code-details">
<summary>CI設定の例を見る</summary>

```yaml
name: Experiments
permissions:
  contents: read
on:
  push:
    paths:
      - 'experiments/rust/**'
      - '.github/workflows/experiments.yml'
  pull_request:
    paths:
      - 'experiments/rust/**'
      - '.github/workflows/experiments.yml'
jobs:
  test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: experiments/rust
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.90.0
      - run: cargo test --locked
```

</details>

`cargo test --locked`には`Cargo.lock`が必要である。依存関係の固定とActionのSHAピン留めは、公開時のセキュリティレビューで検討する。

## G.24 C4モデル：実験基盤の構成

```text
[著者・レビュー担当者]
        │
        ▼
[実験ランナー] ─────> [テストベクトル]
        │
        ├───────────> [教育用Rust実装]
        │
        ├───────────> [参照rusty-kaspa]
        │
        └───────────> [Devnetノード]
        │
        ▼
[結果・ログ・環境情報]
        │
        ▼
[比較レポート] ─────> [書籍の主張]
```

依存関係を明確にすることで、ノードの更新に伴う実験結果の変化を追跡しやすくする。

## G.25 実験結果と本文の対応表

| 本書のテーマ | 実験ID | 何を確認するか |
|---|---|---|
| 暗号技術の基礎（第3章） | G-001 | ハッシュの既知値 |
| BlockDAG（第7章） | G-004 | Past・Anticone |
| GHOSTDAG（第12章） | G-005 | 分類・順序 |
| DAA（第13章） | G-006 | 難易度の参照実装一致 |
| UTXO（第15章） | G-002, G-003 | 保存則・二重消費拒否 |
| RPC・wRPC（第22章） | G-008 | 再接続・取りこぼし対策 |
| Devnet（第28章） | G-007, G-010 | 同期・復旧 |
| アプリケーション（第35章） | G-009 | 通知の冪等性 |

## G.26 公開前のレビュー基準

**本文で「確認した」と書くための条件：**

1. 実行日時とコミットSHAが残っている。
2. 再現コマンドがあり、依存バージョンが明記されている。
3. 期待値の出典が説明できる。
4. 実測値と必要な実行証拠が残り、公開用集計と非公開の生データを区別している。
5. 失敗と未実行を隠していない。
6. 仕様由来の事実と教育用モデルを区別している。
7. 参照実装との比較結果を過大解釈していない。

## G.27 実践課題

<details class="exercise-details">
<summary>実践課題を見る</summary>

### 課題1

G-001を実行し、SHA-256の既知値を保存する。

### 課題2

G-002〜003のRustコードを実行し、`cargo test`ログを保存する。

### 課題3

G-004のDAGをRustで表現し、祖先集合を求める。

### 課題4

`rusty-kaspa`のコミットを固定し、G-005の参照テストを一つ特定する。

### 課題5

Devnetの起動構成を検証し、G-007の未確認範囲・完了基準を定義する。

### 課題6

失敗ケースを一つ作り、最小反例と回帰テストを記録する。

</details>

## G.28 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. テストベクトルと実験結果はどう違うか。
2. なぜGitのブランチ名ではなくコミットSHAを記録するのか。
3. 参照実装と一致しただけでは何が保証されないか。
4. 失敗時の状態不変性を確認する理由は何か。
5. RPC通知を受け取ったことと支払い確定はどう違うか。
6. 性能測定で入力データの分布を固定する理由は何か。
7. 「未実行」と「PASS」を混同すると何が問題になるか。

**解答例**

1. テストベクトルは入力と期待結果、実験結果は実際に実行して観測した値・成否である。期待値を書いただけで実行済みとは扱わない。
2. ブランチは後の更新で参照先が変わるため。固定SHAと環境・設定を残すことで、同じ対象を再調査できる。
3. 一致は比較した入力・条件での結果であり、参照実装自身の無誤り、任意入力での同値性、Mainnetでの安全性や性能までは保証しない。
4. 不正入力や途中の失敗で部分適用・二重適用・資金状態の破壊が起きないことを確認するため。実装が定める更新・保存の契約に合わせて検査する。
5. 通知は特定ノードでの観測であり、合意上の受理や業務上の確認ポリシーの充足とは別である。欠落・重複・再編成も考慮する。
6. 入力数・形状・偏り等で負荷が変わるため。分布や乱数シードを記録し、実装差と入力差を切り分けて比較する。
7. 未検証の内容を成功の根拠として利用してしまい、再現性と判断の信頼性を失う。実行の有無、期待値、実測値、確認範囲を分けて記録する。

</details>

## G.29 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

Kaspaの研究論文、KIP、Rust実装、Devnetを結びつけるのは、**再現可能な証拠**である。本付録の表は実行済み記録の索引と未実施部分の計画を分けて示す。今後、実験を一つずつ実行してログ・ベクトル・コミットを固定することで、『Mastering Kaspa』の各章を検証可能な技術文書に近づけられる。

</details>

## G.30 一次資料・関連資料

<details class="reference-details">
<summary>参考資料を見る</summary>

- [Kaspa公式サイト](https://kaspa.org/)
- [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/book/)
- [Cargoのテスト機能](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [GitHub Actions公式ドキュメント](https://docs.github.com/en/actions)

</details>

次は[付録H — 参考文献・変更履歴](appendix-h.md)。

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-appendix-g)

</details>
