// Copyright 2026 shiga0 and contributors. SPDX-License-Identifier: MIT
// These are hashing fixtures, not consensus-valid transactions or signing requests.
use kaspa_consensus_core::{
    Hash,
    hashing::{
        sighash::{SigHashReusedValuesUnsync, calc_schnorr_signature_hash},
        sighash_type::SIG_HASH_ALL,
        tx::{hash, id},
    },
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{
        ComputeCommit, CovenantBinding, PopulatedTransaction, ScriptPublicKey, Transaction,
        TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
    },
};

fn digests(tx: &Transaction, entry: &UtxoEntry) -> [String; 3] {
    // Avoid a cloned Transaction's cached ID and a previous sighash cache.
    let populated = PopulatedTransaction::new(tx, vec![entry.clone()]);
    [
        id(tx).to_string(),
        hash(tx).to_string(),
        calc_schnorr_signature_hash(
            &populated,
            0,
            SIG_HASH_ALL,
            &SigHashReusedValuesUnsync::new(),
        )
        .to_string(),
    ]
}

#[test]
fn v1_mutation_commitments() {
    let spk = ScriptPublicKey::new(0, vec![0x51].into());
    let base = Transaction::new(
        1,
        vec![TransactionInput::new_with_compute_budget(
            TransactionOutpoint::new(Hash::from_bytes([1; 32]), 0),
            vec![0x51],
            0,
            10,
        )],
        vec![TransactionOutput {
            value: 900,
            script_public_key: spk.clone(),
            covenant: Some(CovenantBinding::new(0, Hash::from_bytes([2; 32]))),
        }],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );
    let entry = UtxoEntry::new(1_000, spk, 0, false, Some(Hash::from_bytes([2; 32])));
    let original = digests(&base, &entry);
    let mut cases = vec![];
    for (name, expected) in [
        ("signature-script", [false, true, false]),
        ("compute-budget", [false, true, false]),
        ("storage-mass", [false, true, false]),
        ("output-value", [true, true, true]),
        ("output-script", [true, true, true]),
        ("covenant-id", [true, true, true]),
        ("authorizing-input", [true, true, true]),
        ("remove-binding", [true, true, true]),
        ("previous-outpoint", [true, true, true]),
        ("sequence", [true, true, true]),
        ("payload", [true, true, true]),
        ("lock-time", [true, true, true]),
        ("utxo-amount", [false, false, true]),
        ("utxo-script", [false, false, true]),
    ] {
        let mut changed = base.clone();
        let mut previous = entry.clone();
        match name {
            "signature-script" => changed.inputs[0].signature_script.push(0x51),
            "compute-budget" => {
                changed.inputs[0].compute_commit = ComputeCommit::ComputeBudget(11.into())
            }
            "storage-mass" => changed.set_storage_mass(1),
            "output-value" => changed.outputs[0].value -= 1,
            "output-script" => {
                changed.outputs[0].script_public_key = ScriptPublicKey::new(0, vec![0x52].into())
            }
            "covenant-id" => {
                changed.outputs[0].covenant.as_mut().unwrap().covenant_id =
                    Hash::from_bytes([3; 32])
            }
            "authorizing-input" => {
                changed.outputs[0]
                    .covenant
                    .as_mut()
                    .unwrap()
                    .authorizing_input = 1
            }
            "remove-binding" => changed.outputs[0].covenant = None,
            "previous-outpoint" => changed.inputs[0].previous_outpoint.index = 1,
            "sequence" => changed.inputs[0].sequence = 1,
            "payload" => changed.payload = vec![1],
            "lock-time" => changed.lock_time = 1,
            "utxo-amount" => previous.amount += 1,
            "utxo-script" => {
                previous.script_public_key = ScriptPublicKey::new(0, vec![0x52].into())
            }
            _ => unreachable!(),
        }
        let actual = digests(&changed, &previous);
        let differences = std::array::from_fn::<_, 3, _>(|i| actual[i] != original[i]);
        assert_eq!(differences, expected, "{name}");
        cases.push(serde_json::json!({
            "case": name, "idChanged": differences[0], "transactionHashChanged": differences[1],
            "sighashAllChanged": differences[2],
        }));
    }
    println!(
        "BOOK_TX_V1_RESULT={}",
        serde_json::json!({"cases": cases, "allChecksPassed": true})
    );
}
