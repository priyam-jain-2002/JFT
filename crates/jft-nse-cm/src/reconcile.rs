//! Replay, trade-reference, and independent-summary reconciliation.

use std::collections::BTreeMap;

use crate::book::{BookProjectionError, OrderBookProjection};
use crate::events::{OrderEvent, Side, TradeEvent};
use crate::framing::FramingError;
use crate::jiffy::ExchangeJiffy;
use crate::layouts::TRIM_ORDER_RECORD_LEN;
use crate::lifecycle::{LifecycleAnomaly, LifecycleTransition, OrderKey, OrderLifecycle};
use crate::parser::{LocatedParse, parse_trim_order_source, parse_trim_trade_source};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderEntryEvidence {
    pub side: Side,
    pub coordinate: SourceCoordinate,
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
pub struct LocatedLifecycleTransition {
    pub coordinate: SourceCoordinate,
    pub event: OrderEvent,
    pub raw: [u8; TRIM_ORDER_RECORD_LEN],
    pub transition: LifecycleTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOrderReplay<'a> {
    pub lifecycle: OrderLifecycle,
    pub active_provenance: BTreeMap<OrderKey, ActiveOrderProvenance>,
    pub order_entries: BTreeMap<OrderKey, OrderEntryEvidence>,
    pub framed_records: usize,
    pub parsed_records: usize,
    pub applied_events: usize,
    pub quarantines: Vec<LocatedQuarantine>,
    pub lifecycle_anomalies: Vec<LocatedLifecycleAnomaly>,
    pub framing_error: Option<FramingError<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSourceReference {
    EntryMatch {
        key: OrderKey,
        entry: OrderEntryEvidence,
    },
    EntrySideMismatch {
        key: OrderKey,
        expected_side: Side,
        entry: OrderEntryEvidence,
    },
    Missing {
        key: OrderKey,
        expected_side: Side,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeOrderSourceReferences {
    pub buy: OrderSourceReference,
    pub sell: OrderSourceReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JiffyRegression {
    pub previous_coordinate: SourceCoordinate,
    pub previous_jiffy: ExchangeJiffy,
    pub current_coordinate: SourceCoordinate,
    pub current_jiffy: ExchangeJiffy,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JiffySequenceProfile {
    pub observation_count: usize,
    pub minimum: Option<ExchangeJiffy>,
    pub maximum: Option<ExchangeJiffy>,
    pub equal_consecutive_observations: usize,
    pub largest_equal_run: usize,
    pub regressions: Vec<JiffyRegression>,
}

impl JiffySequenceProfile {
    #[must_use]
    pub fn range_overlaps(&self, other: &Self) -> bool {
        match (self.minimum, self.maximum, other.minimum, other.maximum) {
            (Some(left_min), Some(left_max), Some(right_min), Some(right_max)) => {
                left_min <= right_max && right_min <= left_max
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiffySourceProfile<'a> {
    pub sequence: JiffySequenceProfile,
    pub framed_records: usize,
    pub parsed_records: usize,
    pub quarantines: Vec<LocatedQuarantine>,
    pub framing_error: Option<FramingError<'a>>,
}

impl JiffySourceProfile<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.parsed_records.checked_add(self.quarantines.len()) == Some(self.framed_records)
            && self.sequence.observation_count == self.parsed_records
    }
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
    replay_trim_order_source_with_observer(source, |_| {})
}

#[must_use]
pub fn replay_trim_order_source_with_observer(
    source: &[u8],
    mut observe: impl FnMut(LocatedLifecycleTransition),
) -> TrimOrderReplay<'_> {
    let mut replay = TrimOrderReplay {
        lifecycle: OrderLifecycle::new(),
        active_provenance: BTreeMap::new(),
        order_entries: BTreeMap::new(),
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
                    &mut replay.order_entries,
                    key,
                    coordinate,
                    transition,
                );
                observe(LocatedLifecycleTransition {
                    coordinate,
                    event: parsed.event,
                    raw: parsed.raw,
                    transition,
                });
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
    entries: &mut BTreeMap<OrderKey, OrderEntryEvidence>,
    key: OrderKey,
    coordinate: SourceCoordinate,
    transition: LifecycleTransition,
) {
    match transition {
        LifecycleTransition::Entered { current } => {
            provenance.insert(
                key,
                ActiveOrderProvenance {
                    entry: coordinate,
                    last_mutation: coordinate,
                },
            );
            entries.insert(
                key,
                OrderEntryEvidence {
                    side: current.side,
                    coordinate,
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

#[must_use]
pub fn reconcile_trim_trade_references(
    orders: &TrimOrderReplay<'_>,
    trade: &TradeEvent,
) -> TradeOrderSourceReferences {
    TradeOrderSourceReferences {
        buy: reconcile_order_source_reference(orders, trade, trade.buy_order_number, Side::Buy),
        sell: reconcile_order_source_reference(orders, trade, trade.sell_order_number, Side::Sell),
    }
}

fn reconcile_order_source_reference(
    orders: &TrimOrderReplay<'_>,
    trade: &TradeEvent,
    order_number: u64,
    expected_side: Side,
) -> OrderSourceReference {
    let key = OrderKey {
        session: trade.session,
        segment: trade.segment,
        symbol_raw: trade.symbol_raw,
        series_raw: trade.series_raw,
        order_number,
    };
    match orders.order_entries.get(&key).copied() {
        Some(entry) if entry.side == expected_side => {
            OrderSourceReference::EntryMatch { key, entry }
        }
        Some(entry) => OrderSourceReference::EntrySideMismatch {
            key,
            expected_side,
            entry,
        },
        None => OrderSourceReference::Missing { key, expected_side },
    }
}

#[must_use]
pub fn profile_trim_order_jiffies(source: &[u8]) -> JiffySourceProfile<'_> {
    profile_jiffies(parse_trim_order_source(source), |event| {
        event.exchange_jiffy
    })
}

#[must_use]
pub fn profile_trim_trade_jiffies(source: &[u8]) -> JiffySourceProfile<'_> {
    profile_jiffies(parse_trim_trade_source(source), |event| {
        event.exchange_jiffy
    })
}

fn profile_jiffies<'a, T, const N: usize>(
    records: impl Iterator<Item = Result<LocatedParse<T, N>, FramingError<'a>>>,
    exchange_jiffy: impl Fn(&T) -> ExchangeJiffy,
) -> JiffySourceProfile<'a> {
    let mut profile = JiffySourceProfile {
        sequence: JiffySequenceProfile::default(),
        framed_records: 0,
        parsed_records: 0,
        quarantines: Vec::new(),
        framing_error: None,
    };
    let mut previous: Option<(SourceCoordinate, ExchangeJiffy)> = None;
    let mut current_equal_run = 0_usize;

    for outcome in records {
        let located = match outcome {
            Ok(located) => located,
            Err(error) => {
                profile.framing_error = Some(error);
                break;
            }
        };
        profile.framed_records += 1;
        let coordinate = SourceCoordinate {
            record_index: located.record_index,
            byte_offset: located.byte_offset,
        };
        let parsed = match located.result {
            Ok(parsed) => parsed,
            Err(quarantined) => {
                profile.quarantines.push(LocatedQuarantine {
                    coordinate,
                    quarantined,
                });
                continue;
            }
        };
        profile.parsed_records += 1;
        let current = exchange_jiffy(&parsed.event);
        profile.sequence.observation_count += 1;
        profile.sequence.minimum = Some(
            profile
                .sequence
                .minimum
                .map_or(current, |minimum| minimum.min(current)),
        );
        profile.sequence.maximum = Some(
            profile
                .sequence
                .maximum
                .map_or(current, |maximum| maximum.max(current)),
        );

        if let Some((previous_coordinate, previous_jiffy)) = previous {
            if current == previous_jiffy {
                profile.sequence.equal_consecutive_observations += 1;
                current_equal_run += 1;
            } else {
                current_equal_run = 1;
            }
            if current < previous_jiffy {
                profile.sequence.regressions.push(JiffyRegression {
                    previous_coordinate,
                    previous_jiffy,
                    current_coordinate: coordinate,
                    current_jiffy: current,
                });
            }
        } else {
            current_equal_run = 1;
        }
        profile.sequence.largest_equal_run =
            profile.sequence.largest_equal_run.max(current_equal_run);
        previous = Some((coordinate, current));
    }

    profile
}
