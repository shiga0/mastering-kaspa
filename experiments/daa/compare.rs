//! Invoke only the pinned official DAA arithmetic on explicit synthetic windows.
use kaspa_consensus::{
    model::stores::{
        block_window_cache::{BlockWindowHeap, WindowOrigin},
        ghostdag::{GhostdagData, GhostdagStore, MemoryGhostdagStore},
        headers::{CompactHeaderData, HeaderStoreReader, HeaderWithBlockLevel},
    },
    processes::{
        difficulty::{FullDifficultyManager, SampledDifficultyManager},
        ghostdag::ordering::SortableBlock,
    },
};
use kaspa_consensus_core::{config::params::ForkActivation, header::Header};
use kaspa_database::prelude::StoreError;
use kaspa_hashes::Hash;
use kaspa_math::Uint256;
use serde::Deserialize;
use serde_json::json;
use std::{
    cmp::Reverse,
    collections::BTreeMap,
    io::{self, Read},
    sync::Arc,
};

#[derive(Clone, Deserialize)]
struct Sample {
    id: u64,
    timestamp: u64,
    bits: u32,
    work: u64,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    mode: String,
    samples: Vec<Sample>,
    min_window: usize,
    window_size: usize,
    target_ms: u64,
    sample_rate: u64,
    max_bits: u32,
    genesis_bits: u32,
    selected_genesis: bool,
    selected_bits: u32,
    grandparent_daa: u64,
    activation: u64,
    prior_target_ms: u64,
}
struct Headers(BTreeMap<Hash, CompactHeaderData>);
impl HeaderStoreReader for Headers {
    fn get_daa_score(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0[&h].daa_score)
    }
    fn get_bits(&self, h: Hash) -> Result<u32, StoreError> {
        Ok(self.0[&h].bits)
    }
    fn get_timestamp(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0[&h].timestamp)
    }
    fn get_blue_score(&self, h: Hash) -> Result<u64, StoreError> {
        Ok(self.0[&h].blue_score)
    }
    fn get_compact_header_data(&self, h: Hash) -> Result<CompactHeaderData, StoreError> {
        Ok(self.0[&h])
    }
    fn get_header(&self, _: Hash) -> Result<Arc<Header>, StoreError> {
        unreachable!("not used by these functions")
    }
    fn get_header_with_block_level(&self, _: Hash) -> Result<HeaderWithBlockLevel, StoreError> {
        unreachable!("unused")
    }
}
fn run(case: &Case, reverse: bool) -> u32 {
    let genesis = Hash::from(1u64);
    let parent = if case.selected_genesis {
        genesis
    } else {
        Hash::from(2u64)
    };
    let grandparent = Hash::from(3u64);
    let data = |bits, daa| CompactHeaderData {
        bits,
        daa_score: daa,
        timestamp: 0,
        blue_score: 0,
    };
    let mut headers = BTreeMap::from([
        (genesis, data(case.genesis_bits, 0)),
        (Hash::from(2u64), data(case.selected_bits, case.activation)),
        (grandparent, data(case.genesis_bits, case.grandparent_daa)),
    ]);
    let mut window = BlockWindowHeap::new(if case.mode == "full" {
        WindowOrigin::Full
    } else {
        WindowOrigin::Sampled
    });
    let mut samples = case.samples.clone();
    if reverse {
        samples.reverse();
    }
    for sample in samples {
        assert!((10..250).contains(&sample.id));
        assert!(headers
            .insert(
                Hash::from(sample.id),
                CompactHeaderData {
                    bits: sample.bits,
                    timestamp: sample.timestamp,
                    daa_score: 0,
                    blue_score: 0
                }
            )
            .is_none());
        window.push(Reverse(SortableBlock::new(
            Hash::from(sample.id),
            sample.work.into(),
        )));
    }
    let headers = Arc::new(Headers(headers));
    let maximum = Uint256::from_compact_target_bits(case.max_bits);
    if case.mode == "full" {
        FullDifficultyManager::new(
            headers,
            case.genesis_bits,
            maximum,
            case.window_size,
            case.min_window,
            case.target_ms,
        )
        .calculate_difficulty_bits(&window)
    } else {
        assert_eq!(case.mode, "sampled");
        let store = Arc::new(MemoryGhostdagStore::new());
        store
            .insert(
                Hash::from(2u64),
                Arc::new(GhostdagData::new_with_selected_parent(grandparent, 2)),
            )
            .unwrap();
        let manager = SampledDifficultyManager::new(
            headers,
            store,
            genesis,
            case.genesis_bits,
            maximum,
            case.window_size,
            case.min_window,
            case.sample_rate,
            case.prior_target_ms,
            case.target_ms,
            ForkActivation::new(case.activation),
        );
        manager
            .calculate_difficulty_bits(&window, &GhostdagData::new_with_selected_parent(parent, 2))
    }
}
fn main() {
    assert!(Hash::from(10u64) < Hash::from(11u64));
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Case> = serde_json::from_str(&input).unwrap();
    let mut results = vec![];
    for case in cases {
        let bits = run(&case, false);
        assert_eq!(
            bits,
            run(&case, true),
            "{}: heap insertion order changed bits",
            case.name
        );
        results.push(json!({"name":case.name,"bits":bits,"reversedWindowInsertionMatched":true}));
    }
    println!(
        "BOOK_DAA_RESULTS {}",
        serde_json::to_string(&results).unwrap()
    );
}
