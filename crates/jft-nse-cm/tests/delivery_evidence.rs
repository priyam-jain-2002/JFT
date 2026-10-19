use jft_nse_cm::events::{MarketSegment, OrderActivity, Session, Side};
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::{SourceDate, TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind,
};
use jft_nse_cm::quarantine::QuarantineReason;
use jft_nse_cm::reconcile::{
    CmStreamSource, CmStreamSourceSetError, CrossSessionLifecycleOperation,
    CrossSessionLifecycleRoute, CrossSessionLifecycleRouteCount, CrossSessionOrderEntryEvidence,
    JiffyStreamRangeOverlap, OrderEntryEvidence, OrderTradeJiffyRangeOverlap,
    SessionExcludedOrderIdentity, SessionOrderEntryOccurrence, SourceCoordinate,
    TradeCandidateStreamSetError, TrimEvidenceObservation, TrimEvidenceObservationKind,
    TrimOrderTradeEvidenceError, TrimSourceByteAccounting, TrimSourceByteAccountingError,
    profile_trim_order_trade_evidence, profile_trim_order_trade_evidence_with_trade_observer,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const MODIFY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_modify.dat");
const CANCEL_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_cancel.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

fn source_date(day: u8) -> SourceDate {
    SourceDate::new(2026, 5, day).expect("valid synthetic source date")
}

fn delivery(record_kind: CmRecordKind, date: SourceDate) -> CmPairedStreamSet {
    CmPairedStreamSet {
        prefix: CmFilePrefix::Cm,
        key: CmLogicalGroupKey {
            product: CmProduct::Trim,
            record_kind,
            source_date: date,
        },
        members: vec![Some(1), Some(2)],
    }
}

fn sell_entry() -> Vec<u8> {
    let mut sell = ENTRY_FIXTURE.to_vec();
    sell[6..22].copy_from_slice(b"0000000000000002");
    sell[22..36].copy_from_slice(b"91200611817222");
    sell[36] = b'S';
    sell
}

#[test]
fn composes_deterministic_order_and_trade_evidence_without_merging_streams() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);
    let sell = sell_entry();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: &sell,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: TRADE_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: TRADE_FIXTURE,
        },
    ];
    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");
    let reversed = profile_trim_order_trade_evidence(
        &order_delivery,
        &[order_sources[1], order_sources[0]],
        &trade_delivery,
        &[trade_sources[1], trade_sources[0]],
    )
    .expect("reversed delivery evidence");

    assert_eq!(evidence, reversed);
    assert!(evidence.counts_reconcile());
    assert_eq!(evidence.orders.delivery, order_delivery);
    assert_eq!(evidence.trades.delivery, trade_delivery);
    assert_eq!(evidence.order_summary.framed_records, 2);
    assert_eq!(evidence.order_summary.applied_events, 2);
    assert_eq!(evidence.order_summary.active_order_observations, 2);
    assert_eq!(evidence.trades.combined_value_summary.trade_count, 2);
    assert_eq!(evidence.trades.combined_value_summary.total_quantity, 50);
    assert_eq!(
        evidence.trades.combined_value_summary.total_notional_paise,
        12_500_000
    );
    assert!(evidence.order_range_overlaps().is_empty());
    assert_eq!(
        evidence.trade_range_overlaps(),
        vec![JiffyStreamRangeOverlap {
            left_member: Some(1),
            right_member: Some(2),
        }]
    );
    assert_eq!(
        evidence.order_trade_range_overlaps(),
        vec![
            OrderTradeJiffyRangeOverlap {
                order_member: Some(2),
                trade_member: Some(1),
            },
            OrderTradeJiffyRangeOverlap {
                order_member: Some(2),
                trade_member: Some(2),
            },
        ]
    );
}

