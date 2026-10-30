# Appendix G — 実験結果・テストベクトル

> **Mastering Kaspa / 付録G**  
> 原稿ステータス：v0.1（技術レビュー前）  
> 対象：Rustによる教育用実装、Kaspa Devnet、コンセンサス関連の再現実験

本付録では、Kaspaの仕様を「理解したつもり」で終わらせず、**入力・期待値・実測値・実行環境を第三者が追跡できる形で残す**ための方法を示す。ここに掲載する値は、明記した教育用例を除き**未測定**である。実際に実行していない試験を「成功」と記録してはならない。

## G.1 目的と適用範囲

実験結果（experimental result）は、特定の環境で実際に観測した結果である。一方、テストベクトル（test vector）は、入力と期待出力を固定した検証用データである。この二つを分けて管理する。

| 種別 | 記録するもの | 例 |
|---|---|---|
| 仕様由来ベクトル | 仕様から導かれる入力・期待出力 | ハッシュの既知値 |
| 実装由来ベクトル | 参照実装で得た出力 | 特定コミットの関数出力 |
| 実験結果 | 実測値・環境・ログ | Devnet同期時間 |
| 性質テスト | 入力全体に期待する条件 | UTXOの二重消費を拒否 |
| 差分テスト | 二つの実装の比較結果 | 教育用実装と参照実装 |

**注意：** 参照実装の出力は必ずしも独立した正しさの証明ではない。仕様との一致、実装間一致、ネットワークでの受理はそれぞれ別の検証対象となる。

## G.2 再現性の最低条件

実験には次の情報を添付する。

1. 実験ID、目的、仮説、判定基準。
2. 使用したOS・CPU・アーキテクチャ・Rust・Dockerのバージョン。
3. `rusty-kaspa`、SDK、実験コードの**GitコミットSHA**。
4. ネットワーク種別（Mainnet / Testnet / Devnet）、設定ファイル、起動引数。
5. 入力データ、期待出力、実測出力、標準出力・標準エラー。
6. 実行手順、所要時間、繰り返し回数、失敗の扱い。
7. 日時、実施者、結論、未解決事項。

「動いた」ではなく、**何が、どの条件で、どの程度確認できたか**を書く。

## G.3 推奨ディレクトリ構成

```text
mastering-kaspa/
├── src/appendices/appendix-g.md
├── experiments/
│   ├── README.md
│   ├── manifest.yaml
│   ├── vectors/
│   │   ├── sha256.json
│   │   ├── utxo.json
│   │   └── dag.json
│   ├── rust/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── devnet/
│   │   └── compose.yaml
│   └── results/
│       └── G-001/
│           ├── metadata.yaml
│           ├── stdout.txt
│           ├── stderr.txt
│           └── report.md
└── examples/
```

ログに秘密鍵、シードフレーズ、APIトークン、個人情報を含めない。

## G.4 実験IDとステータス

| ID | テーマ | 種別 | ステータス |
|---|---|---|---|
| G-001 | ハッシュ関数の既知値 | テストベクトル | **未実行** |
| G-002 | UTXOの保存則 | 性質テスト | **未実行** |
| G-003 | 二重消費の拒否 | 性質テスト | **未実行** |
| G-004 | DAGのPast・Anticone | 教育用モデル | **未実行** |
| G-005 | GHOSTDAG分類 | 参照実装比較 | **未実行** |
| G-006 | 難易度調整 | 参照実装比較 | **未実行** |
| G-007 | Devnetのノード同期 | 統合実験 | **未実行** |
| G-008 | RPC・wRPC再接続 | 障害注入 | **未実行** |
| G-009 | トランザクション通知の重複 | アプリケーションテスト | **未実行** |
| G-010 | Pruning・再起動 | 統合実験 | **未実行** |

