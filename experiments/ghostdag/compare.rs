//! Run as an example in the pinned official workspace; no production keys or network.
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
    blockhash::{self, BlockHashes},
    config::params::{ForkActivation, DEVNET_PARAMS},
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
    fn parallel(&self, a: Hash, b: Hash) -> bool {
        !self.ancestor(a, b) && !self.ancestor(b, a)
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
    fn get_blue_score(&self, _: Hash) -> Result<u64, StoreError> {
        unreachable!("unused")
    }
    fn get_timestamp(&self, _: Hash) -> Result<u64, StoreError> {
        unreachable!("unused")
    }
    fn get_header(&self, _: Hash) -> Result<Arc<Header>, StoreError> {
        unreachable!("unused")
    }
    fn get_header_with_block_level(&self, _: Hash) -> Result<HeaderWithBlockLevel, StoreError> {
        unreachable!("unused")
    }
    fn get_compact_header_data(&self, _: Hash) -> Result<CompactHeaderData, StoreError> {
        unreachable!("unused")
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

#[derive(Clone)]
struct Expected {
    parent: Hash,
    blues: Vec<Hash>,
    reds: Vec<Hash>,
    past_blues: Set,
    score: u64,
    work: u64,
}
#[derive(Default)]
struct Coverage {
    compared: usize,
    candidate_rejections: usize,
    peer_rejections: usize,
    ties: usize,
    reds: usize,
}
// Independent fixture arithmetic: floor(2^256 / (target+1)) for these three bits values.
fn work(bits: u32) -> u64 {
    match bits {
        0x207fffff => 2,
        0x1f7fffff => 512,
        0x1e7fffff => 131072,
        _ => panic!("unsupported fixture bits"),
    }
}
fn oracle(
    graph: &Graph,
    nodes: &BTreeMap<Hash, Expected>,
    parents: &[Hash],
    k: u16,
    floor: u64,
    cov: &mut Coverage,
) -> Expected {
    let parent = *parents.iter().max_by_key(|h| (nodes[h].work, **h)).unwrap();
    if parents
        .iter()
        .filter(|h| nodes[h].work == nodes[&parent].work)
        .count()
        > 1
    {
        cov.ties += 1;
    }
    let past = graph.past(parent);
    let mut merge = Set::new();
    for h in parents {
        merge.extend(graph.past(*h).difference(&past).copied());
    }
    let mut merge: Vec<_> = merge.into_iter().collect();
    merge.sort_by_key(|h| (nodes[h].work, *h));
    let mut all = nodes[&parent].past_blues.clone();
    all.insert(parent);
    let mut blues = vec![parent];
    let mut reds = vec![];
    for candidate in merge {
        let peers: Vec<_> = all
            .iter()
            .copied()
            .filter(|h| graph.parallel(*h, candidate))
            .collect();
        let candidate_ok = peers.len() <= usize::from(k);
        let peers_ok = peers
            .iter()
            .all(|p| all.iter().filter(|h| graph.parallel(**h, *p)).count() < usize::from(k));
        if candidate_ok && peers_ok {
            all.insert(candidate);
            blues.push(candidate);
        } else {
            if !candidate_ok {
                cov.candidate_rejections += 1;
            }
            if candidate_ok && !peers_ok {
                cov.peer_rejections += 1;
            }
            reds.push(candidate);
        }
    }
    cov.reds += reds.len();
    let score = nodes[&parent].score + blues.len() as u64;
    let added: u64 = blues
        .iter()
        .map(|h| work(graph.0.borrow()[h].bits).max(floor))
        .sum();
    Expected {
        parent,
        blues,
        reds,
        past_blues: all,
        score,
        work: nodes[&parent].work + added,
    }
}
fn fixture(seed: u64, count: u64, mixed: bool) -> Vec<(u64, Vec<u64>, u32, u64)> {
    let mut state = seed;
    let mut rows = vec![];
    for id in 2..=count + 1 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut parents = BTreeSet::from([1 + state % (id - 1)]);
        for shift in [17, 33] {
            parents.insert(1 + (state >> shift) % (id - 1));
        }
        // Only incomparable tips can be parents in these fixtures.
        let bits = if mixed {
            [0x207fffff, 0x1f7fffff, 0x1e7fffff][(state % 3) as usize]
        } else {
            0x207fffff
        };
        rows.push((id, parents.into_iter().collect(), bits, (id - 1) * 10));
    }
    rows
}
fn run(
    name: &str,
    rows: &[(u64, Vec<u64>, u32, u64)],
    pre: u16,
    post: u16,
    activation: u64,
    level: bool,
    reverse: bool,
) -> serde_json::Value {
    let genesis = Hash::from(1u64);
    let graph = Graph::default();
    graph.0.borrow_mut().insert(
        genesis,
        Node {
            parents: vec![],
            bits: 0x207fffff,
            daa: 0,
        },
    );
    let store = Arc::new(MemoryGhostdagStore::new());
    let mut params = DEVNET_PARAMS;
    params.prior_ghostdag_k = pre;
    params.crescendo.ghostdag_k = post;
    params.crescendo_activation = ForkActivation::new(activation);
    let manager = if level {
        GhostdagManager::with_level(
            genesis,
            pre,
            store.clone(),
            graph.clone(),
            Arc::new(graph.clone()),
            graph.clone(),
            1,
            249,
        )
    } else {
        GhostdagManager::new(
            genesis,
            params.ghostdag_k(),
            store.clone(),
            graph.clone(),
            Arc::new(graph.clone()),
            graph.clone(),
        )
    };
    let genesis_data = manager.genesis_ghostdag_data();
    assert_eq!(genesis_data.blue_score, 0);
    assert_eq!(genesis_data.blue_work, 0u64);
    store.insert(genesis, Arc::new(genesis_data)).unwrap();
    store
        .insert(blockhash::ORIGIN, manager.origin_ghostdag_data())
        .unwrap();
    let origin = manager.ghostdag(&[blockhash::ORIGIN]);
    assert_eq!(origin.blue_score, 0);
    assert_eq!(origin.blue_work, 0u64);
    assert_eq!(origin.selected_parent, blockhash::ORIGIN);
    let mut expected = BTreeMap::from([(
        genesis,
        Expected {
            parent: blockhash::ORIGIN,
            blues: vec![],
            reds: vec![],
            past_blues: Set::new(),
            score: 0,
            work: 0,
        },
    )]);
    let mut cov = Coverage::default();
    let mut outcomes = vec![];
    for (id, raw_parents, bits, daa) in rows {
        let h = Hash::from(*id);
        let mut parents: Vec<Hash> = raw_parents.iter().map(|p| Hash::from(*p)).collect();
        let original = parents.clone();
        parents.retain(|p| {
            !original
                .iter()
                .any(|other| p != other && graph.ancestor(*p, *other))
        });
        if reverse {
            parents.reverse();
        }
        graph.0.borrow_mut().insert(
            h,
            Node {
                parents: parents.clone(),
                bits: *bits,
                daa: *daa,
            },
        );
        // Determine k without calling ForkedParam::get in the oracle.
        let selected = *parents
            .iter()
            .max_by_key(|h| (expected[h].work, **h))
            .unwrap();
        let k = if level || graph.0.borrow()[&selected].daa < activation {
            pre
        } else {
            post
        };
        let want = oracle(
            &graph,
            &expected,
            &parents,
            k,
            if level { 256 } else { 0 },
            &mut cov,
        );
        let actual = manager.ghostdag(&parents);
        assert_eq!(
            actual.selected_parent, want.parent,
            "{name} block {id} parent"
        );
        assert_eq!(
            actual.mergeset_blues.as_ref(),
            &want.blues,
            "{name} block {id} blues"
        );
        assert_eq!(
            actual.mergeset_reds.as_ref(),
            &want.reds,
            "{name} block {id} reds"
        );
        assert_eq!(actual.blue_score, want.score, "{name} block {id} score");
        assert_eq!(actual.blue_work, want.work, "{name} block {id} work");
        let labels = |hashes: &[Hash]| hashes.iter().map(ToString::to_string).collect::<Vec<_>>();
        outcomes.push(json!({"id":id,"selectedParent":want.parent.to_string(),"k":k,
            "blues":labels(&want.blues),"reds":labels(&want.reds),"blueScore":want.score,"blueWork":want.work.to_string()}));
        cov.compared += 1;
        store.insert(h, Arc::new(actual)).unwrap();
        expected.insert(h, want);
    }
    json!({"name":name,"comparedBlocks":cov.compared,"candidateRejections":cov.candidate_rejections,
        "existingBlueRejections":cov.peer_rejections,"parentWorkTies":cov.ties,"redClassifications":cov.reds,"outcomes":outcomes})
}
fn main() {
    let mut cases = vec![];
    // Wide anticone and a chain after it exercise k=0/1/2 and ordering ties.
    let simple = vec![
        (2, vec![1], 0x207fffff, 10),
        (3, vec![1], 0x207fffff, 20),
        (4, vec![1], 0x207fffff, 30),
        (5, vec![2, 3, 4], 0x207fffff, 40),
        (6, vec![2], 0x1f7fffff, 50),
        (7, vec![5, 6], 0x207fffff, 60),
    ];
    for k in [0, 1, 2, 18] {
        let a = run(
            &format!("explicit-k{k}"),
            &simple,
            k,
            k,
            u64::MAX,
            false,
            false,
        );
        let b = run(
            &format!("explicit-k{k}"),
            &simple,
            k,
            k,
            u64::MAX,
            false,
            true,
        );
        assert_eq!(a, b, "parent array order changed the output");
        cases.push(a);
    }
    for seed in 1..=8 {
        for mixed in [false, true] {
            let rows = fixture(seed, 40, mixed);
            let name = format!("generated-{seed}-mixed-{mixed}");
            let k = (seed % 4) as u16;
            let a = run(&name, &rows, k, k, u64::MAX, false, false);
            assert_eq!(a, run(&name, &rows, k, k, u64::MAX, false, true));
            cases.push(a);
        }
    }
    // Activation at 20 deliberately differs from the current block's DAA score.
    for boundary in [19, 20, 21] {
        let mut rows = simple.clone();
        rows[2].3 = boundary;
        let name = format!("activation-selected-parent-{boundary}");
        let a = run(&name, &rows, 0, 2, 20, false, false);
        assert_eq!(a, run(&name, &rows, 0, 2, 20, false, true));
        cases.push(a);
    }
    let rows = fixture(42, 30, true);
    let a = run("level-work-floor", &rows, 2, 2, u64::MAX, true, false);
    assert_eq!(
        a,
        run("level-work-floor", &rows, 2, 2, u64::MAX, true, true)
    );
    cases.push(a);
    let count = |key: &str| cases.iter().map(|c| c[key].as_u64().unwrap()).sum::<u64>();
    assert!(count("candidateRejections") > 0);
    assert!(count("existingBlueRejections") > 0);
    assert!(count("parentWorkTies") > 0);
    println!("{}",serde_json::to_string_pretty(&json!({"baselineCommit":"fcd9c28f9b211b9872a0c90a8c94f79466ffb063",
        "officialFunction":"GhostdagManager::ghostdag","network":"synthetic fixtures; no node or PoW validation",
        "caseCount":cases.len(),"comparedBlocksPerParentOrder":count("comparedBlocks"),
        "parentOrdersCompared":2,"candidateRejections":count("candidateRejections"),
        "existingBlueRejections":count("existingBlueRejections"),"parentWorkTies":count("parentWorkTies"),
        "allComparisonsMatched":true,"cases":cases})).unwrap());
}
