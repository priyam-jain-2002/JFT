use jft_nse_cm::events::{Session, Side};
use jft_nse_cm::layouts::{TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::parser::parse_trim_trade;
use jft_nse_cm::reconcile::{
    OrderEntryEvidence, OrderSourceReference, SourceCoordinate, reconcile_trim_trade_references,
    replay_trim_order_source,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const CANCEL_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_cancel.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

fn sell_entry() -> Vec<u8> {
    let mut sell = ENTRY_FIXTURE.to_vec();
    sell[6..22].copy_from_slice(b"0000000000000002");
    sell[22..36].copy_from_slice(b"91200611817222");
    sell[36] = b'S';
    sell
}

fn trade() -> jft_nse_cm::events::TradeEvent {
    parse_trim_trade(&TRADE_FIXTURE[..TRIM_TRADE_RECORD_LEN])
        .expect("synthetic trade")
        .event
}

#[test]
fn matches_both_trade_sides_to_exact_entry_coordinates() {
    let sell = sell_entry();
    let source = [ENTRY_FIXTURE, sell.as_slice()].concat();
    let orders = replay_trim_order_source(&source);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(matches!(
        references.buy,
        OrderSourceReference::EntryMatch {
            entry: OrderEntryEvidence {
                side: Side::Buy,
                coordinate: SourceCoordinate {
                    record_index: 0,
                    byte_offset: 0,
                },
            },
            ..
        }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::EntryMatch {
            entry: OrderEntryEvidence {
                side: Side::Sell,
                coordinate: SourceCoordinate {
                    record_index: 1,
                    byte_offset,
                },
            },
            ..
        } if byte_offset == TRIM_ORDER_RECORD_LEN + 1
    ));
}

#[test]
fn missing_sell_reference_does_not_hide_the_matched_buy_reference() {
    let orders = replay_trim_order_source(ENTRY_FIXTURE);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(matches!(
        references.buy,
        OrderSourceReference::EntryMatch { .. }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::Missing {
            expected_side: Side::Sell,
            ..
        }
    ));
}

#[test]
fn entry_side_mismatch_is_distinct_from_a_missing_reference() {
    let mut wrong_side = sell_entry();
    wrong_side[36] = b'B';
    let source = [ENTRY_FIXTURE, wrong_side.as_slice()].concat();
    let orders = replay_trim_order_source(&source);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(matches!(
        references.sell,
        OrderSourceReference::EntrySideMismatch {
            expected_side: Side::Sell,
            entry: OrderEntryEvidence {
                side: Side::Buy,
                ..
            },
            ..
        }
    ));
}

#[test]
fn session_and_raw_instrument_identity_are_required_for_a_match() {
    let sell = sell_entry();
    let source = [ENTRY_FIXTURE, sell.as_slice()].concat();
    let orders = replay_trim_order_source(&source);

    let mut wrong_session = trade();
    wrong_session.session = Session::PreOpen;
    let references = reconcile_trim_trade_references(&orders, &wrong_session);
    assert!(matches!(
        references.buy,
        OrderSourceReference::Missing { .. }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::Missing { .. }
    ));

    let mut wrong_symbol = trade();
    wrong_symbol.symbol_raw = *b"       TCS";
    wrong_symbol.symbol = "TCS".to_owned();
    let references = reconcile_trim_trade_references(&orders, &wrong_symbol);
    assert!(matches!(
        references.buy,
        OrderSourceReference::Missing { .. }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::Missing { .. }
    ));
}

#[test]
fn cancelled_order_remains_an_entry_source_match_without_temporal_claim() {
    let source = [ENTRY_FIXTURE, CANCEL_FIXTURE].concat();
    let orders = replay_trim_order_source(&source);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(orders.lifecycle.is_empty());
    assert!(matches!(
        references.buy,
        OrderSourceReference::EntryMatch { .. }
    ));
}

#[test]
fn repeated_reference_reconciliation_is_identical() {
    let sell = sell_entry();
    let source = [ENTRY_FIXTURE, sell.as_slice()].concat();
    let orders = replay_trim_order_source(&source);
    let event = trade();

    assert_eq!(
        reconcile_trim_trade_references(&orders, &event),
        reconcile_trim_trade_references(&orders, &event)
    );
}
