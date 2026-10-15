use jft_nse_cm::events::{MarketSegment, OrderActivity, Session, Side};
use jft_nse_cm::layouts::{TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::parser::{parse_trim_order, parse_trim_trade};
use jft_nse_cm::quarantine::QuarantineReason;

const ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

fn record_without_lf<const N: usize>(fixture: &[u8]) -> [u8; N] {
    assert_eq!(fixture.len(), N + 1);
    assert_eq!(fixture[N], b'\n');
    fixture[..N].try_into().expect("fixture width was checked")
}

#[test]
fn parses_trim_order_and_preserves_exact_bytes() {
    let raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);
    let parsed = parse_trim_order(&raw).expect("valid synthetic order");

    assert_eq!(parsed.raw, raw);
    assert_eq!(parsed.event.session, Session::RegularMarket);
    assert_eq!(parsed.event.segment, MarketSegment::Cash);
    assert_eq!(parsed.event.order_number, 1);
    assert_eq!(parsed.event.exchange_jiffy.raw(), 91_200_611_817_221);
    assert_eq!(parsed.event.side, Side::Buy);
    assert_eq!(parsed.event.activity, OrderActivity::Entry);
    assert_eq!(parsed.event.symbol_raw, *b"  RELIANCE");
    assert_eq!(parsed.event.symbol, "RELIANCE");
    assert_eq!(parsed.event.series, "EQ");
    assert_eq!(parsed.event.quantity, 100);
    assert_eq!(parsed.event.price_paise, 250_000);
}

#[test]
fn parses_trim_trade_and_preserves_order_references() {
    let raw = record_without_lf::<TRIM_TRADE_RECORD_LEN>(TRADE_FIXTURE);
    let parsed = parse_trim_trade(&raw).expect("valid synthetic trade");

    assert_eq!(parsed.raw, raw);
    assert_eq!(parsed.event.session, Session::RegularMarket);
    assert_eq!(parsed.event.exchange_jiffy.raw(), 91_200_611_817_222);
    assert_eq!(parsed.event.symbol, "RELIANCE");
    assert_eq!(parsed.event.price_paise, 250_000);
    assert_eq!(parsed.event.quantity, 25);
    assert_eq!(parsed.event.buy_order_number, 1);
    assert_eq!(parsed.event.sell_order_number, 2);
}

#[test]
fn quarantines_wrong_width_without_losing_input() {
    let invalid = b"too-short";
    let error = parse_trim_order(invalid).expect_err("wrong width must be quarantined");

    assert_eq!(error.raw, invalid);
    assert_eq!(error.field, None);
    assert_eq!(
        error.reason,
        QuarantineReason::WrongRecordLength {
            expected: TRIM_ORDER_RECORD_LEN,
            actual: invalid.len(),
        }
    );
}

#[test]
fn quarantines_unknown_activity_at_the_exact_field() {
    let mut raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);
    raw[37] = b'2';
    let error = parse_trim_order(&raw).expect_err("unknown activity must be quarantined");

    let field = error.field.expect("field location");
    assert_eq!(field.name, "activity");
    assert_eq!(field.offset, 37);
    assert_eq!(error.raw, raw);
    assert_eq!(
        error.reason,
        QuarantineReason::UnknownValue {
            expected: "1, 3, or 4"
        }
    );
}

#[test]
fn quarantines_non_numeric_quantity() {
    let mut raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);
    raw[55] = b'X';
    let error = parse_trim_order(&raw).expect_err("non-numeric quantity must be quarantined");

    assert_eq!(error.field.expect("field location").name, "quantity");
    assert_eq!(error.reason, QuarantineReason::InvalidUnsignedInteger);
}

#[test]
fn quarantines_zero_trade_quantity() {
    let mut raw = record_without_lf::<TRIM_TRADE_RECORD_LEN>(TRADE_FIXTURE);
    raw[40..50].fill(b'0');
    let error = parse_trim_trade(&raw).expect_err("zero quantity must be quarantined");

    assert_eq!(error.field.expect("field location").name, "quantity");
    assert_eq!(error.reason, QuarantineReason::ZeroNotAllowed);
}

#[test]
fn recognizes_all_documented_sessions() {
    let mut raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);

    raw[0..2].copy_from_slice(b"PO");
    assert_eq!(
        parse_trim_order(&raw).expect("PO").event.session,
        Session::PreOpen
    );

    raw[0..2].copy_from_slice(b"CA");
    assert_eq!(
        parse_trim_order(&raw).expect("CA").event.session,
        Session::ClosingAuction
    );
}
