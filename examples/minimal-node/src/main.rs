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