#[test]
fn composed_trade_observer_matches_the_authoritative_profile_without_reparsing() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);
    let sell = sell_entry();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: &sell,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: TRADE_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: TRADE_FIXTURE,
        },
    ];

    let mut observed = Vec::new();
    let evidence = profile_trim_order_trade_evidence_with_trade_observer(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
        |trade| observed.push(trade),
    )
    .expect("composed evidence with trade observer");
    let baseline = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("composed evidence baseline");

    assert_eq!(evidence, baseline);
    assert!(evidence.counts_reconcile());
    assert_eq!(
        observed.len(),
        evidence.trades.combined_value_summary.trade_count
    );
    assert_eq!(
        observed
            .iter()
            .map(|trade| trade.member)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    for trade in &observed {
        assert_eq!(
            trade.coordinate,
            SourceCoordinate {
                record_index: 0,
                byte_offset: 0,
            }
        );
        assert_eq!(trade.raw, TRADE_FIXTURE[..TRIM_TRADE_RECORD_LEN]);
        assert_eq!(
            trade.event.exchange_jiffy,
            ExchangeJiffy::new(91_200_611_817_222)
        );
        assert_eq!(
            trade
                .candidates
                .buy
                .candidates
                .iter()
                .map(|candidate| candidate.member)
                .collect::<Vec<_>>(),
            vec![Some(1)]
        );
        assert_eq!(
            trade
                .candidates
                .sell
                .candidates
                .iter()
                .map(|candidate| candidate.member)
                .collect::<Vec<_>>(),
            vec![Some(2)]
        );
    }

    let mut reversed_observed = Vec::new();
    let reversed = profile_trim_order_trade_evidence_with_trade_observer(
        &order_delivery,
        &[order_sources[1], order_sources[0]],
        &trade_delivery,
        &[trade_sources[1], trade_sources[0]],
        |trade| reversed_observed.push(trade),
    )
    .expect("reversed composed evidence with trade observer");
    assert_eq!(reversed, evidence);
    assert_eq!(reversed_observed, observed);
}

#[test]
fn rejects_incomplete_order_members_before_reading_trade_bytes() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);
    let order_sources = [CmStreamSource {
        member: Some(1),
        source: ENTRY_FIXTURE,
    }];
    let trade_sources = [
        CmStreamSource {
            member: Some(1),
            source: b"not parsed",
        },
        CmStreamSource {
            member: Some(2),
            source: b"not parsed",
        },
    ];

    assert_eq!(
        profile_trim_order_trade_evidence(
            &order_delivery,
            &order_sources,
            &trade_delivery,
            &trade_sources,
        ),
        Err(TrimOrderTradeEvidenceError::OrderReplay(
            CmStreamSourceSetError::StreamMembersMismatch {
                expected: vec![Some(1), Some(2)],
                actual: vec![Some(1)],
            }
        ))
    );

    let mut observed_trades = 0;
    assert_eq!(
        profile_trim_order_trade_evidence_with_trade_observer(
            &order_delivery,
            &order_sources,
            &trade_delivery,
            &trade_sources,
            |_| observed_trades += 1,
        ),
        Err(TrimOrderTradeEvidenceError::OrderReplay(
            CmStreamSourceSetError::StreamMembersMismatch {
                expected: vec![Some(1), Some(2)],
                actual: vec![Some(1)],
            }
        ))
    );
    assert_eq!(observed_trades, 0);
}

#[test]
fn rejects_cross_date_evidence_before_reading_trade_bytes() {
    let order_date = source_date(18);
    let trade_date = source_date(19);
    let order_delivery = delivery(CmRecordKind::Orders, order_date);
    let trade_delivery = delivery(CmRecordKind::Trades, trade_date);
    let order_sources = [
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(1),
            source: b"not parsed",
        },
        CmStreamSource {
            member: Some(2),
            source: b"not parsed",
        },
    ];

    assert_eq!(
        profile_trim_order_trade_evidence(
            &order_delivery,
            &order_sources,
            &trade_delivery,
            &trade_sources,
        ),
        Err(TrimOrderTradeEvidenceError::TradeProfile(
            TradeCandidateStreamSetError::OrderTradeDeliveryMismatch {
                order_prefix: CmFilePrefix::Cm,
                trade_prefix: CmFilePrefix::Cm,
                order_source_date: order_date,
                trade_source_date: trade_date,
            }
        ))
    );

    let mut observed_trades = 0;
    assert_eq!(
        profile_trim_order_trade_evidence_with_trade_observer(
            &order_delivery,
            &order_sources,
            &trade_delivery,
            &trade_sources,
            |_| observed_trades += 1,
        ),
        Err(TrimOrderTradeEvidenceError::TradeProfile(
            TradeCandidateStreamSetError::OrderTradeDeliveryMismatch {
                order_prefix: CmFilePrefix::Cm,
                trade_prefix: CmFilePrefix::Cm,
                order_source_date: order_date,
                trade_source_date: trade_date,
            }
        ))
    );
    assert_eq!(observed_trades, 0);
}

