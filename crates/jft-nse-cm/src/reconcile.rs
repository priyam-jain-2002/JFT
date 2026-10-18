//! Replay, trade-reference, and independent-summary reconciliation.

use std::collections::BTreeMap;

use crate::book::{BookProjectionError, OrderBookProjection};
use crate::events::{OrderEvent, Side, TradeEvent};
use crate::framing::{FixedWidthLfRecords, FramingError, FramingProfile, profile_fixed_width_lf};
use crate::jiffy::ExchangeJiffy;
use crate::layouts::{
    CURRENT_FULL_ORDER_RECORD_LEN, CURRENT_FULL_TRADE_RECORD_LEN, FieldLayout,
    FullOrderLayoutVersion, FullTradeLayoutVersion, LEGACY_FULL_ORDER_RECORD_LEN,
    LEGACY_FULL_TRADE_100_RECORD_LEN, LEGACY_FULL_TRADE_101_RECORD_LEN, SourceDate,
    TRIM_ORDER_RECORD_LEN, full_cm_layout_for_date,
};
use crate::lifecycle::{LifecycleAnomaly, LifecycleTransition, OrderKey, OrderLifecycle};
use crate::manifest::{CmPairedStreamSet, CmProduct, CmRecordKind};
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
pub struct StreamOrderEntryEvidence {
    pub member: Option<u8>,
    pub entry: OrderEntryEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamOrderSourceCandidates {
    pub key: OrderKey,
    pub expected_side: Side,
    pub candidates: Vec<StreamOrderEntryEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeOrderStreamCandidates {
    pub buy: StreamOrderSourceCandidates,
    pub sell: StreamOrderSourceCandidates,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmStreamSource<'a> {
    pub member: Option<u8>,
    pub source: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiffyStreamProfile<'a> {
    pub member: Option<u8>,
    pub source: JiffySourceProfile<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JiffyStreamRangeOverlap {
    pub left_member: Option<u8>,
    pub right_member: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiffyStreamSetProfile<'a> {
    pub streams: Vec<JiffyStreamProfile<'a>>,
    pub range_overlaps: Vec<JiffyStreamRangeOverlap>,
}

impl JiffyStreamSetProfile<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.streams
            .iter()
            .all(|stream| stream.source.counts_reconcile())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmStreamSourceSetError {
    ExpectedTrim {
        actual: CmProduct,
    },
    RecordKindMismatch {
        expected: CmRecordKind,
        actual: CmRecordKind,
    },
    StreamMembersMismatch {
        expected: Vec<Option<u8>>,
        actual: Vec<Option<u8>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOrderStreamReplay<'a> {
    pub member: Option<u8>,
    pub replay: TrimOrderReplay<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOrderStreamSetReplay<'a> {
    pub streams: Vec<TrimOrderStreamReplay<'a>>,
}

impl TrimOrderStreamSetReplay<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.streams
            .iter()
            .all(|stream| stream.replay.counts_reconcile())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldByteProfile {
    pub field: FieldLayout,
    byte_counts: [u64; 256],
}

impl FieldByteProfile {
    #[must_use]
    pub fn count(&self, byte: u8) -> u64 {
        self.byte_counts[usize::from(byte)]
    }

    pub fn observed_bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.byte_counts
            .iter()
            .enumerate()
            .filter_map(|(byte, count)| (*count > 0).then_some(byte as u8))
    }

    #[must_use]
    pub fn total_observations(&self) -> Option<u64> {
        self.byte_counts
            .iter()
            .try_fold(0_u64, |total, count| total.checked_add(*count))
    }

    #[must_use]
    pub fn observed_bytes_are_ascii_digits(&self) -> bool {
        self.total_observations().is_some_and(|total| total > 0)
            && self.observed_bytes().all(|byte| byte.is_ascii_digit())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldByteSourceProfile<'a> {
    pub framing: FramingProfile<'a>,
    pub fields: Vec<FieldByteProfile>,
}

impl FieldByteSourceProfile<'_> {
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&FieldByteProfile> {
        self.fields
            .iter()
            .find(|profile| profile.field.name == name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldByteProfileError {
    FieldOutOfBounds {
        field: FieldLayout,
        payload_width: usize,
    },
    ByteCountOverflow {
        field: FieldLayout,
        byte: u8,
    },
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

#[must_use]
pub fn collect_trim_trade_stream_candidates(
    orders: &TrimOrderStreamSetReplay<'_>,
    trade: &TradeEvent,
) -> TradeOrderStreamCandidates {
    TradeOrderStreamCandidates {
        buy: collect_order_stream_candidates(orders, trade, trade.buy_order_number, Side::Buy),
        sell: collect_order_stream_candidates(orders, trade, trade.sell_order_number, Side::Sell),
    }
}

fn collect_order_stream_candidates(
    orders: &TrimOrderStreamSetReplay<'_>,
    trade: &TradeEvent,
    order_number: u64,
    expected_side: Side,
) -> StreamOrderSourceCandidates {
    let key = OrderKey {
        session: trade.session,
        segment: trade.segment,
        symbol_raw: trade.symbol_raw,
        series_raw: trade.series_raw,
        order_number,
    };
    let candidates =
        orders
            .streams
            .iter()
            .filter_map(|stream| {
                stream.replay.order_entries.get(&key).copied().map(|entry| {
                    StreamOrderEntryEvidence {
                        member: stream.member,
                        entry,
                    }
                })
            })
            .collect();

    StreamOrderSourceCandidates {
        key,
        expected_side,
        candidates,
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

#[must_use]
pub fn profile_full_order_framing(source: &[u8], source_date: SourceDate) -> FramingProfile<'_> {
    match full_cm_layout_for_date(source_date).order {
        FullOrderLayoutVersion::Bytes87 => {
            profile_fixed_width_lf::<LEGACY_FULL_ORDER_RECORD_LEN>(source)
        }
        FullOrderLayoutVersion::Bytes91 => {
            profile_fixed_width_lf::<CURRENT_FULL_ORDER_RECORD_LEN>(source)
        }
    }
}

#[must_use]
pub fn profile_full_trade_framing(source: &[u8], source_date: SourceDate) -> FramingProfile<'_> {
    match full_cm_layout_for_date(source_date).trade {
        FullTradeLayoutVersion::Bytes100 => {
            profile_fixed_width_lf::<LEGACY_FULL_TRADE_100_RECORD_LEN>(source)
        }
        FullTradeLayoutVersion::Bytes101 => {
            profile_fixed_width_lf::<LEGACY_FULL_TRADE_101_RECORD_LEN>(source)
        }
        FullTradeLayoutVersion::Bytes103 => {
            profile_fixed_width_lf::<CURRENT_FULL_TRADE_RECORD_LEN>(source)
        }
    }
}

pub fn profile_full_order_field_bytes(
    source: &[u8],
    layout: FullOrderLayoutVersion,
) -> Result<FieldByteSourceProfile<'_>, FieldByteProfileError> {
    match layout {
        FullOrderLayoutVersion::Bytes87 => {
            profile_fixed_width_field_bytes::<LEGACY_FULL_ORDER_RECORD_LEN>(source, layout.fields())
        }
        FullOrderLayoutVersion::Bytes91 => profile_fixed_width_field_bytes::<
            CURRENT_FULL_ORDER_RECORD_LEN,
        >(source, layout.fields()),
    }
}

pub fn profile_full_trade_field_bytes(
    source: &[u8],
    layout: FullTradeLayoutVersion,
) -> Result<FieldByteSourceProfile<'_>, FieldByteProfileError> {
    match layout {
        FullTradeLayoutVersion::Bytes100 => profile_fixed_width_field_bytes::<
            LEGACY_FULL_TRADE_100_RECORD_LEN,
        >(source, layout.fields()),
        FullTradeLayoutVersion::Bytes101 => profile_fixed_width_field_bytes::<
            LEGACY_FULL_TRADE_101_RECORD_LEN,
        >(source, layout.fields()),
        FullTradeLayoutVersion::Bytes103 => profile_fixed_width_field_bytes::<
            CURRENT_FULL_TRADE_RECORD_LEN,
        >(source, layout.fields()),
    }
}

pub fn profile_fixed_width_field_bytes<'a, const N: usize>(
    source: &'a [u8],
    fields: &[FieldLayout],
) -> Result<FieldByteSourceProfile<'a>, FieldByteProfileError> {
    for field in fields {
        if field.end() > N {
            return Err(FieldByteProfileError::FieldOutOfBounds {
                field: *field,
                payload_width: N,
            });
        }
    }

    let mut profile = FieldByteSourceProfile {
        framing: FramingProfile {
            payload_width: N,
            source_bytes: source.len(),
            framed_records: 0,
            framing_error: None,
        },
        fields: fields
            .iter()
            .copied()
            .map(|field| FieldByteProfile {
                field,
                byte_counts: [0; 256],
            })
            .collect(),
    };

    for outcome in FixedWidthLfRecords::<N>::new(source) {
        let record = match outcome {
            Ok(record) => record,
            Err(error) => {
                profile.framing.framing_error = Some(error);
                break;
            }
        };
        profile.framing.framed_records += 1;

        for field_profile in &mut profile.fields {
            let field = field_profile.field;
            for byte in &record.raw[field.offset..field.end()] {
                let count = &mut field_profile.byte_counts[usize::from(*byte)];
                *count = count
                    .checked_add(1)
                    .ok_or(FieldByteProfileError::ByteCountOverflow { field, byte: *byte })?;
            }
        }
    }

    Ok(profile)
}

pub fn profile_trim_order_stream_set<'a>(
    delivery: &CmPairedStreamSet,
    sources: &[CmStreamSource<'a>],
) -> Result<JiffyStreamSetProfile<'a>, CmStreamSourceSetError> {
    profile_trim_stream_set(delivery, sources, CmRecordKind::Orders, |source| {
        profile_trim_order_jiffies(source)
    })
}

pub fn profile_trim_trade_stream_set<'a>(
    delivery: &CmPairedStreamSet,
    sources: &[CmStreamSource<'a>],
) -> Result<JiffyStreamSetProfile<'a>, CmStreamSourceSetError> {
    profile_trim_stream_set(delivery, sources, CmRecordKind::Trades, |source| {
        profile_trim_trade_jiffies(source)
    })
}

fn profile_trim_stream_set<'a>(
    delivery: &CmPairedStreamSet,
    sources: &[CmStreamSource<'a>],
    expected_kind: CmRecordKind,
    profile_source: impl Fn(&'a [u8]) -> JiffySourceProfile<'a>,
) -> Result<JiffyStreamSetProfile<'a>, CmStreamSourceSetError> {
    let ordered_sources = ordered_trim_stream_sources(delivery, sources, expected_kind)?;

    let streams = ordered_sources
        .into_iter()
        .map(|stream| JiffyStreamProfile {
            member: stream.member,
            source: profile_source(stream.source),
        })
        .collect::<Vec<_>>();
    let mut range_overlaps = Vec::new();
    for (left_index, left) in streams.iter().enumerate() {
        for right in &streams[left_index + 1..] {
            if left.source.sequence.range_overlaps(&right.source.sequence) {
                range_overlaps.push(JiffyStreamRangeOverlap {
                    left_member: left.member,
                    right_member: right.member,
                });
            }
        }
    }

    Ok(JiffyStreamSetProfile {
        streams,
        range_overlaps,
    })
}

pub fn replay_trim_order_stream_set<'a>(
    delivery: &CmPairedStreamSet,
    sources: &[CmStreamSource<'a>],
) -> Result<TrimOrderStreamSetReplay<'a>, CmStreamSourceSetError> {
    let ordered_sources = ordered_trim_stream_sources(delivery, sources, CmRecordKind::Orders)?;
    let streams = ordered_sources
        .into_iter()
        .map(|stream| TrimOrderStreamReplay {
            member: stream.member,
            replay: replay_trim_order_source(stream.source),
        })
        .collect();

    Ok(TrimOrderStreamSetReplay { streams })
}

fn ordered_trim_stream_sources<'a>(
    delivery: &CmPairedStreamSet,
    sources: &[CmStreamSource<'a>],
    expected_kind: CmRecordKind,
) -> Result<Vec<CmStreamSource<'a>>, CmStreamSourceSetError> {
    if delivery.key.product != CmProduct::Trim {
        return Err(CmStreamSourceSetError::ExpectedTrim {
            actual: delivery.key.product,
        });
    }
    if delivery.key.record_kind != expected_kind {
        return Err(CmStreamSourceSetError::RecordKindMismatch {
            expected: expected_kind,
            actual: delivery.key.record_kind,
        });
    }

    let mut ordered_sources = sources.to_vec();
    ordered_sources.sort_by_key(|source| source.member);
    let actual = ordered_sources
        .iter()
        .map(|source| source.member)
        .collect::<Vec<_>>();
    if actual != delivery.members {
        return Err(CmStreamSourceSetError::StreamMembersMismatch {
            expected: delivery.members.clone(),
            actual,
        });
    }
    Ok(ordered_sources)
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
