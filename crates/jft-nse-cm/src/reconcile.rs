//! Replay, trade-reference, and independent-summary reconciliation.

use std::collections::BTreeMap;

use crate::book::{BookProjectionError, OrderBookProjection};
use crate::events::{MarketSegment, OrderEvent, Session, Side, TradeEvent};
use crate::framing::{FixedWidthLfRecords, FramingError, FramingProfile, profile_fixed_width_lf};
use crate::jiffy::ExchangeJiffy;
use crate::layouts::{
    CURRENT_FULL_ORDER_RECORD_LEN, CURRENT_FULL_TRADE_RECORD_LEN, FieldLayout,
    FullOrderLayoutVersion, FullTradeLayoutVersion, LEGACY_FULL_ORDER_RECORD_LEN,
    LEGACY_FULL_TRADE_100_RECORD_LEN, LEGACY_FULL_TRADE_101_RECORD_LEN, SourceDate,
    TRIM_ORDER_FIELDS, TRIM_ORDER_RECORD_LEN, TRIM_TRADE_FIELDS, full_cm_layout_for_date,
};
use crate::lifecycle::{LifecycleAnomaly, LifecycleTransition, OrderKey, OrderLifecycle};
use crate::manifest::{CmFilePrefix, CmPairedStreamSet, CmProduct, CmRecordKind};
use crate::parser::{LocatedParse, parse_trim_order_source, parse_trim_trade_source};
use crate::quarantine::{QuarantineReason, QuarantinedRecord};

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
    pub exchange_jiffy: ExchangeJiffy,
    pub coordinate: SourceCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedQuarantine {
    pub coordinate: SourceCoordinate,
    pub raw_jiffy: Option<ExchangeJiffy>,
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
    pub jiffy_sequence: JiffySequenceProfile,
    pub session_counts: SessionObservationCounts,
    pub framed_records: usize,
    pub parsed_records: usize,
    pub applied_events: usize,
    pub quarantines: Vec<LocatedQuarantine>,
    pub lifecycle_anomalies: Vec<LocatedLifecycleAnomaly>,
    pub framing_error: Option<FramingError<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrderReplaySummary {
    pub framed_records: usize,
    pub parsed_records: usize,
    pub applied_events: usize,
    pub quarantine_count: usize,
    pub lifecycle_anomaly_count: usize,
    pub active_order_observations: usize,
    pub session_counts: SessionObservationCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SessionObservationCounts {
    pub pre_open: usize,
    pub regular_market: usize,
    pub closing_auction: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionObservationCountError {
    PreOpenOverflow,
    RegularMarketOverflow,
    ClosingAuctionOverflow,
}

impl SessionObservationCounts {
    #[must_use]
    pub fn total(&self) -> Option<usize> {
        self.pre_open
            .checked_add(self.regular_market)
            .and_then(|count| count.checked_add(self.closing_auction))
    }

    pub fn checked_merge(self, other: Self) -> Result<Self, SessionObservationCountError> {
        Ok(Self {
            pre_open: self
                .pre_open
                .checked_add(other.pre_open)
                .ok_or(SessionObservationCountError::PreOpenOverflow)?,
            regular_market: self
                .regular_market
                .checked_add(other.regular_market)
                .ok_or(SessionObservationCountError::RegularMarketOverflow)?,
            closing_auction: self
                .closing_auction
                .checked_add(other.closing_auction)
                .ok_or(SessionObservationCountError::ClosingAuctionOverflow)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderReplaySummaryError {
    FramedRecordOverflow,
    ParsedRecordOverflow,
    AppliedEventOverflow,
    QuarantineCountOverflow,
    LifecycleAnomalyCountOverflow,
    ActiveOrderObservationOverflow,
    SessionObservationCountOverflow(SessionObservationCountError),
}

impl OrderReplaySummary {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.parsed_records.checked_add(self.quarantine_count) == Some(self.framed_records)
            && self
                .applied_events
                .checked_add(self.lifecycle_anomaly_count)
                == Some(self.parsed_records)
            && self.session_counts.total() == Some(self.parsed_records)
    }

    pub fn checked_merge(self, other: Self) -> Result<Self, OrderReplaySummaryError> {
        Ok(Self {
            framed_records: self
                .framed_records
                .checked_add(other.framed_records)
                .ok_or(OrderReplaySummaryError::FramedRecordOverflow)?,
            parsed_records: self
                .parsed_records
                .checked_add(other.parsed_records)
                .ok_or(OrderReplaySummaryError::ParsedRecordOverflow)?,
            applied_events: self
                .applied_events
                .checked_add(other.applied_events)
                .ok_or(OrderReplaySummaryError::AppliedEventOverflow)?,
            quarantine_count: self
                .quarantine_count
                .checked_add(other.quarantine_count)
                .ok_or(OrderReplaySummaryError::QuarantineCountOverflow)?,
            lifecycle_anomaly_count: self
                .lifecycle_anomaly_count
                .checked_add(other.lifecycle_anomaly_count)
                .ok_or(OrderReplaySummaryError::LifecycleAnomalyCountOverflow)?,
            active_order_observations: self
                .active_order_observations
                .checked_add(other.active_order_observations)
                .ok_or(OrderReplaySummaryError::ActiveOrderObservationOverflow)?,
            session_counts: self
                .session_counts
                .checked_merge(other.session_counts)
                .map_err(OrderReplaySummaryError::SessionObservationCountOverflow)?,
        })
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TradeOrderCandidateProfile {
    pub trade_count: usize,
    pub reference_count: usize,
    pub candidate_count: usize,
    pub zero_candidate_references: usize,
    pub single_candidate_references: usize,
    pub multiple_candidate_references: usize,
    pub expected_side_candidates: usize,
    pub side_mismatch_candidates: usize,
    pub entry_before_trade_candidates: usize,
    pub entry_equal_trade_candidates: usize,
    pub entry_after_trade_candidates: usize,
    pub maximum_candidates_per_reference: usize,
}

impl TradeOrderCandidateProfile {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        let minimum_candidate_count = self
            .multiple_candidate_references
            .checked_mul(2)
            .and_then(|count| count.checked_add(self.single_candidate_references));

        self.trade_count.checked_mul(2) == Some(self.reference_count)
            && self
                .zero_candidate_references
                .checked_add(self.single_candidate_references)
                .and_then(|count| count.checked_add(self.multiple_candidate_references))
                == Some(self.reference_count)
            && self
                .expected_side_candidates
                .checked_add(self.side_mismatch_candidates)
                == Some(self.candidate_count)
            && self
                .entry_before_trade_candidates
                .checked_add(self.entry_equal_trade_candidates)
                .and_then(|count| count.checked_add(self.entry_after_trade_candidates))
                == Some(self.candidate_count)
            && minimum_candidate_count.is_some_and(|minimum| minimum <= self.candidate_count)
            && self.maximum_candidates_per_reference <= self.candidate_count
            && (self.reference_count > 0 || self.maximum_candidates_per_reference == 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TradeValueSummary {
    pub trade_count: usize,
    pub total_quantity: u128,
    pub total_notional_paise: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeValueSummaryError {
    TradeCountOverflow,
    TotalQuantityOverflow,
    TotalNotionalOverflow,
}

impl TradeValueSummary {
    pub fn checked_add_trade(self, trade: &TradeEvent) -> Result<Self, TradeValueSummaryError> {
        let trade_count = self
            .trade_count
            .checked_add(1)
            .ok_or(TradeValueSummaryError::TradeCountOverflow)?;
        let total_quantity = self
            .total_quantity
            .checked_add(u128::from(trade.quantity))
            .ok_or(TradeValueSummaryError::TotalQuantityOverflow)?;
        let trade_notional_paise = u128::from(trade.price_paise)
            .checked_mul(u128::from(trade.quantity))
            .ok_or(TradeValueSummaryError::TotalNotionalOverflow)?;
        let total_notional_paise = self
            .total_notional_paise
            .checked_add(trade_notional_paise)
            .ok_or(TradeValueSummaryError::TotalNotionalOverflow)?;

        Ok(Self {
            trade_count,
            total_quantity,
            total_notional_paise,
        })
    }

    pub fn checked_merge(self, other: Self) -> Result<Self, TradeValueSummaryError> {
        Ok(Self {
            trade_count: self
                .trade_count
                .checked_add(other.trade_count)
                .ok_or(TradeValueSummaryError::TradeCountOverflow)?,
            total_quantity: self
                .total_quantity
                .checked_add(other.total_quantity)
                .ok_or(TradeValueSummaryError::TotalQuantityOverflow)?,
            total_notional_paise: self
                .total_notional_paise
                .checked_add(other.total_notional_paise)
                .ok_or(TradeValueSummaryError::TotalNotionalOverflow)?,
        })
    }
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
    pub delivery: CmPairedStreamSet,
    pub streams: Vec<TrimOrderStreamReplay<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionExcludedOrderIdentity {
    pub segment: MarketSegment,
    pub symbol_raw: [u8; 10],
    pub series_raw: [u8; 2],
    pub order_number: u64,
}

impl From<&OrderKey> for SessionExcludedOrderIdentity {
    fn from(key: &OrderKey) -> Self {
        Self {
            segment: key.segment,
            symbol_raw: key.symbol_raw,
            series_raw: key.series_raw,
            order_number: key.order_number,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionOrderEntryOccurrence {
    pub member: Option<u8>,
    pub session: Session,
    pub entry: OrderEntryEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossSessionOrderEntryEvidence {
    pub identity: SessionExcludedOrderIdentity,
    pub occurrences: Vec<SessionOrderEntryOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossSessionLifecycleCandidateEvidence {
    pub anomaly_member: Option<u8>,
    pub anomaly: LocatedLifecycleAnomaly,
    pub other_session_entries: Vec<SessionOrderEntryOccurrence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrossSessionLifecycleOperation {
    Modify,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrossSessionLifecycleRoute {
    pub operation: CrossSessionLifecycleOperation,
    pub anomaly_session: Session,
    pub candidate_entry_session: Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossSessionLifecycleRouteCount {
    pub route: CrossSessionLifecycleRoute,
    pub candidate_event_count: usize,
    pub entry_candidate_count: usize,
    pub entry_before_event_candidates: usize,
    pub entry_equal_event_candidates: usize,
    pub entry_after_event_candidates: usize,
}

impl CrossSessionLifecycleRouteCount {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.entry_before_event_candidates
            .checked_add(self.entry_equal_event_candidates)
            .and_then(|count| count.checked_add(self.entry_after_event_candidates))
            == Some(self.entry_candidate_count)
            && self.candidate_event_count > 0
            && self.candidate_event_count <= self.entry_candidate_count
    }
}

impl TrimOrderStreamSetReplay<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.streams
            .iter()
            .map(|stream| stream.member)
            .eq(self.delivery.members.iter().copied())
            && self
                .streams
                .iter()
                .all(|stream| stream.replay.counts_reconcile())
    }

    pub fn combined_summary(&self) -> Result<OrderReplaySummary, OrderReplaySummaryError> {
        self.streams
            .iter()
            .map(|stream| stream.replay.summary())
            .try_fold(
                OrderReplaySummary::default(),
                OrderReplaySummary::checked_merge,
            )
    }

    /// Reports successful entry evidence whose exact non-session fields occur
    /// under more than one session code. This is occurrence evidence only: it
    /// does not imply that the exchange carried one lifecycle across sessions.
    #[must_use]
    pub fn cross_session_entry_evidence(&self) -> Vec<CrossSessionOrderEntryEvidence> {
        self.session_excluded_entry_occurrences()
            .into_iter()
            .filter_map(|(identity, occurrences)| {
                let first_session = occurrences.first()?.session;
                occurrences
                    .iter()
                    .any(|occurrence| occurrence.session != first_session)
                    .then_some(CrossSessionOrderEntryEvidence {
                        identity,
                        occurrences,
                    })
            })
            .collect()
    }

    /// Reports rejected unknown modify/cancel events that have exact
    /// non-session entry candidates in another session. These are diagnostic
    /// candidates only; replay remains session-isolated and does not mutate.
    #[must_use]
    pub fn cross_session_lifecycle_candidates(
        &self,
    ) -> Vec<CrossSessionLifecycleCandidateEvidence> {
        let entries = self.session_excluded_entry_occurrences();
        let mut candidates = Vec::new();

        for stream in &self.streams {
            for anomaly in &stream.replay.lifecycle_anomalies {
                if !matches!(
                    anomaly.anomaly,
                    LifecycleAnomaly::UnknownModify { .. } | LifecycleAnomaly::UnknownCancel { .. }
                ) {
                    continue;
                }

                let key = OrderKey::from(&anomaly.event);
                let identity = SessionExcludedOrderIdentity::from(&key);
                let other_session_entries = entries
                    .get(&identity)
                    .into_iter()
                    .flatten()
                    .copied()
                    .filter(|entry| entry.session != anomaly.event.session)
                    .collect::<Vec<_>>();

                if !other_session_entries.is_empty() {
                    candidates.push(CrossSessionLifecycleCandidateEvidence {
                        anomaly_member: stream.member,
                        anomaly: anomaly.clone(),
                        other_session_entries,
                    });
                }
            }
        }

        candidates
    }

    /// Aggregates candidate dossiers by rejected operation and session route.
    /// Event count is distinct from entry-candidate count because one rejected
    /// event can have matching entry evidence in multiple independent streams.
    #[must_use]
    pub fn cross_session_lifecycle_route_counts(&self) -> Vec<CrossSessionLifecycleRouteCount> {
        let mut routes =
            BTreeMap::<CrossSessionLifecycleRoute, CrossSessionLifecycleRouteCount>::new();

        for candidate in self.cross_session_lifecycle_candidates() {
            let operation = match candidate.anomaly.anomaly {
                LifecycleAnomaly::UnknownModify { .. } => CrossSessionLifecycleOperation::Modify,
                LifecycleAnomaly::UnknownCancel { .. } => CrossSessionLifecycleOperation::Cancel,
                LifecycleAnomaly::DuplicateEntry { .. }
                | LifecycleAnomaly::RegressingJiffy { .. } => continue,
            };
            let mut event_routes = BTreeMap::<CrossSessionLifecycleRoute, ()>::new();
            for entry in candidate.other_session_entries {
                let route = CrossSessionLifecycleRoute {
                    operation,
                    anomaly_session: candidate.anomaly.event.session,
                    candidate_entry_session: entry.session,
                };
                let counts = routes
                    .entry(route)
                    .or_insert(CrossSessionLifecycleRouteCount {
                        route,
                        candidate_event_count: 0,
                        entry_candidate_count: 0,
                        entry_before_event_candidates: 0,
                        entry_equal_event_candidates: 0,
                        entry_after_event_candidates: 0,
                    });
                counts.entry_candidate_count += 1;
                match entry
                    .entry
                    .exchange_jiffy
                    .cmp(&candidate.anomaly.event.exchange_jiffy)
                {
                    std::cmp::Ordering::Less => counts.entry_before_event_candidates += 1,
                    std::cmp::Ordering::Equal => counts.entry_equal_event_candidates += 1,
                    std::cmp::Ordering::Greater => counts.entry_after_event_candidates += 1,
                }
                event_routes.insert(route, ());
            }

            for route in event_routes.into_keys() {
                routes
                    .get_mut(&route)
                    .expect("candidate route must exist")
                    .candidate_event_count += 1;
            }
        }

        routes.into_values().collect()
    }

    fn session_excluded_entry_occurrences(
        &self,
    ) -> BTreeMap<SessionExcludedOrderIdentity, Vec<SessionOrderEntryOccurrence>> {
        let mut grouped =
            BTreeMap::<SessionExcludedOrderIdentity, Vec<SessionOrderEntryOccurrence>>::new();

        for stream in &self.streams {
            for (key, entry) in &stream.replay.order_entries {
                grouped
                    .entry(SessionExcludedOrderIdentity::from(key))
                    .or_default()
                    .push(SessionOrderEntryOccurrence {
                        member: stream.member,
                        session: key.session,
                        entry: *entry,
                    });
            }
        }

        for occurrences in grouped.values_mut() {
            occurrences.sort_by_key(|occurrence| {
                (
                    occurrence.session,
                    occurrence.member,
                    occurrence.entry.coordinate,
                )
            });
        }

        grouped
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeCandidateSourceProfile<'a> {
    pub candidates: TradeOrderCandidateProfile,
    pub value_summary: TradeValueSummary,
    pub jiffy_sequence: JiffySequenceProfile,
    pub session_counts: SessionObservationCounts,
    pub framed_records: usize,
    pub parsed_records: usize,
    pub quarantines: Vec<LocatedQuarantine>,
    pub framing_error: Option<FramingError<'a>>,
}

impl TradeCandidateSourceProfile<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.parsed_records.checked_add(self.quarantines.len()) == Some(self.framed_records)
            && self.candidates.trade_count == self.parsed_records
            && self.value_summary.trade_count == self.parsed_records
            && self.jiffy_sequence.observation_count == self.parsed_records
            && self.session_counts.total() == Some(self.parsed_records)
            && self.candidates.counts_reconcile()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeCandidateStreamProfile<'a> {
    pub member: Option<u8>,
    pub source: TradeCandidateSourceProfile<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeCandidateStreamSetProfile<'a> {
    pub delivery: CmPairedStreamSet,
    pub streams: Vec<TradeCandidateStreamProfile<'a>>,
    pub combined_value_summary: TradeValueSummary,
    pub combined_session_counts: SessionObservationCounts,
}

impl TradeCandidateStreamSetProfile<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        let members_reconcile = self.streams.iter().map(|stream| stream.member).eq(self
            .delivery
            .members
            .iter()
            .copied());
        let sources_reconcile = self
            .streams
            .iter()
            .all(|stream| stream.source.counts_reconcile());
        let recomputed = self
            .streams
            .iter()
            .try_fold(TradeValueSummary::default(), |summary, stream| {
                summary.checked_merge(stream.source.value_summary)
            });
        let recomputed_sessions = self
            .streams
            .iter()
            .try_fold(SessionObservationCounts::default(), |counts, stream| {
                counts.checked_merge(stream.source.session_counts)
            });

        members_reconcile
            && sources_reconcile
            && recomputed == Ok(self.combined_value_summary)
            && recomputed_sessions == Ok(self.combined_session_counts)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeCandidateStreamSetError {
    TradeSourceSet(CmStreamSourceSetError),
    OrderTradeDeliveryMismatch {
        order_prefix: CmFilePrefix,
        trade_prefix: CmFilePrefix,
        order_source_date: SourceDate,
        trade_source_date: SourceDate,
    },
    SourceValueSummary {
        member: Option<u8>,
        coordinate: SourceCoordinate,
        error: TradeValueSummaryError,
    },
    CombinedValueSummary(TradeValueSummaryError),
    CombinedSessionCounts(SessionObservationCountError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOrderTradeEvidence<'a> {
    pub orders: TrimOrderStreamSetReplay<'a>,
    pub order_summary: OrderReplaySummary,
    pub trades: TradeCandidateStreamSetProfile<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimEvidenceObservationKind {
    OrderQuarantine,
    OrderDuplicateEntry,
    OrderUnknownModify,
    OrderUnknownCancel,
    OrderRegressingJiffy,
    OrderCrossSessionLifecycleCandidate,
    OrderSourceJiffyRegression,
    OrderFramingLoss,
    TradeQuarantine,
    TradeFramingLoss,
    TradeZeroEntryCandidateReference,
    TradeMultipleEntryCandidateReference,
    TradeSideMismatchCandidate,
    TradeEntryEqualJiffyCandidate,
    TradeEntryAfterTradeCandidate,
    TradeSourceJiffyRegression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimEvidenceObservation {
    pub member: Option<u8>,
    pub kind: TrimEvidenceObservationKind,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderTradeJiffyRangeOverlap {
    pub order_member: Option<u8>,
    pub trade_member: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimQuarantineEvidence<'a> {
    pub record_kind: CmRecordKind,
    pub member: Option<u8>,
    pub located: &'a LocatedQuarantine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimFramingLossEvidence<'a> {
    pub record_kind: CmRecordKind,
    pub member: Option<u8>,
    pub error: FramingError<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimQuarantineClassCount<'a> {
    pub record_kind: CmRecordKind,
    pub member: Option<u8>,
    pub field: Option<FieldLayout>,
    pub reason: &'a QuarantineReason,
    pub count: usize,
    pub validated_raw_jiffy_count: usize,
}

impl TrimQuarantineClassCount<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.count > 0 && self.validated_raw_jiffy_count <= self.count
    }
}

impl TrimOrderTradeEvidence<'_> {
    #[must_use]
    pub fn counts_reconcile(&self) -> bool {
        self.orders.counts_reconcile()
            && self.orders.combined_summary() == Ok(self.order_summary)
            && self.order_summary.counts_reconcile()
            && self.trades.counts_reconcile()
    }

    #[must_use]
    pub fn observations(&self) -> Vec<TrimEvidenceObservation> {
        let mut observations = Vec::new();
        let cross_session_candidates = self.orders.cross_session_lifecycle_candidates();

        for stream in &self.orders.streams {
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::OrderQuarantine,
                stream.replay.quarantines.len(),
            );
            for (kind, count) in order_lifecycle_observation_counts(&stream.replay) {
                push_observation(&mut observations, stream.member, kind, count);
            }
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::OrderCrossSessionLifecycleCandidate,
                cross_session_candidates
                    .iter()
                    .filter(|candidate| candidate.anomaly_member == stream.member)
                    .count(),
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::OrderSourceJiffyRegression,
                stream.replay.jiffy_sequence.regressions.len(),
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::OrderFramingLoss,
                usize::from(stream.replay.framing_error.is_some()),
            );
        }

        for stream in &self.trades.streams {
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeQuarantine,
                stream.source.quarantines.len(),
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeFramingLoss,
                usize::from(stream.source.framing_error.is_some()),
            );
            let candidates = &stream.source.candidates;
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeZeroEntryCandidateReference,
                candidates.zero_candidate_references,
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeMultipleEntryCandidateReference,
                candidates.multiple_candidate_references,
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeSideMismatchCandidate,
                candidates.side_mismatch_candidates,
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeEntryEqualJiffyCandidate,
                candidates.entry_equal_trade_candidates,
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeEntryAfterTradeCandidate,
                candidates.entry_after_trade_candidates,
            );
            push_observation(
                &mut observations,
                stream.member,
                TrimEvidenceObservationKind::TradeSourceJiffyRegression,
                stream.source.jiffy_sequence.regressions.len(),
            );
        }

        observations
    }

    /// Borrows every located quarantine in deterministic record-kind, member,
    /// and source-coordinate order. The enclosing report retains delivery
    /// identity; this view adds record kind and stream without copying raw data.
    #[must_use]
    pub fn quarantine_evidence(&self) -> Vec<TrimQuarantineEvidence<'_>> {
        let mut evidence = Vec::new();

        for stream in &self.orders.streams {
            for located in &stream.replay.quarantines {
                evidence.push(TrimQuarantineEvidence {
                    record_kind: CmRecordKind::Orders,
                    member: stream.member,
                    located,
                });
            }
        }
        for stream in &self.trades.streams {
            for located in &stream.source.quarantines {
                evidence.push(TrimQuarantineEvidence {
                    record_kind: CmRecordKind::Trades,
                    member: stream.member,
                    located,
                });
            }
        }

        evidence
    }

    /// Aggregates the borrowed dossiers by exact failure class in their
    /// deterministic first-observed order. Raw-jiffy availability is counted
    /// separately and never changes whether the record remains quarantined.
    #[must_use]
    pub fn quarantine_class_counts(&self) -> Vec<TrimQuarantineClassCount<'_>> {
        let mut classes = Vec::<TrimQuarantineClassCount<'_>>::new();

        for evidence in self.quarantine_evidence() {
            let field = evidence.located.quarantined.field;
            let reason = &evidence.located.quarantined.reason;
            if let Some(class) = classes.iter_mut().find(|class| {
                class.record_kind == evidence.record_kind
                    && class.member == evidence.member
                    && class.field == field
                    && class.reason == reason
            }) {
                class.count += 1;
                class.validated_raw_jiffy_count +=
                    usize::from(evidence.located.raw_jiffy.is_some());
            } else {
                classes.push(TrimQuarantineClassCount {
                    record_kind: evidence.record_kind,
                    member: evidence.member,
                    field,
                    reason,
                    count: 1,
                    validated_raw_jiffy_count: usize::from(evidence.located.raw_jiffy.is_some()),
                });
            }
        }

        classes
    }

    /// Copies only the small framing-error descriptors while preserving their
    /// zero-copy tail slices in deterministic Order/Trade and member order.
    #[must_use]
    pub fn framing_loss_evidence(&self) -> Vec<TrimFramingLossEvidence<'_>> {
        let mut evidence = Vec::new();

        for stream in &self.orders.streams {
            if let Some(error) = stream.replay.framing_error {
                evidence.push(TrimFramingLossEvidence {
                    record_kind: CmRecordKind::Orders,
                    member: stream.member,
                    error,
                });
            }
        }
        for stream in &self.trades.streams {
            if let Some(error) = stream.source.framing_error {
                evidence.push(TrimFramingLossEvidence {
                    record_kind: CmRecordKind::Trades,
                    member: stream.member,
                    error,
                });
            }
        }

        evidence
    }

    #[must_use]
    pub fn order_range_overlaps(&self) -> Vec<JiffyStreamRangeOverlap> {
        let mut overlaps = Vec::new();
        for (left_index, left) in self.orders.streams.iter().enumerate() {
            for right in &self.orders.streams[left_index + 1..] {
                if left
                    .replay
                    .jiffy_sequence
                    .range_overlaps(&right.replay.jiffy_sequence)
                {
                    overlaps.push(JiffyStreamRangeOverlap {
                        left_member: left.member,
                        right_member: right.member,
                    });
                }
            }
        }
        overlaps
    }

    #[must_use]
    pub fn trade_range_overlaps(&self) -> Vec<JiffyStreamRangeOverlap> {
        let mut overlaps = Vec::new();
        for (left_index, left) in self.trades.streams.iter().enumerate() {
            for right in &self.trades.streams[left_index + 1..] {
                if left
                    .source
                    .jiffy_sequence
                    .range_overlaps(&right.source.jiffy_sequence)
                {
                    overlaps.push(JiffyStreamRangeOverlap {
                        left_member: left.member,
                        right_member: right.member,
                    });
                }
            }
        }
        overlaps
    }

    #[must_use]
    pub fn order_trade_range_overlaps(&self) -> Vec<OrderTradeJiffyRangeOverlap> {
        let mut overlaps = Vec::new();
        for order in &self.orders.streams {
            for trade in &self.trades.streams {
                if order
                    .replay
                    .jiffy_sequence
                    .range_overlaps(&trade.source.jiffy_sequence)
                {
                    overlaps.push(OrderTradeJiffyRangeOverlap {
                        order_member: order.member,
                        trade_member: trade.member,
                    });
                }
            }
        }
        overlaps
    }
}

fn order_lifecycle_observation_counts(
    replay: &TrimOrderReplay<'_>,
) -> [(TrimEvidenceObservationKind, usize); 4] {
    let mut duplicate_entry = 0;
    let mut unknown_modify = 0;
    let mut unknown_cancel = 0;
    let mut regressing_jiffy = 0;

    for located in &replay.lifecycle_anomalies {
        match located.anomaly {
            LifecycleAnomaly::DuplicateEntry { .. } => duplicate_entry += 1,
            LifecycleAnomaly::UnknownModify { .. } => unknown_modify += 1,
            LifecycleAnomaly::UnknownCancel { .. } => unknown_cancel += 1,
            LifecycleAnomaly::RegressingJiffy { .. } => regressing_jiffy += 1,
        }
    }

    [
        (
            TrimEvidenceObservationKind::OrderDuplicateEntry,
            duplicate_entry,
        ),
        (
            TrimEvidenceObservationKind::OrderUnknownModify,
            unknown_modify,
        ),
        (
            TrimEvidenceObservationKind::OrderUnknownCancel,
            unknown_cancel,
        ),
        (
            TrimEvidenceObservationKind::OrderRegressingJiffy,
            regressing_jiffy,
        ),
    ]
}

fn push_observation(
    observations: &mut Vec<TrimEvidenceObservation>,
    member: Option<u8>,
    kind: TrimEvidenceObservationKind,
    count: usize,
) {
    if count > 0 {
        observations.push(TrimEvidenceObservation {
            member,
            kind,
            count,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrimOrderTradeEvidenceError {
    OrderReplay(CmStreamSourceSetError),
    OrderSummary(OrderReplaySummaryError),
    TradeProfile(TradeCandidateStreamSetError),
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
            && self.jiffy_sequence.observation_count == self.parsed_records
            && self.session_counts.total() == Some(self.parsed_records)
    }

    #[must_use]
    pub fn summary(&self) -> OrderReplaySummary {
        OrderReplaySummary {
            framed_records: self.framed_records,
            parsed_records: self.parsed_records,
            applied_events: self.applied_events,
            quarantine_count: self.quarantines.len(),
            lifecycle_anomaly_count: self.lifecycle_anomalies.len(),
            active_order_observations: self.lifecycle.len(),
            session_counts: self.session_counts,
        }
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
        jiffy_sequence: JiffySequenceProfile::default(),
        session_counts: SessionObservationCounts::default(),
        framed_records: 0,
        parsed_records: 0,
        applied_events: 0,
        quarantines: Vec::new(),
        lifecycle_anomalies: Vec::new(),
        framing_error: None,
    };
    let mut previous_jiffy = None;
    let mut current_equal_jiffy_run = 0;

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
                    raw_jiffy: validated_raw_jiffy(&quarantined.raw, TRIM_ORDER_FIELDS[3]),
                    quarantined,
                });
                continue;
            }
        };
        replay.parsed_records += 1;
        observe_session(&mut replay.session_counts, parsed.event.session);
        observe_jiffy_sequence(
            &mut replay.jiffy_sequence,
            &mut previous_jiffy,
            &mut current_equal_jiffy_run,
            coordinate,
            parsed.event.exchange_jiffy,
        );
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
                    exchange_jiffy: current.entry_jiffy,
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

#[must_use]
pub fn profile_trim_trade_stream_candidates<'a>(
    orders: &TrimOrderStreamSetReplay<'_>,
    trades: impl IntoIterator<Item = &'a TradeEvent>,
) -> TradeOrderCandidateProfile {
    let mut profile = TradeOrderCandidateProfile::default();

    for trade in trades {
        profile.trade_count += 1;
        let candidates = collect_trim_trade_stream_candidates(orders, trade);
        observe_candidate_reference(&mut profile, &candidates.buy, trade.exchange_jiffy);
        observe_candidate_reference(&mut profile, &candidates.sell, trade.exchange_jiffy);
    }

    profile
}

fn observe_candidate_reference(
    profile: &mut TradeOrderCandidateProfile,
    reference: &StreamOrderSourceCandidates,
    trade_jiffy: ExchangeJiffy,
) {
    profile.reference_count += 1;
    profile.candidate_count += reference.candidates.len();
    profile.maximum_candidates_per_reference = profile
        .maximum_candidates_per_reference
        .max(reference.candidates.len());

    match reference.candidates.len() {
        0 => profile.zero_candidate_references += 1,
        1 => profile.single_candidate_references += 1,
        _ => profile.multiple_candidate_references += 1,
    }

    for candidate in &reference.candidates {
        if candidate.entry.side == reference.expected_side {
            profile.expected_side_candidates += 1;
        } else {
            profile.side_mismatch_candidates += 1;
        }
        match candidate.entry.exchange_jiffy.cmp(&trade_jiffy) {
            std::cmp::Ordering::Less => profile.entry_before_trade_candidates += 1,
            std::cmp::Ordering::Equal => profile.entry_equal_trade_candidates += 1,
            std::cmp::Ordering::Greater => profile.entry_after_trade_candidates += 1,
        }
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
    profile_jiffies(
        parse_trim_order_source(source),
        TRIM_ORDER_FIELDS[3],
        |event| event.exchange_jiffy,
    )
}

#[must_use]
pub fn profile_trim_trade_jiffies(source: &[u8]) -> JiffySourceProfile<'_> {
    profile_jiffies(
        parse_trim_trade_source(source),
        TRIM_TRADE_FIELDS[2],
        |event| event.exchange_jiffy,
    )
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

pub fn profile_trim_trade_candidate_stream_set<'a>(
    orders: &TrimOrderStreamSetReplay<'_>,
    trade_delivery: &CmPairedStreamSet,
    trade_sources: &[CmStreamSource<'a>],
) -> Result<TradeCandidateStreamSetProfile<'a>, TradeCandidateStreamSetError> {
    let ordered_sources =
        ordered_trim_stream_sources(trade_delivery, trade_sources, CmRecordKind::Trades)
            .map_err(TradeCandidateStreamSetError::TradeSourceSet)?;

    if orders.delivery.prefix != trade_delivery.prefix
        || orders.delivery.key.source_date != trade_delivery.key.source_date
    {
        return Err(TradeCandidateStreamSetError::OrderTradeDeliveryMismatch {
            order_prefix: orders.delivery.prefix,
            trade_prefix: trade_delivery.prefix,
            order_source_date: orders.delivery.key.source_date,
            trade_source_date: trade_delivery.key.source_date,
        });
    }

    let mut streams = Vec::with_capacity(ordered_sources.len());
    for stream in ordered_sources {
        let source = profile_trim_trade_candidate_source(orders, stream.source).map_err(
            |(coordinate, error)| TradeCandidateStreamSetError::SourceValueSummary {
                member: stream.member,
                coordinate,
                error,
            },
        )?;
        streams.push(TradeCandidateStreamProfile {
            member: stream.member,
            source,
        });
    }

    let combined_value_summary = streams
        .iter()
        .try_fold(TradeValueSummary::default(), |summary, stream| {
            summary.checked_merge(stream.source.value_summary)
        })
        .map_err(TradeCandidateStreamSetError::CombinedValueSummary)?;
    let combined_session_counts = streams
        .iter()
        .try_fold(SessionObservationCounts::default(), |counts, stream| {
            counts.checked_merge(stream.source.session_counts)
        })
        .map_err(TradeCandidateStreamSetError::CombinedSessionCounts)?;

    Ok(TradeCandidateStreamSetProfile {
        delivery: trade_delivery.clone(),
        streams,
        combined_value_summary,
        combined_session_counts,
    })
}

pub fn profile_trim_order_trade_evidence<'a>(
    order_delivery: &CmPairedStreamSet,
    order_sources: &[CmStreamSource<'a>],
    trade_delivery: &CmPairedStreamSet,
    trade_sources: &[CmStreamSource<'a>],
) -> Result<TrimOrderTradeEvidence<'a>, TrimOrderTradeEvidenceError> {
    let orders = replay_trim_order_stream_set(order_delivery, order_sources)
        .map_err(TrimOrderTradeEvidenceError::OrderReplay)?;
    let order_summary = orders
        .combined_summary()
        .map_err(TrimOrderTradeEvidenceError::OrderSummary)?;
    let trades = profile_trim_trade_candidate_stream_set(&orders, trade_delivery, trade_sources)
        .map_err(TrimOrderTradeEvidenceError::TradeProfile)?;

    Ok(TrimOrderTradeEvidence {
        orders,
        order_summary,
        trades,
    })
}

fn profile_trim_trade_candidate_source<'a>(
    orders: &TrimOrderStreamSetReplay<'_>,
    source: &'a [u8],
) -> Result<TradeCandidateSourceProfile<'a>, (SourceCoordinate, TradeValueSummaryError)> {
    let mut profile = TradeCandidateSourceProfile {
        candidates: TradeOrderCandidateProfile::default(),
        value_summary: TradeValueSummary::default(),
        jiffy_sequence: JiffySequenceProfile::default(),
        session_counts: SessionObservationCounts::default(),
        framed_records: 0,
        parsed_records: 0,
        quarantines: Vec::new(),
        framing_error: None,
    };
    let mut previous_jiffy = None;
    let mut current_equal_jiffy_run = 0;

    for outcome in parse_trim_trade_source(source) {
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
                    raw_jiffy: validated_raw_jiffy(&quarantined.raw, TRIM_TRADE_FIELDS[2]),
                    quarantined,
                });
                continue;
            }
        };
        profile.parsed_records += 1;
        observe_session(&mut profile.session_counts, parsed.event.session);
        observe_jiffy_sequence(
            &mut profile.jiffy_sequence,
            &mut previous_jiffy,
            &mut current_equal_jiffy_run,
            coordinate,
            parsed.event.exchange_jiffy,
        );
        profile.value_summary = profile
            .value_summary
            .checked_add_trade(&parsed.event)
            .map_err(|error| (coordinate, error))?;
        let candidates = collect_trim_trade_stream_candidates(orders, &parsed.event);
        profile.candidates.trade_count += 1;
        observe_candidate_reference(
            &mut profile.candidates,
            &candidates.buy,
            parsed.event.exchange_jiffy,
        );
        observe_candidate_reference(
            &mut profile.candidates,
            &candidates.sell,
            parsed.event.exchange_jiffy,
        );
    }

    Ok(profile)
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

    Ok(TrimOrderStreamSetReplay {
        delivery: delivery.clone(),
        streams,
    })
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
    jiffy_field: FieldLayout,
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
                    raw_jiffy: validated_raw_jiffy(&quarantined.raw, jiffy_field),
                    quarantined,
                });
                continue;
            }
        };
        profile.parsed_records += 1;
        let current = exchange_jiffy(&parsed.event);
        observe_jiffy_sequence(
            &mut profile.sequence,
            &mut previous,
            &mut current_equal_run,
            coordinate,
            current,
        );
    }

    profile
}