#[test]
fn reports_nonzero_observations_in_deterministic_stream_and_kind_order() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);

    let mut invalid_order = ENTRY_FIXTURE.to_vec();
    invalid_order[37] = b'2';
    let mut member_one_orders = [ENTRY_FIXTURE, ENTRY_FIXTURE, invalid_order.as_slice()].concat();
    member_one_orders.extend_from_slice(b"order-tail");
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_orders,
        },
    ];

    let mut invalid_trade = TRADE_FIXTURE.to_vec();
    invalid_trade[40] = b'X';
    let mut member_one_trades = [TRADE_FIXTURE, invalid_trade.as_slice()].concat();
    member_one_trades.extend_from_slice(b"trade-tail");
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_trades,
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");
    let reversed = profile_trim_order_trade_evidence(
        &order_delivery,
        &[order_sources[1], order_sources[0]],
        &trade_delivery,
        &[trade_sources[1], trade_sources[0]],
    )
    .expect("reversed delivery evidence");

    assert!(evidence.counts_reconcile());
    assert_eq!(
        evidence.observations(),
        vec![
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderQuarantine,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderDuplicateEntry,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderFramingLoss,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeQuarantine,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeFramingLoss,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeZeroEntryCandidateReference,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeMultipleEntryCandidateReference,
                count: 1,
            },
        ]
    );

    let quarantines = evidence.quarantine_evidence();
    assert_eq!(quarantines, reversed.quarantine_evidence());
    assert_eq!(quarantines.len(), 2);
    assert!(std::ptr::eq(
        quarantines[0].located,
        &evidence.orders.streams[0].replay.quarantines[0]
    ));
    assert!(std::ptr::eq(
        quarantines[1].located,
        &evidence.trades.streams[0].source.quarantines[0]
    ));
    assert_eq!(quarantines[0].record_kind, CmRecordKind::Orders);
    assert_eq!(quarantines[0].member, Some(1));
    assert_eq!(
        quarantines[0].located.coordinate,
        SourceCoordinate {
            record_index: 2,
            byte_offset: (TRIM_ORDER_RECORD_LEN + 1) * 2,
        }
    );
    assert_eq!(
        quarantines[0].located.raw_jiffy,
        Some(ExchangeJiffy::new(91_200_611_817_221))
    );
    assert_eq!(
        quarantines[0].located.quarantined.raw,
        invalid_order[..TRIM_ORDER_RECORD_LEN]
    );
    assert_eq!(
        quarantines[0]
            .located
            .quarantined
            .field
            .expect("order failure field")
            .name,
        "activity"
    );

    assert_eq!(quarantines[1].record_kind, CmRecordKind::Trades);
    assert_eq!(quarantines[1].member, Some(1));
    assert_eq!(
        quarantines[1].located.coordinate,
        SourceCoordinate {
            record_index: 1,
            byte_offset: TRIM_TRADE_RECORD_LEN + 1,
        }
    );
    assert_eq!(
        quarantines[1].located.raw_jiffy,
        Some(ExchangeJiffy::new(91_200_611_817_222))
    );
    assert_eq!(
        quarantines[1].located.quarantined.raw,
        invalid_trade[..TRIM_TRADE_RECORD_LEN]
    );
    assert_eq!(
        quarantines[1]
            .located
            .quarantined
            .field
            .expect("trade failure field")
            .name,
        "quantity"
    );

    let framing_losses = evidence.framing_loss_evidence();
    assert_eq!(framing_losses, reversed.framing_loss_evidence());
    assert_eq!(framing_losses.len(), 2);
    assert_eq!(framing_losses[0].record_kind, CmRecordKind::Orders);
    assert_eq!(framing_losses[0].member, Some(1));
    assert_eq!(framing_losses[0].error.record_index, 3);
    assert_eq!(
        framing_losses[0].error.byte_offset,
        (TRIM_ORDER_RECORD_LEN + 1) * 3
    );
    assert_eq!(framing_losses[0].error.unconsumed_raw, b"order-tail");
    assert_eq!(
        framing_losses[0].error.unconsumed_raw.as_ptr(),
        evidence.orders.streams[0]
            .replay
            .framing_error
            .expect("order framing loss")
            .unconsumed_raw
            .as_ptr()
    );

    assert_eq!(framing_losses[1].record_kind, CmRecordKind::Trades);
    assert_eq!(framing_losses[1].member, Some(1));
    assert_eq!(framing_losses[1].error.record_index, 2);
    assert_eq!(
        framing_losses[1].error.byte_offset,
        (TRIM_TRADE_RECORD_LEN + 1) * 2
    );
    assert_eq!(framing_losses[1].error.unconsumed_raw, b"trade-tail");
    assert_eq!(
        framing_losses[1].error.unconsumed_raw.as_ptr(),
        evidence.trades.streams[0]
            .source
            .framing_error
            .expect("trade framing loss")
            .unconsumed_raw
            .as_ptr()
    );

    let byte_accounting = evidence
        .source_byte_accounting()
        .expect("checked source byte accounting");
    assert_eq!(
        byte_accounting,
        reversed
            .source_byte_accounting()
            .expect("reversed source byte accounting")
    );
    assert!(
        byte_accounting
            .iter()
            .all(TrimSourceByteAccounting::counts_reconcile)
    );
    assert_eq!(
        byte_accounting,
        vec![
            TrimSourceByteAccounting {
                record_kind: CmRecordKind::Orders,
                member: Some(1),
                payload_width: TRIM_ORDER_RECORD_LEN,
                framed_records: 3,
                framed_bytes: (TRIM_ORDER_RECORD_LEN + 1) * 3,
                residue_bytes: b"order-tail".len(),
                source_bytes: (TRIM_ORDER_RECORD_LEN + 1) * 3 + b"order-tail".len(),
            },
            TrimSourceByteAccounting {
                record_kind: CmRecordKind::Orders,
                member: Some(2),
                payload_width: TRIM_ORDER_RECORD_LEN,
                framed_records: 1,
                framed_bytes: TRIM_ORDER_RECORD_LEN + 1,
                residue_bytes: 0,
                source_bytes: TRIM_ORDER_RECORD_LEN + 1,
            },
            TrimSourceByteAccounting {
                record_kind: CmRecordKind::Trades,
                member: Some(1),
                payload_width: TRIM_TRADE_RECORD_LEN,
                framed_records: 2,
                framed_bytes: (TRIM_TRADE_RECORD_LEN + 1) * 2,
                residue_bytes: b"trade-tail".len(),
                source_bytes: (TRIM_TRADE_RECORD_LEN + 1) * 2 + b"trade-tail".len(),
            },
            TrimSourceByteAccounting {
                record_kind: CmRecordKind::Trades,
                member: Some(2),
                payload_width: TRIM_TRADE_RECORD_LEN,
                framed_records: 0,
                framed_bytes: 0,
                residue_bytes: 0,
                source_bytes: 0,
            },
        ]
    );

    assert_eq!(
        TrimSourceByteAccounting::checked(CmRecordKind::Orders, Some(1), usize::MAX, 1, 0, 0,),
        Err(TrimSourceByteAccountingError::FramedBytesOverflow {
            record_kind: CmRecordKind::Orders,
            member: Some(1),
        })
    );
    assert_eq!(
        TrimSourceByteAccounting::checked(CmRecordKind::Trades, Some(2), 1, usize::MAX / 2, 2, 0,),
        Err(TrimSourceByteAccountingError::SourceBytesOverflow {
            record_kind: CmRecordKind::Trades,
            member: Some(2),
        })
    );
    assert_eq!(
        TrimSourceByteAccounting::checked(
            CmRecordKind::Orders,
            Some(1),
            TRIM_ORDER_RECORD_LEN,
            1,
            0,
            TRIM_ORDER_RECORD_LEN + 2,
        ),
        Err(TrimSourceByteAccountingError::SourceBytesMismatch {
            record_kind: CmRecordKind::Orders,
            member: Some(1),
            actual_source_bytes: TRIM_ORDER_RECORD_LEN + 2,
            accounted_source_bytes: TRIM_ORDER_RECORD_LEN + 1,
        })
    );

    let lifecycle_anomalies = evidence.lifecycle_anomaly_evidence();
    assert_eq!(lifecycle_anomalies, reversed.lifecycle_anomaly_evidence());
    assert_eq!(lifecycle_anomalies.len(), 1);
    assert_eq!(lifecycle_anomalies[0].member, Some(1));
    assert!(std::ptr::eq(
        lifecycle_anomalies[0].located,
        &evidence.orders.streams[0].replay.lifecycle_anomalies[0]
    ));
    assert_eq!(
        lifecycle_anomalies[0].located.coordinate,
        SourceCoordinate {
            record_index: 1,
            byte_offset: TRIM_ORDER_RECORD_LEN + 1,
        }
    );
    assert_eq!(
        lifecycle_anomalies[0].located.raw,
        ENTRY_FIXTURE[..TRIM_ORDER_RECORD_LEN]
    );
    assert!(matches!(
        lifecycle_anomalies[0].located.anomaly,
        jft_nse_cm::lifecycle::LifecycleAnomaly::DuplicateEntry { .. }
    ));
}

