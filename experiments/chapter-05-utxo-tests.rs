// Append to the Rust block extracted from chapter-05.md; see the review record.
#[cfg(test)]
mod tests {
    use super::*;

    fn point(txid: u64, index: u32) -> OutPoint {
        OutPoint { txid, index }
    }

    fn tx(txid: u64, inputs: Vec<OutPoint>, values: &[u64]) -> Transaction {
        Transaction {
            txid,
            inputs,
            outputs: values.iter().map(|&value| Output { value }).collect(),
        }
    }

    fn initial() -> HashMap<OutPoint, Output> {
        HashMap::from([
            (point(1, 0), Output { value: 100 }),
            (point(1, 1), Output { value: 200 }),
            // Model-only fixture for arithmetic overflow, not a valid BTC amount.
            (point(9, 0), Output { value: u64::MAX }),
        ])
    }

    #[test]
    fn spends_once_creates_change_and_preserves_unrelated_outputs() {
        let mut state = initial();
        let payment = tx(2, vec![point(1, 0)], &[30, 69]);
        let mut expected = state.clone();
        expected.remove(&point(1, 0));
        expected.insert(point(2, 0), Output { value: 30 });
        expected.insert(point(2, 1), Output { value: 69 });

        assert_eq!(apply_transaction(&mut state, &payment), Ok(1));
        assert_eq!(state, expected);
        assert_eq!(
            apply_transaction(&mut state, &payment),
            Err(ApplyError::MissingInput)
        );
        assert_eq!(state, expected);
    }

    #[test]
    fn rejected_transactions_leave_the_entire_utxo_set_unchanged() {
        let cases = [
            ("empty inputs", tx(2, vec![], &[1]), ApplyError::EmptyInputs),
            (
                "empty outputs",
                tx(2, vec![point(1, 0)], &[]),
                ApplyError::EmptyOutputs,
            ),
            (
                "missing input",
                tx(2, vec![point(5, 0)], &[1]),
                ApplyError::MissingInput,
            ),
            (
                "duplicate input",
                tx(2, vec![point(1, 0), point(1, 0)], &[1]),
                ApplyError::DuplicateInput,
            ),
            (
                "input sum overflow",
                tx(2, vec![point(9, 0), point(1, 0)], &[1]),
                ApplyError::ValueOverflow,
            ),
            (
                "output sum overflow",
                tx(2, vec![point(1, 0)], &[u64::MAX, 1]),
                ApplyError::ValueOverflow,
            ),
            (
                "overspend",
                tx(2, vec![point(1, 0)], &[101]),
                ApplyError::Overspend,
            ),
            (
                "overwrite unrelated output",
                tx(1, vec![point(1, 1)], &[50]),
                ApplyError::OutputAlreadyExists,
            ),
            (
                "recreate consumed outpoint",
                tx(1, vec![point(1, 0)], &[99]),
                ApplyError::OutputAlreadyExists,
            ),
        ];

        for (name, transaction, error) in cases {
            let mut state = initial();
            let before = state.clone();
            assert_eq!(
                apply_transaction(&mut state, &transaction),
                Err(error),
                "{name}"
            );
            assert_eq!(state, before, "{name}");
        }
    }
}
