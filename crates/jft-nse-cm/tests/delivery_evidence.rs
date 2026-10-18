use jft_nse_cm::layouts::SourceDate;
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind,
};
use jft_nse_cm::reconcile::{
    CmStreamSource, CmStreamSourceSetError, TradeCandidateStreamSetError, TrimEvidenceObservation,
    TrimEvidenceObservationKind, TrimOrderTradeEvidenceError, profile_trim_order_trade_evidence,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
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
