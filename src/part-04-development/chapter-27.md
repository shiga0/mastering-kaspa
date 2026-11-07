# Chapter 27 — Building a Minimal Kaspa-like Node in Rust（Rustによる最小Kaspa型ノードの構築）

<p class="chapter-part">Part IV — Kaspa Development &amp; Implementation（Kaspaの開発と実装）</p>

> **前提知識：** 第7章（BlockDAG）、第10章（P2P）、第14章（Virtual State）、第15章（UTXO）、第25〜26章（Rust実装）

本章の最小ノードは教育用モデルであり、Kaspa互換ノードではない。

<a id="271-この章で作るもの"></a>

## 27.1 この章で学ぶこと

第26章では公式実装の処理を追い、本章では教育用モデルを自作してコンポーネントを接続する。公式実装の読み取りと教材の動作検証は、別の到達目標として扱う。

ここまでの章では、BlockDAG、トランザクション、コンセンサス、P2P、ストレージを個別に学んだ。本章ではそれらを**小さな実行可能プログラムとして接続する**。実装の目的は、Kaspaと互換なノードを作ることではない。ノードの責務と依存関係を手で追い、失敗時の状態変化をテストできるようにすることだ。

完成する教育用ノードには、次の機能を含める。

- UTXOを消費・生成するトランザクション
- Mempoolへの登録と競合検出
- 複数親を持つブロックの格納
- 親が未到着の場合の拒否と、再送による受理
- ブロックに含まれるトランザクションを順番に適用する簡易状態遷移
- ノード間でブロックを受け渡すインメモリのネットワーク実験
- 失敗時に状態を変更しないことを確認するテスト

> **最重要の境界:** この章の実装は**Kaspaプロトコルではない**。GHOSTDAGのblue/red判定、selected parent、blue work、実際のコンセンサス順序、PoW、署名検証、Difficulty Adjustment、Pruning Proof、KaspaのP2Pワイヤープロトコルは実装しない。ここで使う「Kaspa-like」は、**複数親を持つDAGを題材にした教材**という意味に限る。

## 27.2 なぜ最小ノードを作るのか

実装を読むとき、個々の関数が何をしているかは理解できても、コンポーネント間の依存関係が見えないことがある。そこで、まず小さな系を自分で作る。

```text
              ┌──────────────────────┐
              │      CLI / Tests     │
              └──────────┬───────────┘
                         │
              ┌──────────▼───────────┐
              │         Node         │
              │  (orchestration)     │
              └────┬───────────┬─────┘
                   │           │
          ┌────────▼─────┐ ┌──▼──────────┐
          │   Mempool    │ │  BlockDAG   │
          └──────────────┘ └──┬──────────┘
                              │
                       ┌──────▼──────┐
                       │  UTXO State │
                       └─────────────┘
```

この図は**教育用の依存関係**であり、`rusty-kaspa`の実際のクレート構造ではない。P2P通信と永続化は、後から差し替える境界として扱う。

## 27.3 スコープと非スコープ

| 項目 | 本章の教材 | 実際のKaspaで必要になるもの |
|---|---|---|
| ブロック | ID、親ID、トランザクション一覧 | 正式なヘッダー、PoW、コンセンサス規則 |
| DAG | 複数親、親存在検証 | GHOSTDAG、到達可能性、順序付け |
| トランザクション | 入力・出力・金額 | 正式なシリアライズ、署名、Script |
| 状態 | UTXO集合 | Virtual State、reorg、永続化 |
| Mempool | 入力予約と重複拒否 | 手数料、依存関係、再検証、容量制御 |
| P2P | ノード間の関数呼び出し | ハンドシェイク、接続管理、同期、DoS防御 |
| ストレージ | メモリ | トランザクショナルな永続化と復旧 |

この差分を常に明示することで、教材を本番コードと誤認することを防ぐ。

## 27.4 最初に不変条件を決める

コードを書く前に、守るべき条件を定義する。

1. ブロックIDは一意である。
2. Genesis以外のブロックは、登録済みの親を少なくとも1つ参照する。
3. 親IDの重複や自己参照は認めない。
4. 同じUTXOは、一度の状態遷移で二度消費できない。
5. 出力額の合計は入力額の合計を超えない。
6. 検証に失敗したトランザクション・ブロックは、**既存の状態を変更しない**。
7. Mempool内で同じ入力を複数のトランザクションが予約できない。
8. ノード間で受信したデータは、送信元を信用せず再検証する。

> **注意：** これらは教材独自の条件を含む。Kaspaの実際のコンセンサス規則を列挙したものではない。

## 27.5 プロジェクト構成

本書リポジトリの `examples/minimal-node/` に、27.6〜27.12のコードをまとめた実行可能なサンプルを収録している。リポジトリのルートから次のように実行できる。

```bash
cd examples/minimal-node
cargo run --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

以下は、同じ教材を手元で組み立てる手順である。本書のリポジトリとは別の作業用ディレクトリで実行し、27.6〜27.12のRustコードを順にsrc/main.rsへ配置する。27.13以降の断片は発展設計として分ける。

Rustの標準ライブラリだけで実行できるようにする。

```bash
cargo new --bin --edition 2021 minimal-kaspa-like-node
cd minimal-kaspa-like-node
```

最初は`src/main.rs`にすべて置く。後で責務ごとに分割する。小さい教材では、最初からファイルを増やしすぎないことも重要だ。

```text
minimal-kaspa-like-node/
├── Cargo.toml
└── src/
    └── main.rs
