// VM setup follows the ISC-licensed upstream tests cited in main.rs.
// ISC License
// Copyright 2026 Kaspa Developers
// Copyright 2026 shiga0 and contributors
// Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the above copyright notice and this permission notice appear in all copies.
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use super::*;
use kaspa_consensus_core::Hash;
use kaspa_consensus_core::tx::CovenantBinding;
use kaspa_txscript::covenants::CovenantsContext;
use kaspa_txscript::pay_to_script_hash_script;
use kaspa_txscript::script_builder::ScriptBuilder;
use silverscript_abi::{ArtifactValue, SilAbiArtifact};

fn compile(source: &str, args: &[ArtifactValue]) -> SilAbiArtifact {
    compile_to_sil_abi_artifact_with_options(source, args, CompileOptions::default())
        .expect("compile book security example")
}

fn code(artifact: &SilAbiArtifact) -> Vec<u8> {
    assert_eq!(artifact.contracts.len(), 1);
    artifact
        .contracts
        .values()
        .next()
        .unwrap()
        .compiled
        .bytecode
        .clone()
}

fn transaction(output: ScriptPublicKey, lock_time: u64) -> Transaction {
    let input = TransactionInput {
        previous_outpoint: TransactionOutpoint {
            transaction_id: TransactionId::from_bytes([42; 32]),
            index: 0,
        },
        signature_script: vec![],
        sequence: 0,
        compute_commit: SigopCount(1).into(),
    };
    Transaction::new(
        1,
        vec![input],
        vec![TransactionOutput {
            value: 1_000,
            script_public_key: output,
            covenant: None,
        }],
        lock_time,
        Default::default(),
        0,
        vec![],
    )
}

fn evaluate(tx: Transaction, previous: UtxoEntry) -> (&'static str, bool) {
    let populated = MutableTransaction::with_entries(tx, vec![previous]);
    let verifiable = populated.as_verifiable();
    let context = match CovenantsContext::from_tx(&verifiable) {
        Ok(context) => context,
        Err(_) => return ("covenant-context", false),
    };
    let cache = Cache::new(100);
    let reused = SigHashReusedValuesUnsync::new();
    let result = TxScriptEngine::from_transaction_input(
        &verifiable,
        &verifiable.inputs()[0],
        0,
        verifiable.utxo(0).unwrap(),
        EngineCtx::new(&cache)
            .with_reused(&reused)
            .with_covenants_ctx(&context),
        EngineFlags {
            covenants_enabled: true,
            ..Default::default()
        },
    )
    .execute();
    ("vm", result.is_ok())
}

fn record(cases: &mut Vec<serde_json::Value>, name: &str, outcome: (&str, bool), expected: bool) {
    assert_eq!(
        outcome.1, expected,
        "unexpected security result: {name} ({})",
        outcome.0
    );
    cases.push(serde_json::json!({"case": name, "stage": outcome.0, "accepted": outcome.1}));
}

