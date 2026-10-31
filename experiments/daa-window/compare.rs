//! Full DAA windows on synthetic DAGs, using the pinned official implementation.
use kaspa_consensus::model::{
    services::reachability::ReachabilityService,
    stores::{
        ghostdag::{GhostdagData, GhostdagStore, MemoryGhostdagStore},
        headers::{CompactHeaderData, HeaderStoreReader, HeaderWithBlockLevel},
        relations::RelationsStoreReader,
    },
};
use kaspa_consensus::processes::{ghostdag::protocol::GhostdagManager, reachability};
use kaspa_consensus_core::{
    blockhash::{self, BlockHashes},
    config::{genesis::DEVNET_GENESIS, params::ForkedParam},
    header::Header,
    BlockHashSet,
};
use kaspa_database::prelude::{ReadLock, StoreError, StoreResult};
use kaspa_hashes::Hash;
use serde_json::json;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::Arc,
};

type Set = BTreeSet<Hash>;
#[derive(Clone)]
struct Node {
    parents: Vec<Hash>,
    bits: u32,
    daa: u64,
    timestamp: u64,
    blue: u64,
}
#[derive(Clone, Default)]
struct Graph(Rc<RefCell<BTreeMap<Hash, Node>>>);
impl Graph {
    fn past(&self, hash: Hash) -> Set {
        let graph = self.0.borrow();
        let mut result = Set::new();
        let mut stack = vec![hash];
        while let Some(next) = stack.pop() {
            if result.insert(next) {
                stack.extend(&graph[&next].parents);
            }
        }
        result
    }
    fn ancestor(&self, a: Hash, b: Hash) -> bool {
        self.past(b).contains(&a)
    }
}
impl RelationsStoreReader for Graph {
    fn get_parents(&self, h: Hash) -> Result<BlockHashes, StoreError> {
        Ok(Arc::new(self.0.borrow()[&h].parents.clone()))
    }
    fn has(&self, h: Hash) -> Result<bool, StoreError> {
        Ok(self.0.borrow().contains_key(&h))
    }
    fn counts(&self) -> Result<(usize, usize), StoreError> {
        let n = self.0.borrow().len();
        Ok((n, n))
    }
    fn get_children(&self, _: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        unreachable!("unused by GHOSTDAG")
    }
}
impl HeaderStoreReader for Graph {
    fn get_bits(&self, h: Hash) -> Result<u32, StoreError> {
        Ok(self.0.borrow()[&h].bits)
    }
    fn get_daa_score(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0.borrow()[&h].daa)
    }
    fn get_blue_score(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0.borrow()[&h].blue)
    }
    fn get_timestamp(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0.borrow()[&h].timestamp)
    }
    fn get_header(&self, _: Hash) -> Result<Arc<Header>, StoreError> {
        unreachable!("unused")
    }
    fn get_header_with_block_level(&self, _: Hash) -> Result<HeaderWithBlockLevel, StoreError> {
        unreachable!("unused")
    }
    fn get_compact_header_data(&self, h: Hash) -> Result<CompactHeaderData, StoreError> {
        let data = self.0.borrow();
        let n = &data[&h];
        Ok(CompactHeaderData {
            daa_score: n.daa,
            blue_score: n.blue,
            timestamp: n.timestamp,
            bits: n.bits,
        })
    }
}
impl ReachabilityService for Graph {
    fn is_dag_ancestor_of(&self, a: Hash, b: Hash) -> bool {
        self.ancestor(a, b)
    }
    fn is_dag_ancestor_of_result(&self, a: Hash, b: Hash) -> reachability::Result<bool> {
        Ok(self.ancestor(a, b))
    }
    fn is_dag_ancestor_of_any(&self, a: Hash, b: &mut impl Iterator<Item = Hash>) -> bool {
        b.any(|b| self.ancestor(a, b))
    }
    fn is_any_dag_ancestor(&self, a: &mut impl Iterator<Item = Hash>, b: Hash) -> bool {
        a.any(|a| self.ancestor(a, b))
    }
    fn is_any_dag_ancestor_result(
        &self,
        a: &mut impl Iterator<Item = Hash>,
        b: Hash,
    ) -> reachability::Result<bool> {
        Ok(self.is_any_dag_ancestor(a, b))
    }
    fn is_chain_ancestor_of(&self, _: Hash, _: Hash) -> bool {
        unreachable!("unused")
    }
    fn get_next_chain_ancestor(&self, _: Hash, _: Hash) -> Hash {
        unreachable!("unused")
    }
    fn get_chain_parent(&self, _: Hash) -> Hash {
        unreachable!("unused")
    }
    fn has_reachability_data(&self, h: Hash) -> bool {
        self.0.borrow().contains_key(&h)
    }
}