#[test]
fn retains_source_jiffy_regressions_in_the_same_evidence_pass() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);

    let mut lower_order = ENTRY_FIXTURE.to_vec();
    lower_order[22..36].copy_from_slice(b"91200611817220");
    let member_one_orders = [ENTRY_FIXTURE, lower_order.as_slice()].concat();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_orders,
        },
    ];

    let mut lower_trade = TRADE_FIXTURE.to_vec();
    lower_trade[6..20].copy_from_slice(b"91200611817220");
    let member_one_trades = [TRADE_FIXTURE, lower_trade.as_slice()].concat();
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_trades,
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");

    assert!(evidence.counts_reconcile());
    assert_eq!(
        evidence.orders.streams[0]
            .replay
            .jiffy_sequence
            .observation_count,
        2
    );
    assert_eq!(
        evidence.orders.streams[0]
            .replay
            .jiffy_sequence
            .regressions
            .len(),
        1
    );
    assert_eq!(
        evidence.trades.streams[0]
            .source
            .jiffy_sequence
            .observation_count,
        2
    );
    assert_eq!(
        evidence.trades.streams[0]
            .source
            .jiffy_sequence
            .regressions
            .len(),
        1
    );
    assert_eq!(
        evidence.observations(),
        vec![
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderDuplicateEntry,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderSourceJiffyRegression,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeZeroEntryCandidateReference,
                count: 2,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeEntryAfterTradeCandidate,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::TradeSourceJiffyRegression,
                count: 1,
            },
        ]
    );
}

