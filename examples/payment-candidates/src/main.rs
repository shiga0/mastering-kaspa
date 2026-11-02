use payment_candidates::{Invoice, Network, Observation, ObservationState, OutputId};
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    network: String,
    address: String,
    outputs: Vec<Output>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    transaction_id: String,
    output_index: u32,
    amount_sompi: String,
}

fn amount(text: &str) -> Result<u64, String> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("amount must be a decimal integer string".into());
    }
    text.parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "amount must be in 1..=u64::MAX".into())
}

fn transaction_id(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("transaction_id must contain exactly 64 hexadecimal digits".into());
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| "invalid transaction_id")?;
    }
    Ok(bytes)
}

fn import(json: &str, assigned_address: &str, requested: u64) -> Result<serde_json::Value, String> {
    let snapshot: Snapshot = serde_json::from_str(json).map_err(|error| error.to_string())?;
    if snapshot.network != "devnet" {
        return Err("expected devnet".into());
    }
    // Association comes from the invoice configuration, not an arbitrary RPC result.
    if !assigned_address.starts_with("kaspadev:") || snapshot.address != assigned_address {
        return Err("snapshot does not match the invoice's assigned Devnet address".into());
    }
    let mut invoice = Invoice::new(Network::Devnet, requested).map_err(|e| format!("{e:?}"))?;
    let mut seen = HashSet::new();
    for output in snapshot.outputs {
        // ANCHOR: candidate-observation
        let observation = Observation {
            network: Network::Devnet,
            revision: 0, // Only an initial import into a fresh, in-memory invoice.
            id: OutputId {
                tx: transaction_id(&output.transaction_id)?,
                index: output.output_index,
            },
            amount_sompi: amount(&output.amount_sompi)?,
            state: ObservationState::Candidate,
        };
        invoice.observe(observation).map_err(|e| format!("{e:?}"))?;
        // ANCHOR_END: candidate-observation
        seen.insert(observation.id);
    }
    Ok(serde_json::json!({
        "network": "devnet",
        "candidateOutputCount": seen.len(),
        "invoiceStatus": format!("{:?}", invoice.status().map_err(|e| format!("{e:?}"))?),
        "eligibleTotalSompi": invoice.eligible_total().map_err(|e| format!("{e:?}"))?.to_string(),
    }))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(
            "usage: payment-candidates <assigned-devnet-address> <invoice-amount-sompi>".into(),
        );
    }
    let requested = amount(&args[2])?;
    // Limit transport input before JSON allocation; this is a small snapshot demo.
    let mut json = String::new();
    io::stdin()
        .take(1_048_577)
        .read_to_string(&mut json)
        .map_err(|e| e.to_string())?;
    if json.len() > 1_048_576 {
        return Err("snapshot exceeds 1 MiB".into());
    }
    println!("{}", import(&json, &args[1], requested)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const ADDRESS: &str = "kaspadev:assigned-in-test";

    fn output(value: &str) -> serde_json::Value {
        serde_json::json!({"transaction_id": "ab".repeat(32), "output_index": 2, "amount_sompi": value})
    }
    fn snapshot(outputs: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({"network":"devnet", "address": ADDRESS, "outputs":outputs})
    }

    #[test]
    fn actual_shape_and_duplicate_outputs_stay_candidates() {
        let result = import(
            &snapshot(vec![output("100000000"), output("100000000")]).to_string(),
            ADDRESS,
            1,
        )
        .unwrap();
        assert_eq!(result["candidateOutputCount"], 1);
        assert_eq!(result["invoiceStatus"], "Observed");
        assert_eq!(result["eligibleTotalSompi"], "0");
    }

    #[test]
    fn empty_snapshot_is_pending() {
        assert_eq!(
            import(&snapshot(vec![]).to_string(), ADDRESS, 1).unwrap()["invoiceStatus"],
            "Pending"
        );
    }

    #[test]
    fn recorded_official_sdk_snapshot_is_observed_without_eligible_credit() {
        let text = include_str!("../fixtures/devnet-snapshot.json");
        let snapshot: Snapshot = serde_json::from_str(text).unwrap();
        assert!(!snapshot.outputs.is_empty());
        let result = import(text, &snapshot.address, 100_000_000).unwrap();
        assert_eq!(result["candidateOutputCount"], snapshot.outputs.len());
        assert_eq!(result["invoiceStatus"], "Observed");
        assert_eq!(result["eligibleTotalSompi"], "0");
    }

    #[test]
    fn wrong_network_and_wrong_invoice_address_are_rejected() {
        let mut value = snapshot(vec![output("1")]);
        value["network"] = "mainnet".into();
        assert!(import(&value.to_string(), ADDRESS, 1).is_err());
        value["network"] = "devnet".into();
        value["address"] = "kaspadev:another-invoice".into();
        assert!(import(&value.to_string(), ADDRESS, 1).is_err());
    }

    #[test]
    fn malformed_ids_indices_and_amounts_are_rejected() {
        for field in ["transaction_id", "output_index", "amount_sompi"] {
            let mut entry = output("1");
            entry[field] = match field {
                "output_index" => serde_json::json!(4294967296_u64),
                _ => "invalid".into(),
            };
            assert!(import(&snapshot(vec![entry]).to_string(), ADDRESS, 1).is_err());
        }
        for value in ["0", "-1", "+1", "1.5", "18446744073709551616"] {
            assert!(import(&snapshot(vec![output(value)]).to_string(), ADDRESS, 1).is_err());
        }
        assert_eq!(amount("18446744073709551615").unwrap(), u64::MAX);
    }

    #[test]
    fn conflicting_duplicate_is_not_silently_overwritten() {
        assert!(import(
            &snapshot(vec![output("1"), output("2")]).to_string(),
            ADDRESS,
            1
        )
        .is_err());
    }

    #[test]
    fn transport_cannot_supply_eligibility_or_revision() {
        for (field, value) in [
            ("state", serde_json::json!("Eligible")),
            ("revision", serde_json::json!(9)),
        ] {
            let mut entry = output("1");
            entry[field] = value;
            assert!(import(&snapshot(vec![entry]).to_string(), ADDRESS, 1).is_err());
        }
    }
}
