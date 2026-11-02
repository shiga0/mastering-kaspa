//! Execute pinned BlockDepthManager + official in-memory Reachability on fixtures.
use kaspa_consensus::{
    model::{
        services::reachability::MTReachabilityService,
        stores::{
            depth::DepthStoreReader,
            ghostdag::{GhostdagData, GhostdagStore, MemoryGhostdagStore},
            headers::{CompactHeaderData, HeaderStoreReader, HeaderWithBlockLevel},
            reachability::MemoryReachabilityStore,
        },
    },
    processes::{block_depth::BlockDepthManager, reachability::inquirer},
};
use kaspa_consensus_core::{
    blockhash::ORIGIN,
    config::params::{ForkActivation, DEVNET_PARAMS},
    header::Header,
};
use kaspa_database::prelude::StoreError;
use kaspa_hashes::Hash;
use parking_lot::RwLock;
use serde::Deserialize;
use serde_json::json;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Read},
    sync::Arc,
};

#[derive(Deserialize)]
struct Node {
    id: u64,
    parent: u64,
    blue: u64,
    daa: u64,
    #[serde(default)]
    merges: Vec<u64>,
}
#[derive(Deserialize)]
struct Query {
    name: String,
    parent: u64,
    blue: u64,
    pruning: u64,
    store_as: Option<u64>,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    pre: u64,
    post: u64,
    activation: u64,
    nodes: Vec<Node>,
    previous: Vec<(u64, u64)>,
    queries: Vec<Query>,
}
fn hash(id: u64) -> Hash {
    if id == 0 {
        ORIGIN
    } else {
        Hash::from(id)
    }
}
struct Depths(RefCell<BTreeMap<Hash, Hash>>);
impl DepthStoreReader for Depths {
    fn merge_depth_root(&self, _: Hash) -> Result<Hash, StoreError> {
        unreachable!("finality only")
    }
    fn finality_point(&self, h: Hash) -> Result<Hash, StoreError> {
        Ok(self.0.borrow()[&h])
    }
}
struct Headers(BTreeMap<Hash, CompactHeaderData>);
impl HeaderStoreReader for Headers {
    fn get_daa_score(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0[&h].daa_score)
    }
    fn get_blue_score(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0[&h].blue_score)
    }
    fn get_bits(&self, _: Hash) -> Result<u32, StoreError> {
        unreachable!("not used")
    }
    fn get_timestamp(&self, _: Hash) -> Result<u64, StoreError> {
        unreachable!("not used")
    }
    fn get_compact_header_data(&self, h: Hash) -> Result<CompactHeaderData, StoreError> {
        Ok(self.0[&h])
    }
    fn get_header(&self, _: Hash) -> Result<Arc<Header>, StoreError> {
        unreachable!("not used")
    }
    fn get_header_with_block_level(&self, _: Hash) -> Result<HeaderWithBlockLevel, StoreError> {
        unreachable!("not used")
    }
}
fn run(case: &Case) -> serde_json::Value {
    let mut reachability = MemoryReachabilityStore::new();
    inquirer::init(&mut reachability).unwrap();
    let ghostdag = Arc::new(MemoryGhostdagStore::new());
    let mut headers = BTreeMap::new();
    for node in &case.nodes {
        inquirer::add_block(
            &mut reachability,
            hash(node.id),
            hash(node.parent),
            &mut node.merges.iter().copied().map(hash),
        )
        .unwrap();
        ghostdag
            .insert(
                hash(node.id),
                Arc::new(GhostdagData {
                    selected_parent: hash(node.parent),
                    blue_score: node.blue,
                    ..Default::default()
                }),
            )
            .unwrap();
        assert!(headers
            .insert(
                hash(node.id),
                CompactHeaderData {
                    daa_score: node.daa,
                    blue_score: node.blue,
                    bits: 0,
                    timestamp: 0
                }
            )
            .is_none());
    }
    if case.name == "off-chain-pruning" {
        assert!(inquirer::is_dag_ancestor_of(&reachability, hash(100), hash(101)).unwrap());
        assert!(!inquirer::is_chain_ancestor_of(&reachability, hash(100), hash(101)).unwrap());
    }
    let points = Arc::new(Depths(RefCell::new(
        case.previous
            .iter()
            .map(|&(id, point)| (hash(id), hash(point)))
            .collect(),
    )));
    let mut params = DEVNET_PARAMS;
    params.prior_finality_depth = case.pre;
    params.crescendo.finality_depth = case.post;
    params.crescendo_activation = ForkActivation::new(case.activation);
    let manager = BlockDepthManager::new(
        params.merge_depth(),
        params.finality_depth(),
        hash(1),
        points.clone(),
        MTReachabilityService::new(Arc::new(RwLock::new(reachability))),
        ghostdag,
        Arc::new(Headers(headers)),
    );
    let mut outcomes = Vec::new();
    for query in &case.queries {
        let point = manager.calc_finality_point(
            &GhostdagData {
                selected_parent: hash(query.parent),
                blue_score: query.blue,
                ..Default::default()
            },
            hash(query.pruning),
        );
        let point_id = if point == ORIGIN {
            0
        } else {
            case.nodes.iter().find(|n| hash(n.id) == point).unwrap().id
        };
        if let Some(id) = query.store_as {
            assert!(points.0.borrow_mut().insert(hash(id), point).is_none());
        }
        outcomes.push(json!({"name": query.name, "point": point_id}));
    }
    json!({"name":case.name, "outcomes":outcomes})
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Case> = serde_json::from_str(&input).unwrap();
    let results: Vec<_> = cases.iter().map(run).collect();
    println!(
        "BOOK_FINALITY_RESULTS {}",
        serde_json::to_string(&results).unwrap()
    );
}
