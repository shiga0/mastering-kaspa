// Adapted from silverscript-lang/tests/examples_tests.rs and tests/common.rs
// at 3ed973335b59269293564805cc2c58a14595ec03; modified for the book's 2-of-2 source.
// ISC License
// Copyright 2026 Kaspa Developers
// Copyright 2026 shiga0 and contributors
// Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the above copyright notice and this permission notice appear in all copies.
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use kaspa_consensus_core::hashing::sighash::{
    SigHashReusedValuesUnsync, calc_schnorr_signature_hash,
};
use kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use kaspa_consensus_core::mass::units::SigopCount;
use kaspa_consensus_core::tx::{
    MutableTransaction, ScriptPublicKey, Transaction, TransactionId, TransactionInput,
    TransactionOutpoint, TransactionOutput, UtxoEntry, VerifiableTransaction,
};
use kaspa_txscript::caches::Cache;
use kaspa_txscript::{EngineCtx, EngineFlags, TxScriptEngine};
use secp256k1::{Keypair, Message, Secp256k1};
use silverscript_abi::encode_contract_entry_sig_script;
use silverscript_lang::compiler::{CompileOptions, compile_to_sil_abi_artifact_with_options};

fn accepts(transaction: Transaction, previous: UtxoEntry) -> bool {
    let transaction = MutableTransaction::with_entries(transaction, vec![previous.clone()]);
    let populated = transaction.as_verifiable();
    let reused = SigHashReusedValuesUnsync::new();
    let cache = Cache::new(100);
    TxScriptEngine::from_transaction_input(
        &populated,
        &populated.inputs()[0],
        0,
        &previous,
        EngineCtx::new(&cache).with_reused(&reused),
        EngineFlags {
            covenants_enabled: true,
            ..Default::default()
        },
    )
    .execute()
    .is_ok()
}

fn main() {
    let secp = Secp256k1::new();
    let owner_a = Keypair::new(&secp, &mut rand::thread_rng());
    let owner_b = Keypair::new(&secp, &mut rand::thread_rng());
    let public_a = owner_a.x_only_public_key().0.serialize();
    let public_b = owner_b.x_only_public_key().0.serialize();
    assert_ne!(public_a, public_b);

    // ANCHOR: compile-vault
    let artifact = compile_to_sil_abi_artifact_with_options(
        include_str!("two-of-two.sil"),
        &[public_a.to_vec().into(), public_b.to_vec().into()],
        CompileOptions::default(),
    )
    .expect("compile TwoOfTwoVault");
    let script = artifact.contracts["TwoOfTwoVault"]
        .compiled
        .bytecode
        .clone();
    // ANCHOR_END: compile-vault
    let input = TransactionInput {
        previous_outpoint: TransactionOutpoint {
            transaction_id: TransactionId::from_bytes([9; 32]),
            index: 0,
        },
        signature_script: vec![],
        sequence: 0,
        compute_commit: SigopCount(2).into(),
    };
    let output = TransactionOutput {
        value: 900,
        script_public_key: ScriptPublicKey::new(0, script.clone().into()),
        covenant: None,
    };
    let transaction = Transaction::new(
        1,
        vec![input],
        vec![output],
        0,
        Default::default(),
        0,
        vec![],
    );
    let previous = UtxoEntry::new(
        1_000,
        ScriptPublicKey::new(0, script.clone().into()),
        0,
        false,
        None,
    );
    // ANCHOR: sign-vault
    let populated = MutableTransaction::with_entries(transaction.clone(), vec![previous.clone()]);
    let hash = calc_schnorr_signature_hash(
        &populated.as_verifiable(),
        0,
        SIG_HASH_ALL,
        &SigHashReusedValuesUnsync::new(),
    );
    let message = Message::from_digest_slice(hash.as_bytes().as_slice()).expect("32-byte hash");
    let signature = |key: &Keypair| {
        let mut value = key.sign_schnorr(message).as_ref().to_vec();
        value.push(SIG_HASH_ALL.to_u8());
        value
    };
    let signature_a = signature(&owner_a);
    let signature_b = signature(&owner_b);
    // ANCHOR_END: sign-vault

    let mut cases = Vec::new();
    let mut run = |name: &str, a: Vec<u8>, b: Vec<u8>, change_output: bool, expected: bool| {
        let mut spending = transaction.clone();
        let encoded = encode_contract_entry_sig_script(
            &artifact,
            "TwoOfTwoVault",
            "spend",
            &[a.into(), b.into()],
        );
        let sigscript = match encoded {
            Ok(value) => value,
            Err(silverscript_abi::CodecError::InvalidLength {
                name: argument,
                expected: 65,
                actual: 0,
            }) if name == "missing-second" && argument == "signatureB" && !expected => {
                cases.push(serde_json::json!({"case": name, "accepted": false, "stage": "abi"}));
                return;
            }
            Err(_) => panic!("unexpected ABI error for {name}"),
        };
        spending.inputs[0].signature_script = sigscript;
        if change_output {
            spending.outputs[0].value -= 1;
        }
        let accepted = accepts(spending, previous.clone());
        assert_eq!(accepted, expected, "unexpected VM result for {name}");
        cases.push(serde_json::json!({"case": name, "accepted": accepted, "stage": "vm"}));
    };
    run(
        "both-valid",
        signature_a.clone(),
        signature_b.clone(),
        false,
        true,
    );
    run("missing-second", signature_a.clone(), vec![], false, false);
    run(
        "same-signer-twice",
        signature_a.clone(),
        signature_a.clone(),
        false,
        false,
    );
    run(
        "swapped-signatures",
        signature_b.clone(),
        signature_a.clone(),
        false,
        false,
    );
    run(
        "changed-output-after-signing",
        signature_a.clone(),
        signature_b.clone(),
        true,
        false,
    );
    let mut corrupted = signature_a;
    corrupted[0] ^= 1;
    run("corrupted-signature", corrupted, signature_b, false, false);
    println!(
        "{}",
        serde_json::json!({"cases": cases, "compiledScriptBytes": script.len(), "allChecksPassed": true})
    );
}