#[test]
fn counts_every_documented_session_without_defining_transitions() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);

    let mut pre_open_order = ENTRY_FIXTURE.to_vec();
    pre_open_order[0..2].copy_from_slice(b"PO");
    let mut closing_order = ENTRY_FIXTURE.to_vec();
    closing_order[0..2].copy_from_slice(b"CA");
    let member_one_orders = [
        pre_open_order.as_slice(),
        ENTRY_FIXTURE,
        closing_order.as_slice(),
    ]
    .concat();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_orders,
        },
    ];

    let mut pre_open_trade = TRADE_FIXTURE.to_vec();
    pre_open_trade[0..2].copy_from_slice(b"PO");
    let mut closing_trade = TRADE_FIXTURE.to_vec();
    closing_trade[0..2].copy_from_slice(b"CA");
    let member_one_trades = [
        pre_open_trade.as_slice(),
        TRADE_FIXTURE,
        closing_trade.as_slice(),
    ]
    .concat();
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_trades,
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");

    assert!(evidence.counts_reconcile());
    assert_eq!(evidence.order_summary.session_counts.pre_open, 1);
    assert_eq!(evidence.order_summary.session_counts.regular_market, 1);
    assert_eq!(evidence.order_summary.session_counts.closing_auction, 1);
    assert_eq!(evidence.trades.combined_session_counts.pre_open, 1);
    assert_eq!(evidence.trades.combined_session_counts.regular_market, 1);
    assert_eq!(evidence.trades.combined_session_counts.closing_auction, 1);
}