fn validated_raw_jiffy(raw: &[u8], field: FieldLayout) -> Option<ExchangeJiffy> {
    let value_bytes = raw.get(field.offset..field.end())?;
    if !value_bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }

    value_bytes
        .iter()
        .try_fold(0_u64, |value, digit| {
            value
                .checked_mul(10)
                .and_then(|current| current.checked_add(u64::from(digit - b'0')))
        })
        .map(ExchangeJiffy::new)
}

fn observe_jiffy_sequence(
    profile: &mut JiffySequenceProfile,
    previous: &mut Option<(SourceCoordinate, ExchangeJiffy)>,
    current_equal_run: &mut usize,
    coordinate: SourceCoordinate,
    current: ExchangeJiffy,
) {
    profile.observation_count += 1;
    profile.minimum = Some(
        profile
            .minimum
            .map_or(current, |minimum| minimum.min(current)),
    );
    profile.maximum = Some(
        profile
            .maximum
            .map_or(current, |maximum| maximum.max(current)),
    );

    if let Some((previous_coordinate, previous_jiffy)) = *previous {
        if current == previous_jiffy {
            profile.equal_consecutive_observations += 1;
            *current_equal_run += 1;
        } else {
            *current_equal_run = 1;
        }
        if current < previous_jiffy {
            profile.regressions.push(JiffyRegression {
                previous_coordinate,
                previous_jiffy,
                current_coordinate: coordinate,
                current_jiffy: current,
            });
        }
    } else {
        *current_equal_run = 1;
    }
    profile.largest_equal_run = profile.largest_equal_run.max(*current_equal_run);
    *previous = Some((coordinate, current));
}

fn observe_session(counts: &mut SessionObservationCounts, session: Session) {
    match session {
        Session::PreOpen => counts.pre_open += 1,
        Session::RegularMarket => counts.regular_market += 1,
        Session::ClosingAuction => counts.closing_auction += 1,
    }
}
