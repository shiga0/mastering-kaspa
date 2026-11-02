use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet(u32),
    Devnet,
    Simnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputId {
    pub tx: [u8; 32],
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationState {
    Candidate,
    Eligible,
    Retracted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Observation {
    pub network: Network,
    pub revision: u64,
    pub id: OutputId,
    pub amount_sompi: u64,
    pub state: ObservationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceStatus {
    Pending,
    Observed,
    Paid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentError {
    ZeroInvoiceAmount,
    ConflictingObservation,
    AmountOverflow,
    WrongNetwork,
    ZeroOutputAmount,
    StaleObservation,
    ConflictingRevision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoice {
    network: Network,
    amount_sompi: u64,
    outputs: HashMap<OutputId, Observation>,
}

impl Invoice {
    pub fn new(network: Network, amount_sompi: u64) -> Result<Self, PaymentError> {
        if amount_sompi == 0 {
            return Err(PaymentError::ZeroInvoiceAmount);
        }
        Ok(Self {
            network,
            amount_sompi,
            outputs: HashMap::new(),
        })
    }

    // 同じ出力の同じ版・同じ内容は再処理しても結果が変わらない。
    // 同じ出力IDに異なる金額が届いた場合はデータ不整合として拒否する。
    pub fn observe(&mut self, observation: Observation) -> Result<(), PaymentError> {
        if observation.network != self.network {
            return Err(PaymentError::WrongNetwork);
        }
        if observation.amount_sompi == 0 {
            return Err(PaymentError::ZeroOutputAmount);
        }
        if let Some(existing) = self.outputs.get(&observation.id) {
            if existing.amount_sompi != observation.amount_sompi {
                return Err(PaymentError::ConflictingObservation);
            }
            if observation.revision < existing.revision {
                return Err(PaymentError::StaleObservation);
            }
            if observation.revision == existing.revision {
                return if *existing == observation {
                    Ok(())
                } else {
                    Err(PaymentError::ConflictingRevision)
                };
            }
        }
        let mut next = self.clone();
        next.outputs.insert(observation.id, observation);
        next.eligible_total()?;
        *self = next;
        Ok(())
    }

    pub fn eligible_total(&self) -> Result<u64, PaymentError> {
        self.outputs
            .values()
            .filter(|o| o.state == ObservationState::Eligible)
            .try_fold(0_u64, |total, o| {
                total
                    .checked_add(o.amount_sompi)
                    .ok_or(PaymentError::AmountOverflow)
            })
    }

    pub fn status(&self) -> Result<InvoiceStatus, PaymentError> {
        if self.eligible_total()? >= self.amount_sompi {
            Ok(InvoiceStatus::Paid)
        } else if self
            .outputs
            .values()
            .any(|o| o.state != ObservationState::Retracted)
        {
            Ok(InvoiceStatus::Observed)
        } else {
            Ok(InvoiceStatus::Pending)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(state: ObservationState, amount: u64) -> Observation {
        Observation {
            network: Network::Testnet(11),
            revision: 0,
            id: OutputId {
                tx: [1; 32],
                index: 0,
            },
            amount_sompi: amount,
            state,
        }
    }

    #[test]
    fn zero_invoice_amount_is_rejected() {
        assert!(matches!(
            Invoice::new(Network::Testnet(11), 0),
            Err(PaymentError::ZeroInvoiceAmount)
        ));
    }

    #[test]
    fn duplicate_notification_is_idempotent() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        let output = sample(ObservationState::Eligible, 100);
        invoice.observe(output).unwrap();
        invoice.observe(output).unwrap();
        assert_eq!(invoice.eligible_total().unwrap(), 100);
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Paid);
    }

    #[test]
    fn candidate_is_not_paid() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        invoice
            .observe(sample(ObservationState::Candidate, 100))
            .unwrap();
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Observed);
    }

    #[test]
    fn retraction_recalculates_status() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        invoice
            .observe(sample(ObservationState::Eligible, 100))
            .unwrap();
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Paid);
        invoice
            .observe(Observation {
                revision: 1,
                ..sample(ObservationState::Retracted, 100)
            })
            .unwrap();
        assert_eq!(invoice.status().unwrap(), InvoiceStatus::Pending);
    }

    #[test]
    fn conflicting_amount_is_rejected() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        invoice
            .observe(sample(ObservationState::Eligible, 100))
            .unwrap();
        let error = invoice.observe(sample(ObservationState::Eligible, 200));
        assert_eq!(error, Err(PaymentError::ConflictingObservation));
        assert_eq!(invoice.eligible_total().unwrap(), 100);
    }

    #[test]
    fn amount_overflow_is_detected() {
        let mut invoice = Invoice::new(Network::Testnet(11), 1).unwrap();
        invoice
            .observe(sample(ObservationState::Eligible, u64::MAX))
            .unwrap();
        let before = invoice.clone();
        assert_eq!(
            invoice.observe(Observation {
                id: OutputId {
                    tx: [2; 32],
                    index: 0
                },
                ..sample(ObservationState::Eligible, 1)
            }),
            Err(PaymentError::AmountOverflow)
        );
        assert_eq!(invoice, before);
        assert_eq!(invoice.eligible_total(), Ok(u64::MAX));
    }

    #[test]
    fn wrong_network_including_testnet_suffix_preserves_state() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        let before = invoice.clone();
        for network in [Network::Mainnet, Network::Testnet(10), Network::Devnet] {
            assert_eq!(
                invoice.observe(Observation {
                    network,
                    ..sample(ObservationState::Eligible, 100)
                }),
                Err(PaymentError::WrongNetwork)
            );
            assert_eq!(invoice, before);
        }
    }

    #[test]
    fn stale_eligible_output_cannot_undo_retraction() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        let original = sample(ObservationState::Eligible, 100);
        invoice.observe(original).unwrap();
        invoice
            .observe(Observation {
                revision: 1,
                state: ObservationState::Retracted,
                ..original
            })
            .unwrap();
        let before = invoice.clone();
        assert_eq!(
            invoice.observe(original),
            Err(PaymentError::StaleObservation)
        );
        assert_eq!(invoice, before);
        assert_eq!(invoice.status(), Ok(InvoiceStatus::Pending));
    }

    #[test]
    fn same_revision_cannot_describe_two_states() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        invoice
            .observe(sample(ObservationState::Candidate, 100))
            .unwrap();
        let before = invoice.clone();
        assert_eq!(
            invoice.observe(sample(ObservationState::Eligible, 100)),
            Err(PaymentError::ConflictingRevision)
        );
        assert_eq!(invoice, before);
    }

    #[test]
    fn newer_revision_can_restore_the_same_output() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        invoice
            .observe(sample(ObservationState::Eligible, 100))
            .unwrap();
        invoice
            .observe(Observation {
                revision: 1,
                ..sample(ObservationState::Retracted, 100)
            })
            .unwrap();
        invoice
            .observe(Observation {
                revision: 2,
                ..sample(ObservationState::Eligible, 100)
            })
            .unwrap();
        assert_eq!(invoice.eligible_total(), Ok(100));
        assert_eq!(invoice.status(), Ok(InvoiceStatus::Paid));
    }

    #[test]
    fn zero_value_output_does_not_mark_invoice_observed() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        let before = invoice.clone();
        assert_eq!(
            invoice.observe(sample(ObservationState::Candidate, 0)),
            Err(PaymentError::ZeroOutputAmount)
        );
        assert_eq!(invoice, before);
    }

    #[test]
    fn partial_outputs_and_repeated_delivery_are_counted_once() {
        let mut invoice = Invoice::new(Network::Testnet(11), 100).unwrap();
        let first = sample(ObservationState::Eligible, 60);
        let second = Observation {
            id: OutputId {
                tx: [2; 32],
                index: 0,
            },
            ..sample(ObservationState::Eligible, 40)
        };
        invoice.observe(first).unwrap();
        assert_eq!(invoice.status(), Ok(InvoiceStatus::Observed));
        for _ in 0..100 {
            invoice.observe(second).unwrap();
        }
        assert_eq!(invoice.eligible_total(), Ok(100));
        assert_eq!(invoice.status(), Ok(InvoiceStatus::Paid));
        invoice
            .observe(Observation {
                revision: 1,
                state: ObservationState::Retracted,
                ..first
            })
            .unwrap();
        assert_eq!(invoice.eligible_total(), Ok(40));
        assert_eq!(invoice.status(), Ok(InvoiceStatus::Observed));
    }
}

#[derive(Debug, Clone)]
pub struct ChainHealth {
    pub connected: bool,
    pub synced: bool,
    pub network: Network,
}

pub trait ChainReader {
    type Error;

    fn health(&self) -> Result<ChainHealth, Self::Error>;
    fn observations_for_address(&self, address: &str) -> Result<Vec<Observation>, Self::Error>;
}