use kaspa_consensus::model::stores::block_window_cache::{
    BlockWindowCacheStore, BlockWindowCacheWriter, BlockWindowHeap, WindowOrigin,
};
use kaspa_consensus::processes::window::{FullWindowManager, WindowManager};
use kaspa_consensus_core::errors::block::RuleError;
use kaspa_database::prelude::CachePolicy;
use kaspa_math::Uint256;
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Clone, Deserialize)]
struct InputNode {
    id: u64,
    parents: Vec<u64>,
    bits: u32,
    timestamp: u64,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    nodes: Vec<InputNode>,
    k: u16,
    window_size: usize,
}
fn execute(case: &Case, cache_mode: &str, reverse: bool) -> serde_json::Value {
    let genesis = Hash::from(1u64);
    let graph = Graph::default();
    graph.0.borrow_mut().insert(
        genesis,
        Node {
            parents: vec![],
            bits: 0x207fffff,
            daa: 0,
            timestamp: 0,
            blue: 0,
        },
    );
    let store = Arc::new(MemoryGhostdagStore::new());
    let ghostdag = GhostdagManager::new(
        genesis,
        ForkedParam::new_const(case.k),
        store.clone(),
        graph.clone(),
        Arc::new(graph.clone()),
        graph.clone(),
    );
    store
        .insert(genesis, Arc::new(ghostdag.genesis_ghostdag_data()))
        .unwrap();
    let cache = Arc::new(BlockWindowCacheStore::new(CachePolicy::Count(1024)));
    let mut genesis_info = DEVNET_GENESIS;
    genesis_info.hash = genesis;
    genesis_info.bits = 0x207fffff;
    genesis_info.timestamp = 0;
    let windows = FullWindowManager::new(
        &genesis_info,
        store.clone(),
        Arc::new(graph.clone()),
        cache.clone(),
        Arc::new(BlockWindowCacheStore::new(CachePolicy::Empty)),
        Uint256::from_compact_target_bits(0x207fffff),
        1000,
        case.window_size,
        case.window_size.min(3),
        17,
    );
    let origin = GhostdagData::new_with_selected_parent(blockhash::ORIGIN, 1);
    if case.window_size > 0 {
        assert!(matches!(
            windows.block_daa_window(&origin),
            Err(RuleError::InsufficientDaaWindowSize(0))
        ));
    }
    let mut names = BTreeMap::from([(genesis, 1u64)]);
    let mut output = vec![];
    for node in &case.nodes {
        assert!((2..250).contains(&node.id));
        let h = Hash::from(node.id);
        assert!(!names.contains_key(&h));
        let mut parents: Vec<Hash> = node.parents.iter().map(|p| Hash::from(*p)).collect();
        let original = parents.clone();
        parents.retain(|p| !original.iter().any(|q| p != q && graph.ancestor(*p, *q)));
        if reverse {
            parents.reverse();
        }
        let gd = ghostdag.ghostdag(&parents);
        let daa = windows.block_daa_window(&gd).unwrap();
        let mut window: Vec<u64> = daa.window.iter().map(|b| names[&b.0.hash]).collect();
        window.sort_unstable();
        let mut non_daa: Vec<u64> = daa.mergeset_non_daa.iter().map(|h| names[h]).collect();
        non_daa.sort_unstable();
        let mut merge: Vec<u64> = gd.unordered_mergeset().map(|h| names[&h]).collect();
        merge.sort_unstable();
        let mut normalized: Vec<_> = parents.iter().map(|h| names[h]).collect();
        normalized.sort_unstable();
        let bits = if case.window_size >= 2 {
            Some(windows.calculate_difficulty_bits(&gd, &daa))
        } else {
            None
        };
        output.push(json!({"id":node.id,"parents":normalized,"hash":h.to_string(),"selectedParent":names[&gd.selected_parent],
   "blueWork":gd.blue_work.to_string(),"blueScore":gd.blue_score,"daaScore":daa.daa_score,
   "window":window,"nonDaa":non_daa,"mergeSet":merge,"computedBits":bits}));
        graph.0.borrow_mut().insert(
            h,
            Node {
                parents,
                bits: node.bits,
                daa: daa.daa_score,
                timestamp: node.timestamp,
                blue: gd.blue_score,
            },
        );
        if cache_mode == "warm" {
            cache.insert(h, daa.window.clone());
        } else if cache_mode == "wrong-origin" {
            cache.insert(h, Arc::new(BlockWindowHeap::new(WindowOrigin::Sampled)));
        }
        names.insert(h, node.id);
        store.insert(h, Arc::new(gd)).unwrap();
    }
    json!({"name":case.name,"k":case.k,"windowSize":case.window_size,"blocks":output})
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Case> = serde_json::from_str(&input).unwrap();
    let mut results = vec![];
    for case in cases {
        let result = execute(&case, "cold", false);
        for cache in ["cold", "warm", "wrong-origin"] {
            for reverse in [false, true] {
                if cache == "cold" && !reverse {
                    continue;
                }
                assert_eq!(
                    result,
                    execute(&case, cache, reverse),
                    "{} cache={cache} reverse={reverse}",
                    case.name
                );
            }
        }
        results.push(result);
    }
    println!(
        "BOOK_WINDOW_RESULTS {}",
        serde_json::to_string(&results).unwrap()
    );
}