#[test]
fn reports_cross_session_entry_occurrences_without_claiming_lifecycle_carry() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);

    let mut pre_open = ENTRY_FIXTURE.to_vec();
    pre_open[0..2].copy_from_slice(b"PO");
    let same_session_only = sell_entry();
    let member_one_orders = [
        ENTRY_FIXTURE,
        pre_open.as_slice(),
        same_session_only.as_slice(),
    ]
    .concat();
    let mut closing_auction = ENTRY_FIXTURE.to_vec();
    closing_auction[0..2].copy_from_slice(b"CA");
    let member_two_orders = [closing_auction.as_slice(), same_session_only.as_slice()].concat();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: &member_two_orders,
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_orders,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: b"",
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");
    let reversed = profile_trim_order_trade_evidence(
        &order_delivery,
        &[order_sources[1], order_sources[0]],
        &trade_delivery,
        &[trade_sources[1], trade_sources[0]],
    )
    .expect("reversed delivery evidence");

    assert!(evidence.counts_reconcile());
    assert_eq!(
        evidence.orders.cross_session_entry_evidence(),
        reversed.orders.cross_session_entry_evidence()
    );
    assert_eq!(
        evidence.orders.cross_session_entry_evidence(),
        vec![CrossSessionOrderEntryEvidence {
            identity: SessionExcludedOrderIdentity {
                segment: MarketSegment::Cash,
                symbol_raw: *b"  RELIANCE",
                series_raw: *b"EQ",
                order_number: 1,
            },
            occurrences: vec![
                SessionOrderEntryOccurrence {
                    member: Some(1),
                    session: Session::PreOpen,
                    entry: OrderEntryEvidence {
                        side: Side::Buy,
                        exchange_jiffy: ExchangeJiffy::new(91_200_611_817_221),
                        coordinate: SourceCoordinate {
                            record_index: 1,
                            byte_offset: TRIM_ORDER_RECORD_LEN + 1,
                        },
                    },
                },
                SessionOrderEntryOccurrence {
                    member: Some(1),
                    session: Session::RegularMarket,
                    entry: OrderEntryEvidence {
                        side: Side::Buy,
                        exchange_jiffy: ExchangeJiffy::new(91_200_611_817_221),
                        coordinate: SourceCoordinate {
                            record_index: 0,
                            byte_offset: 0,
                        },
                    },
                },
                SessionOrderEntryOccurrence {
                    member: Some(2),
                    session: Session::ClosingAuction,
                    entry: OrderEntryEvidence {
                        side: Side::Buy,
                        exchange_jiffy: ExchangeJiffy::new(91_200_611_817_221),
                        coordinate: SourceCoordinate {
                            record_index: 0,
                            byte_offset: 0,
                        },
                    },
                },
            ],
        }]
    );
}

