use jft_nse_cm::layouts::SourceDate;
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind,
};
use jft_nse_cm::reconcile::{
    CmStreamSource, CmStreamSourceSetError, TradeCandidateStreamSetError,
    TrimOrderTradeEvidenceError, profile_trim_order_trade_evidence,
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
