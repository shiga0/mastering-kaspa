//! Sampled windows and Dual routing on explicit synthetic activation boundaries.
use kaspa_consensus::model::{
    services::reachability::ReachabilityService,
    stores::{
        ghostdag::{GhostdagStore, MemoryGhostdagStore},
        headers::{CompactHeaderData, HeaderStoreReader, HeaderWithBlockLevel},
        relations::RelationsStoreReader,
    },
};
use kaspa_consensus::processes::{ghostdag::protocol::GhostdagManager, reachability};
use kaspa_consensus_core::{
    blockhash::BlockHashes,
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
use kaspa_consensus::model::stores::daa::DaaStoreReader;
use kaspa_consensus::processes::window::{
    DaaWindow, DualWindowManager, FullWindowManager, SampledWindowManager, WindowManager,
    WindowType,
};
use kaspa_consensus_core::config::params::ForkActivation;
use kaspa_database::prelude::CachePolicy;
use kaspa_math::Uint256;
use serde::Deserialize;
use std::io::{self, Read};

struct UnusedDaaStore;
impl DaaStoreReader for UnusedDaaStore {
    fn get_mergeset_non_daa(&self, _: Hash) -> Result<Arc<BlockHashSet>, StoreError> {
        unreachable!("not used by these entry points")
    }
}
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
    full_size: usize,
    sampled_size: usize,
    sample_rate: u64,
    activation: u64,
}
fn window_contents(
    data: &DaaWindow,
    names: &BTreeMap<Hash, u64>,
) -> (Vec<u64>, Vec<u64>, u64, String) {
    let mut window: Vec<_> = data.window.iter().map(|b| names[&b.0.hash]).collect();
    window.sort_unstable();
    let mut excluded: Vec<_> = data.mergeset_non_daa.iter().map(|h| names[h]).collect();
    excluded.sort_unstable();
    (
        window,
        excluded,
        data.daa_score,
        format!("{:?}", data.window.origin()),
    )
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
    let median = Arc::new(BlockWindowCacheStore::new(CachePolicy::Empty));
    let mut genesis_info = DEVNET_GENESIS;
    genesis_info.hash = genesis;
    genesis_info.bits = 0x207fffff;
    genesis_info.timestamp = 0;
    let maximum = Uint256::from_compact_target_bits(0x207fffff);
    let activation = ForkActivation::new(case.activation);
    let dual = DualWindowManager::new(
        &genesis_info,
        store.clone(),
        Arc::new(graph.clone()),
        Arc::new(UnusedDaaStore),
        cache.clone(),
        median.clone(),
        maximum,
        1000,
        100,
        activation,
        case.full_size,
        case.sampled_size,
        3,
        case.sample_rate,
        17,
        9,
        2,
    );
    let full = FullWindowManager::new(
        &genesis_info,
        store.clone(),
        Arc::new(graph.clone()),
        cache.clone(),
        median.clone(),
        maximum,
        1000,
        case.full_size,
        3.min(case.full_size),
        17,
    );
    let sampled = SampledWindowManager::new(
        &genesis_info,
        store.clone(),
        Arc::new(graph.clone()),
        Arc::new(UnusedDaaStore),
        cache.clone(),
        median,
        maximum,
        1000,
        100,
        activation,
        case.sampled_size,
        3.min(case.sampled_size),
        case.sample_rate,
        9,
        2,
    );
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
        let sp_daa = graph.0.borrow()[&gd.selected_parent].daa;
        let active = sp_daa >= case.activation;
        if cache_mode == "wrong-origin" && gd.selected_parent != genesis {
            let opposite = if active {
                WindowOrigin::Full
            } else {
                WindowOrigin::Sampled
            };
            cache.insert(gd.selected_parent, Arc::new(BlockWindowHeap::new(opposite)));
        }
        let daa = dual.block_daa_window(&gd).unwrap();
        let direct = if active {
            sampled.block_daa_window(&gd).unwrap()
        } else {
            full.block_daa_window(&gd).unwrap()
        };
        assert_eq!(
            window_contents(&daa, &names),
            window_contents(&direct, &names),
            "Dual vs direct {}",
            case.name
        );
        let bits = dual.calculate_difficulty_bits(&gd, &daa);
        let direct_bits = if active {
            sampled.calculate_difficulty_bits(&gd, &direct)
        } else {
            full.calculate_difficulty_bits(&gd, &direct)
        };
        assert_eq!(bits, direct_bits);
        assert_eq!(
            dual.window_size(&gd, WindowType::DifficultyWindow),
            if active {
                case.sampled_size
            } else {
                case.full_size
            }
        );
        assert_eq!(
            dual.sample_rate(&gd, WindowType::DifficultyWindow),
            if active { case.sample_rate } else { 1 }
        );
        let (window, non_daa, score, origin) = window_contents(&daa, &names);
        let mut merge: Vec<_> = gd.unordered_mergeset().map(|h| names[&h]).collect();
        merge.sort_unstable();
        let mut normalized: Vec<_> = parents.iter().map(|h| names[h]).collect();
        normalized.sort_unstable();
        output.push(json!({"id":node.id,"parents":normalized,"hash":h.to_string(),"selectedParent":names[&gd.selected_parent],
   "selectedParentDaa":sp_daa,"blueWork":gd.blue_work.to_string(),"blueScore":gd.blue_score,"daaScore":score,
   "window":window,"nonDaa":non_daa,"mergeSet":merge,"computedBits":bits,"origin":origin}));
        graph.0.borrow_mut().insert(
            h,
            Node {
                parents,
                bits: node.bits,
                daa: score,
                timestamp: node.timestamp,
                blue: gd.blue_score,
            },
        );
        if cache_mode == "warm" {
            cache.insert(h, daa.window.clone());
        }
        names.insert(h, node.id);
        store.insert(h, Arc::new(gd)).unwrap();
    }
    json!({"name":case.name,"blocks":output})
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
                    "{} {cache} reverse={reverse}",
                    case.name
                );
            }
        }
        results.push(result);
    }
    println!(
        "BOOK_TRANSITION_RESULTS {}",
        serde_json::to_string(&results).unwrap()
    );
}