pub fn run() -> Vec<serde_json::Value> {
    let mut cases = Vec::new();
    let secp = Secp256k1::new();
    let sender = Keypair::new(&secp, &mut rand::thread_rng());
    let recipient = Keypair::new(&secp, &mut rand::thread_rng());
    let deadline = 1_700_000_000_000i64; // Unix milliseconds; not seconds or DAA score.
    let refund = compile(
        include_str!("timeout-refund.sil"),
        &[
            sender.x_only_public_key().0.serialize().to_vec().into(),
            recipient.x_only_public_key().0.serialize().to_vec().into(),
            deadline.into(),
        ],
    );
    let refund_code = code(&refund);
    let spk = pay_to_script_hash_script(&refund_code);
    let previous = UtxoEntry::new(1_000, spk.clone(), 0, false, None);
    for (name, entry, signer, time, mutate, expected) in [
        ("claim-recipient", "claim", &recipient, 0, false, true),
        ("claim-wrong-signer", "claim", &sender, 0, false, false),
        (
            "refund-before-deadline",
            "refund",
            &sender,
            deadline as u64 - 1,
            false,
            false,
        ),
        (
            "refund-at-deadline",
            "refund",
            &sender,
            deadline as u64,
            false,
            true,
        ),
        (
            "refund-seconds-as-milliseconds",
            "refund",
            &sender,
            deadline as u64 / 1000,
            false,
            false,
        ),
        (
            "refund-wrong-signer",
            "refund",
            &recipient,
            deadline as u64,
            false,
            false,
        ),
        (
            "refund-output-changed-after-signing",
            "refund",
            &sender,
            deadline as u64,
            true,
            false,
        ),
        (
            "claim-still-possible-after-deadline",
            "claim",
            &recipient,
            deadline as u64 + 1,
            false,
            true,
        ),
    ] {
        let mut tx = transaction(spk.clone(), time);
        let populated = MutableTransaction::with_entries(tx.clone(), vec![previous.clone()]);
        let digest = calc_schnorr_signature_hash(
            &populated.as_verifiable(),
            0,
            SIG_HASH_ALL,
            &SigHashReusedValuesUnsync::new(),
        );
        let message = Message::from_digest_slice(&digest.as_bytes()).unwrap();
        let mut signature = signer.sign_schnorr(message).as_ref().to_vec();
        signature.push(SIG_HASH_ALL.to_u8());
        tx.inputs[0].signature_script =
            encode_contract_entry_sig_script(&refund, "TimeoutRefund", entry, &[signature.into()])
                .unwrap();
        tx.inputs[0]
            .signature_script
            .extend_from_slice(&ScriptBuilder::new().add_data(&refund_code).unwrap().drain());
        if mutate {
            tx.outputs[0].value -= 1;
        }
        record(&mut cases, name, evaluate(tx, previous.clone()), expected);
    }

    for (contract, source, bound) in [
        ("Counter", include_str!("counter.sil"), false),
        ("BoundCounter", include_str!("bound-counter.sil"), true),
    ] {
        let current = compile(source, &[0i64.into()]);
        let correct = compile(source, &[1i64.into()]);
        let wrong = compile(source, &[2i64.into()]);
        let current_code = code(&current);
        let lineage = Hash::from_bytes([65; 32]);
        let other = Hash::from_bytes([66; 32]);
        for (mutation, expected) in [
            ("valid-successor", true),
            ("wrong-state", false),
            ("wrong-program", false),
            ("drained-value", false),
            ("extra-output", false),
            ("missing-binding", !bound),
            ("new-lineage-clone", true),
        ] {
            let active_id = if mutation == "new-lineage-clone" {
                other
            } else {
                lineage
            };
            let previous = UtxoEntry::new(
                1_000,
                pay_to_script_hash_script(&current_code),
                0,
                false,
                bound.then_some(active_id),
            );
            let mut tx = transaction(pay_to_script_hash_script(&code(&correct)), 0);
            if bound && mutation != "missing-binding" {
                tx.outputs[0].covenant = Some(CovenantBinding {
                    authorizing_input: 0,
                    covenant_id: active_id,
                });
            }
            match mutation {
                "wrong-state" => {
                    tx.outputs[0].script_public_key = pay_to_script_hash_script(&code(&wrong))
                }
                "wrong-program" => {
                    tx.outputs[0].script_public_key = ScriptPublicKey::new(0, vec![0x51].into())
                }
                "drained-value" => tx.outputs[0].value -= 1,
                "extra-output" => tx.outputs.push(tx.outputs[0].clone()),
                _ => {}
            }
            tx.inputs[0].signature_script =
                encode_contract_entry_sig_script(&current, contract, "step", &[]).unwrap();
            tx.inputs[0].signature_script.extend_from_slice(
                &ScriptBuilder::new()
                    .add_data(&current_code)
                    .unwrap()
                    .drain(),
            );
            record(
                &mut cases,
                &format!("{contract}-{mutation}"),
                evaluate(tx, previous),
                expected,
            );
        }
        if bound {
            for (name, authorizer, output_id) in [
                ("wrong-lineage", 0, other),
                ("out-of-range-authorizer", 1, lineage),
            ] {
                let previous = UtxoEntry::new(
                    1_000,
                    pay_to_script_hash_script(&current_code),
                    0,
                    false,
                    Some(lineage),
                );
                let mut tx = transaction(pay_to_script_hash_script(&code(&correct)), 0);
                tx.outputs[0].covenant = Some(CovenantBinding {
                    authorizing_input: authorizer,
                    covenant_id: output_id,
                });
                tx.inputs[0].signature_script =
                    encode_contract_entry_sig_script(&current, contract, "step", &[]).unwrap();
                tx.inputs[0].signature_script.extend_from_slice(
                    &ScriptBuilder::new()
                        .add_data(&current_code)
                        .unwrap()
                        .drain(),
                );
                record(
                    &mut cases,
                    &format!("{contract}-{name}"),
                    evaluate(tx, previous),
                    false,
                );
            }
        }
    }
    assert_eq!(cases.len(), 24);
    cases
}