#[test]
fn reports_cross_session_lifecycle_candidates_without_applying_them() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);

    let mut pre_open_modify = MODIFY_FIXTURE.to_vec();
    pre_open_modify[0..2].copy_from_slice(b"PO");
    let mut closing_auction_cancel = CANCEL_FIXTURE.to_vec();
    closing_auction_cancel[0..2].copy_from_slice(b"CA");
    let member_one_orders = [
        ENTRY_FIXTURE,
        pre_open_modify.as_slice(),
        closing_auction_cancel.as_slice(),
    ]
    .concat();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: MODIFY_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_orders,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: b"",
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");
    let reversed = profile_trim_order_trade_evidence(
        &order_delivery,
        &[order_sources[1], order_sources[0]],
        &trade_delivery,
        &[trade_sources[1], trade_sources[0]],
    )
    .expect("reversed delivery evidence");

    assert!(evidence.counts_reconcile());
    assert_eq!(evidence.order_summary.applied_events, 1);
    assert_eq!(evidence.order_summary.lifecycle_anomaly_count, 3);
    let candidates = evidence.orders.cross_session_lifecycle_candidates();
    assert_eq!(
        candidates,
        reversed.orders.cross_session_lifecycle_candidates()
    );
    assert_eq!(candidates.len(), 2);

    let candidate = &candidates[0];
    assert_eq!(candidate.anomaly_member, Some(1));
    assert_eq!(
        candidate.anomaly.coordinate,
        SourceCoordinate {
            record_index: 1,
            byte_offset: TRIM_ORDER_RECORD_LEN + 1,
        }
    );
    assert_eq!(candidate.anomaly.event.session, Session::PreOpen);
    assert_eq!(candidate.anomaly.event.activity, OrderActivity::Modify);
    assert_eq!(
        candidate.anomaly.raw,
        pre_open_modify[..TRIM_ORDER_RECORD_LEN]
    );
    assert!(matches!(
        candidate.anomaly.anomaly,
        jft_nse_cm::lifecycle::LifecycleAnomaly::UnknownModify { .. }
    ));
    assert_eq!(
        candidate.other_session_entries,
        vec![SessionOrderEntryOccurrence {
            member: Some(1),
            session: Session::RegularMarket,
            entry: OrderEntryEvidence {
                side: Side::Buy,
                exchange_jiffy: ExchangeJiffy::new(91_200_611_817_221),
                coordinate: SourceCoordinate {
                    record_index: 0,
                    byte_offset: 0,
                },
            },
        }]
    );

    let cancel_candidate = &candidates[1];
    assert_eq!(cancel_candidate.anomaly_member, Some(1));
    assert_eq!(
        cancel_candidate.anomaly.coordinate,
        SourceCoordinate {
            record_index: 2,
            byte_offset: (TRIM_ORDER_RECORD_LEN + 1) * 2,
        }
    );
    assert_eq!(
        cancel_candidate.anomaly.event.session,
        Session::ClosingAuction
    );
    assert_eq!(
        cancel_candidate.anomaly.event.activity,
        OrderActivity::Cancel
    );
    assert_eq!(
        cancel_candidate.anomaly.raw,
        closing_auction_cancel[..TRIM_ORDER_RECORD_LEN]
    );
    assert!(matches!(
        cancel_candidate.anomaly.anomaly,
        jft_nse_cm::lifecycle::LifecycleAnomaly::UnknownCancel { .. }
    ));
    assert_eq!(
        cancel_candidate.other_session_entries,
        candidate.other_session_entries
    );

    assert_eq!(
        evidence.observations(),
        vec![
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderUnknownModify,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderUnknownCancel,
                count: 1,
            },
            TrimEvidenceObservation {
                member: Some(1),
                kind: TrimEvidenceObservationKind::OrderCrossSessionLifecycleCandidate,
                count: 2,
            },
            TrimEvidenceObservation {
                member: Some(2),
                kind: TrimEvidenceObservationKind::OrderUnknownModify,
                count: 1,
            },
        ]
    );
    assert_eq!(
        evidence.orders.cross_session_lifecycle_route_counts(),
        vec![
            CrossSessionLifecycleRouteCount {
                route: CrossSessionLifecycleRoute {
                    operation: CrossSessionLifecycleOperation::Modify,
                    anomaly_session: Session::PreOpen,
                    candidate_entry_session: Session::RegularMarket,
                },
                candidate_event_count: 1,
                entry_candidate_count: 1,
                entry_before_event_candidates: 1,
                entry_equal_event_candidates: 0,
                entry_after_event_candidates: 0,
            },
            CrossSessionLifecycleRouteCount {
                route: CrossSessionLifecycleRoute {
                    operation: CrossSessionLifecycleOperation::Cancel,
                    anomaly_session: Session::ClosingAuction,
                    candidate_entry_session: Session::RegularMarket,
                },
                candidate_event_count: 1,
                entry_candidate_count: 1,
                entry_before_event_candidates: 1,
                entry_equal_event_candidates: 0,
                entry_after_event_candidates: 0,
            },
        ]
    );

    let regular_market_unknown = &evidence.orders.streams[1].replay.lifecycle_anomalies[0];
    assert_eq!(regular_market_unknown.event.session, Session::RegularMarket);
    assert_eq!(regular_market_unknown.event.activity, OrderActivity::Modify);
}