```

<!-- example:minimal-node:start -->

## 27.6 データモデル

トランザクションはUTXOの参照（`OutPoint`）を入力に持ち、金額を持つ新しい出力を作る。ここでは公開鍵やScriptを省略する。

<details class="code-details" open>
<summary>ノードのデータモデルを見る</summary>

```rust
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct OutPoint {
    txid: String,
    index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Output {
    value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Transaction {
    id: String,
    inputs: Vec<OutPoint>,
    outputs: Vec<Output>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Block {
    id: String,
    parents: Vec<String>,
    transactions: Vec<Transaction>,
}
```

</details>

`String`のIDは教材用のラベルであり、暗号学的ハッシュではない。実運用では、ハッシュ計算、シリアライズ、サイズ制限、入力検証が必要になる。

## 27.7 UTXO状態遷移を実装する

状態遷移は**検証してからコミット**する。途中で失敗した場合に、半分だけ更新された状態を残さないためだ。

<details class="code-details" open>
<summary>UTXO状態遷移の実装を見る</summary>

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct UtxoSet {
    entries: HashMap<OutPoint, Output>,
    applied_txids: HashSet<String>,
}

impl UtxoSet {
    fn apply(&mut self, tx: &Transaction) -> Result<(), String> {
        if tx.id.is_empty() {
            return Err("empty transaction id".into());
        }
        if self.applied_txids.contains(&tx.id) {
            return Err("duplicate transaction id".into());
        }
        if tx.inputs.iter().any(|input| input.txid == tx.id) {
            return Err("self-referencing transaction id".into());
        }
        if tx.inputs.is_empty() || tx.outputs.is_empty() {
            return Err("empty inputs or outputs".into());
        }
        let mut seen = HashSet::new();
        let mut input_sum = 0u64;
        for input in &tx.inputs {
            if !seen.insert(input.clone()) {
                return Err("duplicate input".into());
            }
            let prev = self.entries.get(input).ok_or("missing input")?;
            input_sum = input_sum.checked_add(prev.value).ok_or("input overflow")?;
        }
        let mut output_sum = 0u64;
        for output in &tx.outputs {
            output_sum = output_sum
                .checked_add(output.value)
                .ok_or("output overflow")?;
        }
        if output_sum > input_sum {
            return Err("overspend".into());
        }
        let mut created = Vec::with_capacity(tx.outputs.len());
        for (index, output) in tx.outputs.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| "too many outputs")?;
            let point = OutPoint {
                txid: tx.id.clone(),
                index,
            };
            if self.entries.contains_key(&point) {
                return Err("output collision".into());
            }
            created.push((point, output.clone()));
        }
        // ここまで成功してから変更する。ID履歴も同じ候補状態に反映する。
        for input in &tx.inputs {
            self.entries.remove(input);
        }
        self.entries.extend(created);
        self.applied_txids.insert(tx.id.clone());
        Ok(())
    }
}
```

</details>

applied_txidsは、この状態で適用したラベルの履歴である。出力が消費済みでもIDを再利用せず、入力が自分のIDを参照する取引や既存出力への衝突を拒否する。初期残高のseedも使用済みラベルとして登録する。

これは暗号学的なTX IDの計算ではなく、履歴を削らないメモリ内の一意性検査である。署名・Script・成熟・mass等は検証しない。出力indexはu32への変換を検査するが、受信サイズ・メモリ使用量の上限を設けた意味ではない。

### 失敗時の原子性

トランザクションの適用で失敗しても元の状態を保持するには、複数トランザクションを含むブロック単位でも同じ考え方が必要だ。

```rust
fn apply_block_transactions(
    state: &UtxoSet,
    transactions: &[Transaction],
) -> Result<UtxoSet, String> {
    let mut next = state.clone();
    for tx in transactions {
        next.apply(tx)?;
    }
    Ok(next)
}
```

この`clone`方式は単純だが、大きなUTXO集合では高コストになる。実装を発展させる場合は、変更差分（undo log）やストレージトランザクションを検討する。

## 27.8 Mempoolの最小実装

Mempoolは、コンセンサスで確定した状態ではない。未処理のトランザクション候補を管理する領域だ。

<details class="code-details" open>
<summary>Mempoolの入力予約の実装を見る</summary>

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Mempool {
    transactions: HashMap<String, Transaction>,
    reserved_inputs: HashSet<OutPoint>,
}

impl Mempool {
    fn insert(&mut self, tx: Transaction) -> Result<(), String> {
        if tx.id.is_empty() {
            return Err("empty transaction id".into());
        }
        if tx.inputs.iter().any(|input| input.txid == tx.id) {
            return Err("self-referencing transaction id".into());
        }
        if self.transactions.contains_key(&tx.id) {
            return Err("duplicate transaction".into());
        }
        if tx.inputs.is_empty() || tx.outputs.is_empty() {
            return Err("empty transaction".into());
        }
        let mut local = HashSet::new();
        for input in &tx.inputs {
            if !local.insert(input.clone()) || self.reserved_inputs.contains(input) {
                return Err("conflicting input".into());
            }
        }
        for input in &tx.inputs {
            self.reserved_inputs.insert(input.clone());
        }
        self.transactions.insert(tx.id.clone(), tx);
        Ok(())
    }

    fn remove(&mut self, id: &str) {
        if let Some(tx) = self.transactions.remove(id) {
            for input in tx.inputs {
                self.reserved_inputs.remove(&input);
            }
        }
    }
}
```

</details>

これは**競合入力の予約モデル**であり、UTXOの存在や署名を検証していない。親子トランザクション、手数料、期限切れ、Reorganization後の再投入も扱わない。これらを追加する前に、Mempoolの責務を明確にしておく。

## 27.9 BlockDAGの最小実装

複数親を登録できる構造を作る。ただし**親を複数持つことだけではGHOSTDAGにならない**。

<details class="code-details" open>
<summary>BlockDAGの最小実装を見る</summary>

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Dag {
    blocks: HashMap<String, Block>,
}

impl Dag {
    fn insert(&mut self, block: Block) -> Result<(), String> {
        if block.id.is_empty() {
            return Err("empty block id".into());
        }
        if self.blocks.contains_key(&block.id) {
            return Err("duplicate block".into());
        }
        if block.parents.is_empty() && !self.blocks.is_empty() {
            return Err("missing parents".into());
        }
        let mut seen = HashSet::new();
        for parent in &block.parents {
            if parent == &block.id || !seen.insert(parent) {
                return Err("invalid parent reference".into());
            }
            if !self.blocks.contains_key(parent) {
                return Err("unknown parent".into());
            }
        }
        self.blocks.insert(block.id.clone(), block);
        Ok(())
    }
}
```

</details>

ここでは**最初に登録した親なしブロックをGenesisとして扱う**。正式なネットワークではGenesisは任意に決められず、ネットワークパラメータで固定される。

### DAGのイメージ

```text
           G
          / \
         A   B
          \ /
           C
```

`C`が`A`と`B`を親として参照する。これはDAGの構造を示す図であり、ブロックの優先順位やトランザクションの適用順序を示すものではない。

## 27.10 Nodeでコンポーネントを接続する

この教材では、通常のErrでDAG・UTXO・適用済みID・Mempool予約を維持するため、ブロック受理の候補状態を先に作る。これはメモリ内の直列処理の境界であり、[第26章](chapter-26.md)の実ノードのヘッダー・本体・Virtualの別段階を再現したものではない。

<details class="code-details" open>
<summary>Nodeによるコンポーネントの接続を見る</summary>

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Node {
    dag: Dag,
    utxos: UtxoSet,
    mempool: Mempool,
}

impl Node {
    fn accept_block(&mut self, block: Block) -> Result<(), String> {
        // DAGとUTXOを別々に変更すると部分更新の危険がある。
        let mut candidate_dag = Dag {
            blocks: self.dag.blocks.clone(),
        };
        candidate_dag.insert(block.clone())?;
        let candidate_utxos = apply_block_transactions(&self.utxos, &block.transactions)?;
        self.dag = candidate_dag;
        self.utxos = candidate_utxos;
        for tx in &block.transactions {
            self.mempool.remove(&tx.id);
        }
        Ok(())
    }
}
```

</details>

このコードでは、**ブロック受理順にトランザクションを適用する**。そのため、ネットワークの受信順が異なると状態が異なる可能性がある。これは分散コンセンサスとして**成立しない**。実際のKaspaでは、GHOSTDAGとコンセンサス順序、Virtual Stateの仕組みがこの問題を解決する。ここは教材の最大の未実装部分である。

適用失敗でブロック全体を拒否するのも教材独自の規則である。実際のKaspaのブロック妥当性と、各取引のVirtualでの受理を同一視しない。

また、Mempoolからの削除は簡略化している。ブロック受理後に、残存トランザクションの再検証や競合トランザクションの除去が必要になる場合がある。

## 27.11 `main`で動かす

次の`main`関数を、これまでの型と実装の下に追加する。

<details class="code-details" open>
<summary>main関数の実行例を見る</summary>

```rust
fn main() -> Result<(), String> {
    let mut node = Node::default();
    node.accept_block(Block {
        id: "genesis".into(),
        parents: vec![],
        transactions: vec![],
    })?;

    // 教材用の初期残高。実際のcoinbaseではない。
    node.utxos.entries.insert(
        OutPoint {
            txid: "seed".into(),
            index: 0,
        },
        Output { value: 100 },
    );
    node.utxos.applied_txids.insert("seed".into());

    let tx = Transaction {
        id: "payment-1".into(),
        inputs: vec![OutPoint {
            txid: "seed".into(),
            index: 0,
        }],
        outputs: vec![Output { value: 70 }, Output { value: 29 }],
    };
    node.mempool.insert(tx.clone())?;
    node.accept_block(Block {
        id: "block-a".into(),
        parents: vec!["genesis".into()],
        transactions: vec![tx],
    })?;

    println!("blocks: {}", node.dag.blocks.len());
    println!("utxos: {}", node.utxos.entries.len());
    println!("mempool: {}", node.mempool.transactions.len());
    Ok(())
}
```

</details>

実行する。

```bash
cargo run
```

**期待結果：**

```text
blocks: 2
utxos: 2
mempool: 0
```

入力100に対して出力70と29なので、差額1はこの教材では手数料相当として消える。**報酬受取先への配分は実装していない**。

## 27.12 テストで不変条件を確認する

以下を同じファイルに追加する。

<details class="code-details">
<summary>テストコードを見る（16件の単体テスト）</summary>

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_node() -> Node {
        let mut node = Node::default();
        node.accept_block(Block {
            id: "genesis".into(),
            parents: vec![],
            transactions: vec![],
        })
        .unwrap();
        node.utxos.entries.insert(
            OutPoint {
                txid: "seed".into(),
                index: 0,
            },
            Output { value: 100 },
        );
        node.utxos.applied_txids.insert("seed".into());
        node
    }

    fn spend(id: &str, amount: u64) -> Transaction {
        Transaction {
            id: id.into(),
            inputs: vec![OutPoint {
                txid: "seed".into(),
                index: 0,
            }],
            outputs: vec![Output { value: amount }],
        }
    }

    #[test]
    fn valid_block_updates_utxos() {
        let mut node = seeded_node();
        node.accept_block(Block {
            id: "a".into(),
            parents: vec!["genesis".into()],
            transactions: vec![spend("tx1", 99)],
        })
        .unwrap();
        assert!(!node.utxos.entries.contains_key(&OutPoint {
            txid: "seed".into(),
            index: 0
        }));
        assert_eq!(node.utxos.entries.len(), 1);
    }

    #[test]
    fn invalid_block_does_not_mutate_state() {
        let mut node = seeded_node();
        let before = node.clone();
        assert!(node
            .accept_block(Block {
                id: "bad".into(),
                parents: vec!["genesis".into()],
                transactions: vec![spend("tx1", 101)],
            })
            .is_err());
        assert_eq!(node, before);
        assert!(!node.dag.blocks.contains_key("bad"));
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let mut node = seeded_node();
        assert!(node
            .accept_block(Block {
                id: "orphan".into(),
                parents: vec!["missing".into()],
                transactions: vec![],
            })
            .is_err());
    }

    #[test]
    fn multiple_parents_are_allowed() {
        let mut node = seeded_node();
        for id in ["a", "b"] {
            node.accept_block(Block {
                id: id.into(),
                parents: vec!["genesis".into()],
                transactions: vec![],
            })
            .unwrap();
        }
        node.accept_block(Block {
            id: "c".into(),
            parents: vec!["a".into(), "b".into()],
            transactions: vec![],
        })
        .unwrap();
        assert_eq!(node.dag.blocks["c"].parents.len(), 2);
    }

    #[test]
    fn mempool_rejects_conflicting_inputs() {
        let mut pool = Mempool::default();
        pool.insert(spend("tx1", 99)).unwrap();
        assert!(pool.insert(spend("tx2", 98)).is_err());
    }

    #[test]
    fn duplicate_input_is_rejected_without_mutation() {
        let mut state = seeded_node().utxos;
        let before = state.clone();
        let mut tx = spend("tx1", 50);
        tx.inputs.push(tx.inputs[0].clone());
        assert!(state.apply(&tx).is_err());
        assert_eq!(state, before);
    }

    fn block(id: &str, parent: &str, transactions: Vec<Transaction>) -> Block {
        Block {
            id: id.into(),
            parents: vec![parent.into()],
            transactions,
        }
    }
    fn child_spend(id: &str, parent_tx: &str, amount: u64) -> Transaction {
        Transaction {
            id: id.into(),
            inputs: vec![OutPoint {
                txid: parent_tx.into(),
                index: 0,
            }],
            outputs: vec![Output { value: amount }],
        }
    }

    #[test]
    fn later_invalid_transaction_preserves_entire_node() {
        let mut node = seeded_node();
        let first = spend("tx1", 99);
        node.mempool.insert(first.clone()).unwrap();
        let before = node.clone();
        let bad = child_spend("tx2", "tx1", 100);
        assert!(node
            .accept_block(block("bad", "genesis", vec![first, bad]))
            .is_err());
        assert_eq!(node, before);
    }
    #[test]
    fn dependent_transactions_in_one_block_succeed() {
        let mut node = seeded_node();
        node.accept_block(block(
            "a",
            "genesis",
            vec![spend("tx1", 99), child_spend("tx2", "tx1", 98)],
        ))
        .unwrap();
        assert_eq!(node.utxos.entries.len(), 1);
        assert_eq!(
            node.utxos.entries[&OutPoint {
                txid: "tx2".into(),
                index: 0
            }]
                .value,
            98
        );
        assert!(node.utxos.applied_txids.contains("tx1"));
        assert!(node.utxos.applied_txids.contains("tx2"));
    }
    #[test]
    fn transaction_id_cannot_be_reused_after_its_outputs_are_spent() {
        let mut node = seeded_node();
        node.accept_block(block("a", "genesis", vec![spend("tx1", 99)]))
            .unwrap();
        node.accept_block(block("b", "a", vec![child_spend("tx2", "tx1", 98)]))
            .unwrap();
        let before = node.clone();
        assert_eq!(
            node.accept_block(block("c", "b", vec![child_spend("tx1", "tx2", 97)])),
            Err("duplicate transaction id".into())
        );
        assert_eq!(node, before);
    }
    #[test]
    fn transaction_cannot_recreate_its_own_input_label() {
        let mut state = UtxoSet::default();
        state.entries.insert(
            OutPoint {
                txid: "loop".into(),
                index: 0,
            },
            Output { value: 100 },
        );
        let before = state.clone();
        assert_eq!(
            state.apply(&child_spend("loop", "loop", 99)),
            Err("self-referencing transaction id".into())
        );
        assert_eq!(state, before);
    }
    #[test]
    fn output_collision_preserves_all_utxo_state() {
        let mut state = seeded_node().utxos;
        state.entries.insert(
            OutPoint {
                txid: "tx1".into(),
                index: 0,
            },
            Output { value: 7 },
        );
        let before = state.clone();
        assert_eq!(
            state.apply(&spend("tx1", 99)),
            Err("output collision".into())
        );
        assert_eq!(state, before);
    }
    #[test]
    fn arithmetic_overflow_preserves_state() {
        let mut state = seeded_node().utxos;
        state.entries.insert(
            OutPoint {
                txid: "large".into(),
                index: 0,
            },
            Output { value: u64::MAX },
        );
        let before = state.clone();
        let mut tx = spend("tx1", 1);
        tx.inputs.push(OutPoint {
            txid: "large".into(),
            index: 0,
        });
        assert_eq!(state.apply(&tx), Err("input overflow".into()));
        assert_eq!(state, before);
        let mut tx = spend("tx2", 1);
        tx.outputs = vec![Output { value: u64::MAX }, Output { value: 1 }];
        assert_eq!(state.apply(&tx), Err("output overflow".into()));
        assert_eq!(state, before);
    }
    #[test]
    fn failed_mempool_insert_does_not_reserve_earlier_inputs() {
        let mut pool = Mempool::default();
        pool.insert(spend("tx1", 99)).unwrap();
        let before = pool.clone();
        let mut tx = spend("tx2", 98);
        tx.inputs.insert(
            0,
            OutPoint {
                txid: "free".into(),
                index: 0,
            },
        );
        assert!(pool.insert(tx).is_err());
        assert_eq!(pool, before);
    }
    fn relay_block(sender: &Node, receiver: &mut Node, id: &str) -> Result<(), String> {
        let block = sender
            .dag
            .blocks
            .get(id)
            .ok_or("sender does not have block")?
            .clone();
        receiver.accept_block(block)
    }

    #[test]
    fn receiver_retries_child_after_parent_delivery() {
        let mut sender = seeded_node();
        let mut receiver = seeded_node();
        let parent = block("a", "genesis", vec![]);
        let child = block("b", "a", vec![]);
        sender.accept_block(parent).unwrap();
        sender.accept_block(child).unwrap();
        let before = receiver.clone();
        assert!(relay_block(&sender, &mut receiver, "b").is_err());
        assert_eq!(receiver, before);
        relay_block(&sender, &mut receiver, "a").unwrap();
        relay_block(&sender, &mut receiver, "b").unwrap();
        assert_eq!(receiver, sender);
    }
    #[test]
    fn opposite_delivery_orders_diverge_on_conflicting_blocks() {
        let a = block("a", "genesis", vec![spend("tx-a", 99)]);
        let b = block("b", "genesis", vec![spend("tx-b", 98)]);
        let mut x = seeded_node();
        let mut y = seeded_node();
        x.accept_block(a.clone()).unwrap();
        assert!(x.accept_block(b.clone()).is_err());
        y.accept_block(b).unwrap();
        assert!(y.accept_block(a).is_err());
        assert_ne!(x.utxos, y.utxos);
        assert_ne!(x.dag, y.dag);
    }
    #[test]
    fn empty_labels_are_rejected_without_mutation() {
        let mut node = seeded_node();
        let before = node.clone();
        assert!(node.accept_block(block("", "genesis", vec![])).is_err());
        assert!(node
            .accept_block(block("a", "genesis", vec![spend("", 99)]))
            .is_err());
        assert!(node.mempool.insert(spend("", 99)).is_err());
        assert_eq!(node, before);
    }
}
```

</details>

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

2026-11-01、Rust 1.90.0で改訂教材の16テスト・書式・Clippyとmainの期待表示を確認した。ブロック途中の失敗で全Node状態を維持すること、使用済みID・自己参照・出力衝突、算術overflow、Mempoolの部分予約防止、関数呼び出しによる親の再送と受信順の不一致を検査した。記録は `experiments/chapter-27-review-2026-11-01.md`。古い6テストは `experiments/minimal-node-2026-10-31.md` の実施時点の結果である。

<!-- example:minimal-node:end -->

## 27.13 ノード間でブロックを伝播させる

次に、2つの`Node`インスタンスを使ってP2P伝播の**概念**を再現する。実際のTCP通信はまだ行わない。

```rust
fn relay_block(sender: &Node, receiver: &mut Node, id: &str)
    -> Result<(), String>
{
    let block = sender.dag.blocks.get(id)
        .ok_or("sender does not have block")?.clone();
    receiver.accept_block(block)
}
```

収録コードではテスト内のrelay_blockでこの関数呼び出しを再現する。TCP・WebSocket・非同期タスクを使った伝送ではない。

送信側が`genesis → a → b`を保持し、受信側が`genesis`しか持たないとする。`b`だけ先に送れば、親`a`がないため拒否される。`a`を受信してから`b`を再送すれば登録できる。

```text
Node A                      Node B
  │                           │
  │ ───── block b ──────────> │  unknown parent: a
  │                           │
  │ ───── block a ──────────> │  accepted
  │ ───── block b ──────────> │  accepted
  │                           │
```

ここから、**伝播と同期は異なる問題**だとわかる。伝播は新しい情報を届ける処理、同期は欠落した履歴を取得して整合性を回復する処理である。

## 27.14 受信順による不一致を観察する

この教材には意図的な欠点がある。例えば、`a`と`b`が同じUTXOを使う競合トランザクションを含む場合、ノードXが`a`を先に受信し、ノードYが`b`を先に受信すると、採用するトランザクションが異なる。

```text
          Genesis
           /   \
          A     B
       (tx-A) (tx-B)
           \   /
             C
```

| ノード | 受信順 | 簡略実装で起きること |
|---|---|---|
| X | A → B | Aのトランザクションを適用し、Bの競合トランザクションを拒否 |
| Y | B → A | Bのトランザクションを適用し、Aの競合トランザクションを拒否 |

このケースをopposite_delivery_orders_diverge_on_conflicting_blocksで確認した。各ノードは競合側のブロック全体を拒否するため、登録済みDAGも異なる。**同じブロック集合を受信しても、同じ状態になる保証がない。** これが、DAGの保存だけではコンセンサスにならない理由だ。

Kaspaでは、GHOSTDAGに基づくコンセンサス順序とVirtual Stateの計算が重要になる。教材を改良する際は、受信順に即時適用する設計をやめ、**DAGの保存と状態評価を分離**する必要がある。

## 27.15 改良版の設計：DAGと状態を分離する

次のように境界を切り直す。

```text
Incoming Block
      │
      ▼
Structural Validation
      │
      ▼
DAG Storage ────────────────┐
                            │
                            ▼
                  Consensus Ordering
                            │
                            ▼
                    State Evaluation
                            │
                            ▼
                      Virtual State
```

この改良版の`ConsensusOrdering`は、**本章ではインターフェースだけ**を設計する。次の断片は収録バイナリには含めておらず、順序付けの実装・動作検証はしていない。

```rust
trait ConsensusOrdering {
    fn ordered_block_ids(&self, dag: &Dag) -> Result<Vec<String>, String>;
}
```

> **注意：** DAGの一般的なトポロジカルソートは、KaspaのGHOSTDAG順序を再現しない。辞書順ソートなどで仮の順序を作っても、それはKaspaのコンセンサスにはならない。正しいアルゴリズムは第12・14・26章と、対象バージョンの公式実装を参照する。

## 27.16 永続化をどう設計するか

メモリだけのノードは再起動すると状態を失う。永続化では、次の点を考える。

- ブロックの保存とインデックス更新をどの単位でコミットするか
- UTXO変更をどのようにロールバックするか
- 途中でプロセスが落ちたとき、どこから再開するか
- データ破損やスキーマ変更をどう検出するか
- ノードが同期済みと判断する条件をどこに置くか

例えば、DAGだけ永続化してUTXO更新に失敗すると、保存内容と状態が一致しなくなる。ストレージのトランザクション境界は、単なるパフォーマンス上の選択ではなく**正しさの条件**でもある。

## 27.17 P2P通信を追加するときの境界

インメモリ通信を実ネットワークへ置き換えると、以下の問題が現れる。

| 問題 | 追加すべき仕組み |
|---|---|
| 同じブロックを何度も受信 | IDによる重複排除 |
| 親がまだない | 未解決ブロックの一時保管と再取得 |
| 不正な巨大メッセージ | サイズ制限、タイムアウト、検証 |
| 遅いピア | 非同期処理、キュー上限、切断方針 |
| 接続が途切れる | 再接続、欠落データの同期 |
| 悪意あるピア | 入力の不信、リソース制限、監視 |

Rustでは`tokio`などを利用して非同期I/Oを構築できるが、**非同期化してもコンセンサスの正しさは自動的には得られない**。まず同期的なモデルで不変条件を固め、通信をアダプターとして追加する。

## 27.18 ステートマシンでノードを考える

ノード全体を単純化すると、次の状態がある。

```text
Created
   │
   ▼
Loading
   │
   ▼
Syncing ─────────────┐
   │                 │
   ▼                 │
Ready ── network loss┘
   │
   ▼
Stopping
   │
   ▼
Stopped
```

`Ready`であっても、ネットワーク分断や再編成の可能性は消えない。API利用側は「プロセスが動いている」と「同期している」と「要求に使える状態」を分けて判断する必要がある。

## 27.19 DDDとC4による責務整理

この教材の境界をドメインの言葉で説明すると、次のようになる。

| コンポーネント | 責務 | 責務に含めないもの |
|---|---|---|
| `Dag` | ブロック間の参照関係を保存 | トランザクションの支出権限判定 |
| `UtxoSet` | UTXOの消費・生成 | ピア接続管理 |
| `Mempool` | 未処理トランザクション候補と入力予約 | コンセンサス上の確定判断 |
| `Node` | 処理順序と失敗時の整合性 | 個々の暗号アルゴリズム |
| `ConsensusOrdering` | ブロック順序の決定（将来） | TCPの読み書き |

重要なのは、**どのコンポーネントが何を知る必要があるか**を減らすことだ。たとえばMempoolがP2Pソケットの詳細を知る必要はない。

## 27.20 `rusty-kaspa`と対応づける

教材が動いたら、公式実装で同じ責務がどこにあるかを調べる。クレート名やディレクトリは変更されるため、調査時にはコミットを固定する。

```bash
# 第1章の固定版checkoutの親ディレクトリから実行
cd rusty-kaspa-v1.0.1
git rev-parse HEAD
git status --short
KASPA_MINIMAL_REVIEW_DIR="$(mktemp -d)"
cargo metadata --locked --offline --no-deps --format-version 1 > "$KASPA_MINIMAL_REVIEW_DIR/metadata.json"
```

未取得なら[第1章](../part-01-foundations/chapter-01.md)を参照し、HEADを基準版v1.0.1の完全SHA `fcd9c28f9b211b9872a0c90a8c94f79466ffb063` と照合する。metadataには絶対パスが含まれるため生JSONを公開しない。具体的な入口は[第25章](chapter-25.md)・[第26章](chapter-26.md)を参照する。

調査の順番は以下を推奨する。

1. ノード起動処理から、コンセンサスとネットワークの初期化を追う。
2. ブロック受信から検証処理への呼び出しを追う。
3. DAGデータの保存とコンセンサス計算の境界を探す。
4. Virtual Stateの更新経路とテストを読む。
5. Pruning・同期・再起動時の復旧処理を追う。

実装の場所を探すときは、`rg 'virtual|ghostdag|mempool|pruning'`などを入口にできる。ただし検索ヒットは根拠の出発点であって、動作の証明ではない。呼び出し元・テスト・対象コミットをセットで記録する。

## 27.21 テストを段階的に強化する

| レベル | 目的 | 例 |
|---|---|---|
| 単体テスト | 各責務の不変条件 | 二重入力の拒否 |
| 結合テスト | コンポーネント間の整合性 | ブロック拒否時のUTXO保持 |
| 性質テスト | 多様な入力で不変条件を検証 | 入力順の変更、重複ID |
| 障害注入 | 失敗時の回復 | 保存途中の停止 |
| 差分テスト | 公式実装との振る舞い比較 | 対象仕様に沿ったベクトル |
| Devnet実験 | 実ノードとの統合確認 | 同期、送信、ログ観測 |

差分テストは、**比較対象が同じ仕様を実装しているときに意味を持つ**。本章の簡略DAGをそのままKaspaのGHOSTDAGと比較しても、一致するはずはない。

## 27.22 失敗モードと設計上のトレードオフ

### 失敗1：親が届いていない

本章ではブロックを拒否する。実際のノードでは、欠落した親の取得や同期が必要になる。拒否だけではネットワーク遅延に弱い。

### 失敗2：受信順によって状態が変わる

本章最大の制約。DAGを保存する処理とコンセンサス順序を分離し、同じブロック集合に対して同じ状態を導出できる仕組みが必要になる。

### 失敗3：ブロック検証の途中でクラッシュする

候補を作る`clone`はこの教材の通常のErrでの状態保持に使うが、永続化のクラッシュ整合性までは保証しない。

### 失敗4：無制限のメモリ使用

ブロック、UTXO、Mempool、使用済みID履歴には容量上限がなく、DoS対策は未実装である。サイズ・件数・処理時間の上限を設ける必要がある。

### 失敗5：送信元の情報を信用する

ノードが「これは有効なブロックだ」と送ってきても、その主張を信用してはいけない。各ノードが独立に検証することが重要だ。

## 27.23 発展課題：最小ノードから次の段階へ

<details class="exercise-details">
<summary>発展課題を見る</summary>

### 課題A：未解決ブロックの保管
未知の親を参照するブロックを`OrphanPool`に一時保存し、親が到着したら再試行する。件数・保持時間の上限も設定する。

### 課題B：UTXOのUndo Log
ブロック適用前後の差分を記録し、再編成に備えたロールバックを実装する。複数ブロックの巻き戻し順序もテストする。

### 課題C：決定的な教材用順序
27.15の設計に沿ってDAGの保存と状態評価を分離する。まず、競合取引の適用失敗を理由にブロック全体をDAGから除外する現行の教材規則を変更し、親関係等の教材上の構造検査を通ったブロックを保存する。

そのうえで、親子関係を守り、並行ブロック間の優先規則も明示した決定的な順序を**教材独自のものとして**実装する。各ノードが同じブロック集合を取得した後、共通の初期UTXOからその順序で状態を再評価する。競合などで適用できない取引は状態へ反映せず、受理・不受理の結果を記録する。既に受信順で更新したUTXOへ並べ替えた取引を追加適用するだけでは、受信順の影響を除去できない。

27.14の競合ブロックを逆順に届け、最終的な保存済みDAG・評価順序・取引の受理結果・UTXO集合が一致することを確認する。これは教材の保存・受理規則を変更する演習であり、Kaspaのブロック検証・GHOSTDAG・Virtual Stateを再現するものではない。

### 課題D：TCPアダプター
Tokioで2ノードを接続し、メッセージのシリアライズ、タイムアウト、サイズ制限、再送を実装する。

### 課題E：公式実装の追跡
`rusty-kaspa`の特定コミットを固定し、「受信 → 検証 → DAG登録 → Virtual State更新」の実装経路を関数名・テスト名付きで記録する。

</details>

## 27.24 理解度チェック

<details class="quiz-answers">
<summary>理解度チェックを見る</summary>

1. なぜ複数親のDAGを作るだけではKaspa互換ノードにならないのか。
2. トランザクション検証後に状態を更新する設計にはどんな利点があるか。
3. MempoolのトランザクションとVirtual Stateで受理されたトランザクションは何が違うか。
4. ブロック受信順がノード間で異なると何が起こり得るか。
5. なぜ一般的なトポロジカルソートはGHOSTDAGの代替にならないのか。
6. 親が未知のブロックを受信した場合、どのような同期処理が必要か。
7. ストレージの原子性が失われると、どのような障害が起こるか。
8. P2Pの通信処理とコンセンサス処理を分離する利点は何か。
9. 本章のテストで保証できること、保証できないことは何か。
10. 公式実装を調査するときにコミットIDを記録する理由は何か。

**解答例**

1. Kaspa固有の形式・ハッシュ・PoW・DAA・GHOSTDAG・Virtual State・Script・同期等の規則を実装していないから。複数親はデータ構造の一条件にすぎない。
2. 途中の失敗で入力消費や出力追加だけが残ることを防ぎ、失敗時の状態不変と再試行を検査しやすくする。
3. Mempoolはローカルな候補、Virtual Stateでの受理は合意文脈の順序と検証を経た結果である。候補への登録だけで現在UTXOへ反映されたとはいえない。
4. 受信順で競合取引を適用すると、ノード間で異なるUTXO状態や勝者を選び得る。教材の処理を分散コンセンサスへ一般化しない。
5. トポロジカルソートは祖先関係を守る並べ方を求めるだけで、PoWに基づく親選択・blue/red分類・決定的な競合処理等を提供しないから。
6. 不足親と待機ブロックを管理し、P2P・同期で必要データを取得して、依存関係が揃った時点で再検証する。待機資源の上限も設ける。
7. 入力だけ消費された状態、履歴とUTXOの不一致、再起動後の二重適用等が残る可能性がある。更新単位と回復手順を定める。
8. 通信の再送・切断・入力制限と、決定的な検証・状態遷移を別々に設計・テストできる。通信成功を合意受理と混同しにくくなる。
9. 教材の固定入力に対する成功・拒否・失敗時状態保持等を確認できる。一方、実PoW・署名・GHOSTDAG・ネットワーク同期や任意入力でのコンセンサス安全性は保証しない。
10. 後から同じコード・テスト・規則を参照し、教材との対応や差分を確認できるようにするため。

</details>

## 27.25 まとめ

<details class="summary-details">
<summary>まとめを見る</summary>

本章では、Rustの標準ライブラリを使って、**DAG・UTXO・Mempool・Nodeを組み合わせた教育用の最小ノード**を作った。

この実験で得られる最も重要な理解は、**データを保存できることと、分散したノードが同じ状態に合意できることは別**だという点である。BlockDAGは並行して作られたブロックを表現できるが、それだけでは競合トランザクションの扱いを決められない。KaspaではGHOSTDAGによるコンセンサス順序とVirtual Stateがその役割を担う。

また、ノードを設計するときは、アルゴリズムだけでなく、検証の境界、状態更新の原子性、同期、永続化、障害回復、DoS耐性を一緒に考える必要がある。

</details>

[次章](chapter-28.md)では、ここで整理した責務を踏まえて、**Devnet上の実ノードを使った統合テスト**に進む。

## 27.26 参考資料

<details class="reference-details">
<summary>参考資料を見る</summary>

- [Kaspa公式サイト](https://kaspa.org/)
- [Kaspa公式Wiki](https://wiki.kaspa.org/)
- [rusty-kaspa（公式Rust実装）](https://github.com/kaspanet/rusty-kaspa)
- [Kaspa Improvement Proposals](https://github.com/kaspanet/kips)
- [PHANTOM and GHOSTDAG: A Scalable Generalization of Nakamoto Consensus](https://eprint.iacr.org/2018/104)
- [Rust公式ドキュメント](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Tokio公式ドキュメント](https://tokio.rs/)

</details>

<nav class="chapter-next" aria-label="次のページ">

[次章：Chapter 28 — Devnet & Integration Testing（Devnetと統合テスト）](chapter-28.md)

</nav>

<details class="verification-details">
<summary>検証状況を見る</summary>

[対象バージョン](../baseline.md)・[このページのレビュー記録](../review-status.md#review-chapter-27)

> **確認範囲:** 改訂後の教材16テスト・実行表示・書式・Clippyと本文同期を確認した。GHOSTDAG・署名・PoW・本物のP2P・永続化・reorg・Kaspa互換性は未実装であり、付録Gの実ノード実験と混同しない。記録は `experiments/chapter-27-review-2026-11-01.md`。


</details>