ステータスは `未実行 / 実行中 / PASS / FAIL / BLOCKED / INCONCLUSIVE` から選ぶ。**FAILを削除しない**。再試験は別の実行IDで記録する。

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
  "status": "not-run"
}
```

これは**一般的なSHA-256の既知値**であり、KaspaのPoWハッシュそのものを検証するベクトルではない。

## G.6 Rustで再現できる最小テスト

以下はKaspaのコンセンサス実装ではなく、**UTXO台帳の保存則と二重消費防止を理解するための教育用コード**である。Rust標準ライブラリだけを使用する。

`experiments/rust/Cargo.toml`：

```toml
[package]
name = "mastering-kaspa-experiments"
version = "0.1.0"
edition = "2021"
```

`experiments/rust/src/lib.rs`：

```rust
use std::collections::{HashMap, HashSet};

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
        input_total = input_total.checked_add(value).ok_or(ApplyError::ValueOverflow)?;
    }
    let mut output_total = 0u64;
    for (index, output) in outputs.iter().enumerate() {
        output_total = output_total.checked_add(output.value).ok_or(ApplyError::ValueOverflow)?;
        let index = u32::try_from(index).map_err(|_| ApplyError::ValueOverflow)?;
        if utxos.contains_key(&Outpoint { tx_id: new_tx_id, index }) {
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
        utxos.insert(Outpoint { tx_id: new_tx_id, index: index as u32 }, *output);
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
        ).unwrap();
        assert_eq!(fee, 5);
        assert_eq!(utxos.values().map(|v| v.value).sum::<u64>(), 95);
    }

    #[test]
    fn rejects_double_spend_without_mutation() {
        let (mut utxos, input) = initial();
        let before = utxos.clone();
        let err = apply_transaction(
            &mut utxos, &[input, input], 2, &[Output { value: 100 }],
        ).unwrap_err();
        assert_eq!(err, ApplyError::DuplicateInput);
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_overspending_without_mutation() {
        let (mut utxos, input) = initial();
        let before = utxos.clone();
        let err = apply_transaction(
            &mut utxos, &[input], 2, &[Output { value: 101 }],
        ).unwrap_err();
        assert_eq!(err, ApplyError::OutputsExceedInputs);
        assert_eq!(utxos, before);
    }

    #[test]
    fn rejects_missing_input() {
        let (mut utxos, _) = initial();
        let missing = Outpoint { tx_id: 99, index: 0 };
        assert_eq!(
            apply_transaction(&mut utxos, &[missing], 2, &[]),
            Err(ApplyError::MissingInput)
        );
    }
}
```

実行コマンド：

```bash
cd experiments/rust
cargo test
cargo test -- --nocapture
```

**検証状況：** この原稿の作成時点では実行結果を採録していない。`PASS` と記録するのは実際の実行ログを保存した後に限る。また、このモデルは署名検証、Script、Mass、Coinbase、DAGの順序付けを実装していない。

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

**結果欄：** `未実行`。実測値とコマンド実行環境を追記する。

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
  D     E
  |\   /|
  | \ / |
  B  C  |
   \ | /
     A
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

**検証項目：** 閉路がないこと、祖先探索が期待集合に一致すること、参照方向を逆にしていないこと。

## G.10 G-005：GHOSTDAGの分類比較

GHOSTDAGは単純な「DAGのトポロジカルソート」ではない。比較には少なくとも次を固定する。

- ブロックIDと親集合。
- 適用する `k` とネットワーク・プロトコル条件。
- 同点処理・選択規則を含む参照実装のコミット。
- Selected Parent、Blue/Red分類、Blue Score、Blue Workなどの期待値。

**手順：** 小規模DAGを生成 → 参照実装に与える → 出力をJSON化 → 教育用実装と差分比較 → 不一致時に最小反例を保存する。

**現時点の結果：** `未実行`。ここでは特定DAGのBlue/Red期待値を捏造しない。

## G.11 G-006：難易度調整の比較

難易度調整の実験では、単なる「時間平均の逆数」ではなく、**対象バージョンのKaspaコンセンサス規則**を確認する。

| 項目 | 固定する条件 |
|---|---|
| 入力 | 過去ブロックのタイムスタンプ、DAA関連データ |
| 出力 | 対象ブロックの難易度関連値 |
| 参照 | `rusty-kaspa`の特定コミット |
| 変化点 | ハードフォーク・パラメータ変更の境界 |
| 異常系 | 同一時刻、極端な時刻差、境界値 |

**現時点の結果：** `未実行`。固定ベクトルは参照実装から取得後、出典とともに追加する。

## G.12 G-007：Devnetノード同期

**目的：** 二つのノードを起動し、接続と同期の状態を確認する。

**前提：** 使用するDockerイメージ、起動引数、ポート、RPCインターフェースはリリースによって異なる。付録A・Dと対象バージョンのヘルプを照合する。

```bash
# 実験環境の識別情報を保存
uname -a
rustc --version
cargo --version
docker --version
docker compose version

git -C rusty-kaspa rev-parse HEAD