#[test]
fn route_counts_distinguish_candidate_events_from_entry_multiplicity() {
    let date = source_date(18);
    let mut order_delivery = delivery(CmRecordKind::Orders, date);
    order_delivery.members.push(Some(3));
    let mut trade_delivery = delivery(CmRecordKind::Trades, date);
    trade_delivery.members.push(Some(3));

    let mut pre_open_modify = MODIFY_FIXTURE.to_vec();
    pre_open_modify[0..2].copy_from_slice(b"PO");
    let member_one_orders = [ENTRY_FIXTURE, pre_open_modify.as_slice()].concat();
    let mut equal_jiffy_entry = ENTRY_FIXTURE.to_vec();
    equal_jiffy_entry[22..36].copy_from_slice(b"91200611817223");
    let mut after_jiffy_entry = ENTRY_FIXTURE.to_vec();
    after_jiffy_entry[22..36].copy_from_slice(b"91200611817224");
    let order_sources = [
        CmStreamSource {
            member: Some(3),
            source: &after_jiffy_entry,
        },
        CmStreamSource {
            member: Some(2),
            source: &equal_jiffy_entry,
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_orders,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(3),
            source: b"",
        },
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: b"",
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");

    assert!(evidence.counts_reconcile());
    let route_counts = evidence.orders.cross_session_lifecycle_route_counts();
    assert!(route_counts.iter().all(|count| count.counts_reconcile()));
    assert_eq!(
        route_counts,
        vec![CrossSessionLifecycleRouteCount {
            route: CrossSessionLifecycleRoute {
                operation: CrossSessionLifecycleOperation::Modify,
                anomaly_session: Session::PreOpen,
                candidate_entry_session: Session::RegularMarket,
            },
            candidate_event_count: 1,
            entry_candidate_count: 3,
            entry_before_event_candidates: 1,
            entry_equal_event_candidates: 1,
            entry_after_event_candidates: 1,
        }]
    );

    let mut invalid = route_counts[0];
    invalid.entry_equal_event_candidates = 2;
    assert!(!invalid.counts_reconcile());
    invalid = route_counts[0];
    invalid.candidate_event_count = 0;
    assert!(!invalid.counts_reconcile());
    invalid = route_counts[0];
    invalid.candidate_event_count = 4;
    assert!(!invalid.counts_reconcile());
}

#[test]
fn quarantine_classes_preserve_exact_failure_and_jiffy_availability() {
    let date = source_date(18);
    let order_delivery = delivery(CmRecordKind::Orders, date);
    let trade_delivery = delivery(CmRecordKind::Trades, date);

    let mut invalid_activity = ENTRY_FIXTURE.to_vec();
    invalid_activity[37] = b'2';
    let repeated_activity = [invalid_activity.as_slice(), invalid_activity.as_slice()].concat();
    let mut invalid_jiffy = ENTRY_FIXTURE.to_vec();
    invalid_jiffy[22] = b'X';
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: &invalid_jiffy,
        },
        CmStreamSource {
            member: Some(1),
            source: &repeated_activity,
        },
    ];
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: b"",
        },
        CmStreamSource {
            member: Some(1),
            source: b"",
        },
    ];

    let evidence = profile_trim_order_trade_evidence(
        &order_delivery,
        &order_sources,
        &trade_delivery,
        &trade_sources,
    )
    .expect("delivery evidence");
    let reversed = profile_trim_order_trade_evidence(
        &order_delivery,
        &[order_sources[1], order_sources[0]],
        &trade_delivery,
        &[trade_sources[1], trade_sources[0]],
    )
    .expect("reversed delivery evidence");

    assert!(evidence.counts_reconcile());
    let classes = evidence.quarantine_class_counts();
    assert_eq!(classes, reversed.quarantine_class_counts());
    assert_eq!(classes.len(), 2);
    assert!(classes.iter().all(|class| class.counts_reconcile()));

    assert_eq!(classes[0].record_kind, CmRecordKind::Orders);
    assert_eq!(classes[0].member, Some(1));
    assert_eq!(classes[0].field.expect("activity field").name, "activity");
    assert!(matches!(
        classes[0].reason,
        QuarantineReason::UnknownValue { .. }
    ));
    assert_eq!(classes[0].count, 2);
    assert_eq!(classes[0].validated_raw_jiffy_count, 2);

    assert_eq!(classes[1].record_kind, CmRecordKind::Orders);
    assert_eq!(classes[1].member, Some(2));
    assert_eq!(
        classes[1].field.expect("exchange jiffy field").name,
        "exchange_jiffy"
    );
    assert_eq!(classes[1].reason, &QuarantineReason::InvalidUnsignedInteger);
    assert_eq!(classes[1].count, 1);
    assert_eq!(classes[1].validated_raw_jiffy_count, 0);

    let mut invalid_count = classes[0];
    invalid_count.count = 0;
    assert!(!invalid_count.counts_reconcile());
    invalid_count = classes[0];
    invalid_count.validated_raw_jiffy_count = 3;
    assert!(!invalid_count.counts_reconcile());
}
