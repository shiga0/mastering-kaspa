use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct OutPoint {
    txid: String,
    index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Output {
    value: u64,
}

#[derive(Clone, Debug)]
struct Transaction {
    id: String,
    inputs: Vec<OutPoint>,
    outputs: Vec<Output>,
}

#[derive(Clone, Debug)]
struct Block {
    id: String,
    parents: Vec<String>,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Default)]
struct UtxoSet {
    entries: HashMap<OutPoint, Output>,
}

impl UtxoSet {
    fn apply(&mut self, tx: &Transaction) -> Result<(), String> {
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
        for (index, _) in tx.outputs.iter().enumerate() {
            let point = OutPoint {
                txid: tx.id.clone(),
                index: index as u32,
            };
            if self.entries.contains_key(&point) && !seen.contains(&point) {
                return Err("output collision".into());
            }
        }
        // ここまで成功してから変更する。
        for input in &tx.inputs {
            self.entries.remove(input);
        }
        for (index, output) in tx.outputs.iter().enumerate() {
            self.entries.insert(
                OutPoint {
                    txid: tx.id.clone(),
                    index: index as u32,
                },
                output.clone(),
            );
        }
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

#[derive(Default)]
struct Mempool {
    transactions: HashMap<String, Transaction>,
    reserved_inputs: HashSet<OutPoint>,
}

impl Mempool {
    fn insert(&mut self, tx: Transaction) -> Result<(), String> {
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

#[derive(Default)]
struct Dag {
    blocks: HashMap<String, Block>,
}

impl Dag {
    fn insert(&mut self, block: Block) -> Result<(), String> {
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

#[derive(Default)]
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
        let before = node.utxos.entries.clone();
        assert!(node
            .accept_block(Block {
                id: "bad".into(),
                parents: vec!["genesis".into()],
                transactions: vec![spend("tx1", 101)],
            })
            .is_err());
        assert_eq!(node.utxos.entries, before);
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
        let before = state.entries.clone();
        let mut tx = spend("tx1", 50);
        tx.inputs.push(tx.inputs[0].clone());
        assert!(state.apply(&tx).is_err());
        assert_eq!(state.entries, before);
    }
}
