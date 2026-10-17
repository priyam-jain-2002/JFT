//! Replay, trade-reference, and independent-summary reconciliation.

use std::collections::BTreeMap;

use crate::book::{BookProjectionError, OrderBookProjection};
use crate::events::OrderEvent;
use crate::framing::FramingError;
use crate::layouts::TRIM_ORDER_RECORD_LEN;
use crate::lifecycle::{LifecycleAnomaly, LifecycleTransition, OrderKey, OrderLifecycle};
use crate::parser::parse_trim_order_source;
use crate::quarantine::QuarantinedRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceCoordinate {
    pub record_index: usize,
    pub byte_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveOrderProvenance {
    pub entry: SourceCoordinate,
    pub last_mutation: SourceCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedQuarantine {
    pub coordinate: SourceCoordinate,
    pub quarantined: QuarantinedRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedLifecycleAnomaly {
    pub coordinate: SourceCoordinate,
    pub event: OrderEvent,
    pub raw: [u8; TRIM_ORDER_RECORD_LEN],
    pub anomaly: LifecycleAnomaly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOrderReplay<'a> {
    pub lifecycle: OrderLifecycle,
    pub active_provenance: BTreeMap<OrderKey, ActiveOrderProvenance>,
    pub framed_records: usize,
    pub parsed_records: usize,
    pub applied_events: usize,
    pub quarantines: Vec<LocatedQuarantine>,
    pub lifecycle_anomalies: Vec<LocatedLifecycleAnomaly>,
    pub framing_error: Option<FramingError<'a>>,
}

impl TrimOrderReplay<'_> {
    pub fn book_projection(&self) -> Result<OrderBookProjection, BookProjectionError> {
        OrderBookProjection::from_lifecycle(&self.lifecycle)
    }

    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.parsed_records.checked_add(self.quarantines.len()) == Some(self.framed_records)
            && self
                .applied_events
                .checked_add(self.lifecycle_anomalies.len())
                == Some(self.parsed_records)
    }
}

#[must_use]
pub fn replay_trim_order_source(source: &[u8]) -> TrimOrderReplay<'_> {
    let mut replay = TrimOrderReplay {
        lifecycle: OrderLifecycle::new(),
        active_provenance: BTreeMap::new(),
        framed_records: 0,
        parsed_records: 0,
        applied_events: 0,
        quarantines: Vec::new(),
        lifecycle_anomalies: Vec::new(),
        framing_error: None,
    };

    for outcome in parse_trim_order_source(source) {
        let located = match outcome {
            Ok(located) => located,
            Err(error) => {
                replay.framing_error = Some(error);
                break;
            }
        };
        replay.framed_records += 1;
        let coordinate = SourceCoordinate {
            record_index: located.record_index,
            byte_offset: located.byte_offset,
        };

        let parsed = match located.result {
            Ok(parsed) => parsed,
            Err(quarantined) => {
                replay.quarantines.push(LocatedQuarantine {
                    coordinate,
                    quarantined,
                });
                continue;
            }
        };
        replay.parsed_records += 1;
        let key = OrderKey::from(&parsed.event);

        match replay.lifecycle.apply(&parsed.event) {
            Ok(transition) => {
                replay.applied_events += 1;
                update_active_provenance(
                    &mut replay.active_provenance,
                    key,
                    coordinate,
                    transition,
                );
            }
            Err(anomaly) => replay.lifecycle_anomalies.push(LocatedLifecycleAnomaly {
                coordinate,
                event: parsed.event,
                raw: parsed.raw,
                anomaly,
            }),
        }
    }

    replay
}

fn update_active_provenance(
    provenance: &mut BTreeMap<OrderKey, ActiveOrderProvenance>,
    key: OrderKey,
    coordinate: SourceCoordinate,
    transition: LifecycleTransition,
) {
    match transition {
        LifecycleTransition::Entered { .. } => {
            provenance.insert(
                key,
                ActiveOrderProvenance {
                    entry: coordinate,
                    last_mutation: coordinate,
                },
            );
        }
        LifecycleTransition::Modified { .. } => {
            provenance
                .get_mut(&key)
                .expect("successful modify must have entry provenance")
                .last_mutation = coordinate;
        }
        LifecycleTransition::Cancelled { .. } => {
            provenance.remove(&key);
        }
    }
}