# 実際の起動設定を確認した後で実行する
# docker compose -f experiments/devnet/compose.yaml up -d
# docker compose -f experiments/devnet/compose.yaml logs --timestamps > devnet.log
```

**測定するもの：** ピア数、同期状態、DAG情報、再起動後の復帰、エラー数。同期時間の測定には起点と終点の定義が必要。

**現時点の結果：** `BLOCKED`（使用するCompose構成とノードバージョン未固定）。

## G.13 G-008：RPC・wRPC再接続

**目的：** 一時切断後にイベントを取りこぼさないアプリケーション設計を確認する。

1. チェーン監視を開始し、観測位置（カーソル相当）を保存する。
2. 接続を意図的に切断する。
3. 切断中にテスト用イベントを発生させる。
4. 再接続後、RPCによる照合と不足分の補完を実施する。
5. イベント重複、欠落、順序の差を記録する。

**注意：** wRPC通知だけで完全な履歴再生が保証されると仮定しない。必要な照合方法は実装とAPI仕様に依存する。

**現時点の結果：** `未実行`。

## G.14 G-009：重複通知と冪等性

支払い監視アプリの例では、同じイベントを複数回受け取っても入金額が二重計上されないことを検証する。

```text
通知A ─┐
通知A ─┼─> イベント正規化 ─> 重複判定 ─> 台帳反映（最大1回）
通知B ─┘
```

検証対象は「イベントID」だけではない。トランザクションID、出力インデックス、チェーン状態、再編成の扱いも設計する。データベースの一意制約とトランザクション境界を併用する。

**現時点の結果：** `未実行`。

## G.15 G-010：Pruning・再起動

**目的：** 再起動や保存データの整理後に、ノードとアプリケーションが整合性を回復できるかを確認する。

**観測項目：** 起動時間、ディスク使用量、同期状態、RPC応答、保持データの範囲、復旧に必要な手順。Pruningの挙動はコンセンサス・ストレージ設定・ノードバージョンに依存する。

**現時点の結果：** `未実行`。

## G.16 実験メタデータのテンプレート

`experiments/results/G-001/metadata.yaml`：

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
  stdout: stdout.txt
  stderr: stderr.txt
conclusion: null
limitations: []
```

`NOT_RUN`を`PASS`に変えるには、実測値と証拠を添える。

## G.17 結果レポートのテンプレート

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

## G.18 実験結果の信頼度

| レベル | 内容 | 記載例 |
|---|---|---|
| L0 | 実行していない | 未実行 |
| L1 | 手元で一度成功 | ローカル単発PASS |
| L2 | 複数回成功、環境固定 | 再実行PASS |
| L3 | CIで自動検証 | CI PASS |
| L4 | 独立環境・別実装で検証 | クロス検証PASS |

**L4でも数学的な完全証明ではない**。また、ネットワークの安全性を単一のテストで保証できるわけではない。

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

次は教育用Rustテストを実行するGitHub Actionsの例。

`.github/workflows/experiments.yml`：

```yaml
name: Experiments
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
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --locked
```

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
4. 実測ログが残っている。
5. 失敗と未実行を隠していない。
6. 仕様由来の事実と教育用モデルを区別している。
7. 参照実装との比較結果を過大解釈していない。

## G.27 実践課題

**課題1：** G-001を実行し、SHA-256の既知値を保存する。  
**課題2：** G-002〜003のRustコードを実行し、`cargo test`ログを保存する。  
**課題3：** G-004のDAGをRustで表現し、祖先集合を求める。  
**課題4：** `rusty-kaspa`のコミットを固定し、G-005の参照テストを一つ特定する。  
**課題5：** Devnetの起動構成を検証し、G-007のBLOCKEDを解除する。  
**課題6：** 失敗ケースを一つ作り、最小反例と回帰テストを記録する。

## G.28 理解度チェック

1. テストベクトルと実験結果はどう違うか。
2. なぜGitのブランチ名ではなくコミットSHAを記録するのか。
3. 参照実装と一致しただけでは何が保証されないか。
4. 失敗時の状態不変性を確認する理由は何か。
5. RPC通知を受け取ったことと支払い確定はどう違うか。
6. 性能測定で入力データの分布を固定する理由は何か。
7. 「未実行」と「PASS」を混同すると何が問題になるか。

## G.29 まとめ

Kaspaの研究論文、KIP、Rust実装、Devnetを結びつけるのは、**再現可能な証拠**である。本付録の表は実験の計画表であり、未実施の結果を示すものではない。今後、実験を一つずつ実行してログ・ベクトル・コミットを固定することで、『Mastering Kaspa』の各章を検証可能な技術文書に近づけられる。

## G.30 一次資料・関連資料

- [Kaspa公式サイト](https://kaspa.org/)
- [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals（KIPs）](https://github.com/kaspanet/kips)
- [Kaspa Wiki](https://wiki.kaspa.org/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/book/)
- [Cargoのテスト機能](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [GitHub Actions公式ドキュメント](https://docs.github.com/en/actions)

> **編集メモ：** 出版前に、各実験の結果を実測ログで埋め、参照実装のコミットSHA・ネットワークバージョン・適用KIPを確定すること。
