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

#[test]
fn recognizes_all_documented_sides_and_activities() {
    let mut raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);

    raw[36] = b'S';
    assert_eq!(parse_trim_order(&raw).expect("sell").event.side, Side::Sell);

    raw[37] = b'3';
    assert_eq!(
        parse_trim_order(&raw).expect("cancel").event.activity,
        OrderActivity::Cancel
    );

    raw[37] = b'4';
    assert_eq!(
        parse_trim_order(&raw).expect("modify").event.activity,
        OrderActivity::Modify
    );
}

#[test]
fn every_order_numeric_byte_is_strictly_validated() {
    let fields = [
        (6, 22, "order_number"),
        (22, 36, "exchange_jiffy"),
        (50, 60, "quantity"),
        (60, 68, "price_paise"),
    ];

    for (start, end, name) in fields {
        for position in start..end {
            let mut raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);
            raw[position] = b'X';
            let error = parse_trim_order(&raw).expect_err("non-digit must fail");
            assert_eq!(error.field.expect("field location").name, name);
            assert_eq!(error.reason, QuarantineReason::InvalidUnsignedInteger);
        }
    }
}

#[test]
fn every_trade_numeric_byte_is_strictly_validated() {
    let fields = [
        (6, 20, "exchange_jiffy"),
        (32, 40, "price_paise"),
        (40, 50, "quantity"),
        (50, 66, "buy_order_number"),
        (66, 82, "sell_order_number"),
    ];

    for (start, end, name) in fields {
        for position in start..end {
            let mut raw = record_without_lf::<TRIM_TRADE_RECORD_LEN>(TRADE_FIXTURE);
            raw[position] = b'X';
            let error = parse_trim_trade(&raw).expect_err("non-digit must fail");
            assert_eq!(error.field.expect("field location").name, name);
            assert_eq!(error.reason, QuarantineReason::InvalidUnsignedInteger);
        }
    }
}

#[test]
fn quarantines_unknown_session_segment_and_side() {
    let base = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);

    let mut unknown_session = base;
    unknown_session[0..2].copy_from_slice(b"XX");
    let error = parse_trim_order(&unknown_session).expect_err("unknown session");
    assert_eq!(
        error.field.expect("field location").name,
        "record_indicator"
    );

    let mut unknown_segment = base;
    unknown_segment[2..6].copy_from_slice(b"FAOO");
    let error = parse_trim_order(&unknown_segment).expect_err("unknown segment");
    assert_eq!(error.field.expect("field location").name, "segment");

    let mut unknown_side = base;
    unknown_side[36] = b'X';
    let error = parse_trim_order(&unknown_side).expect_err("unknown side");
    assert_eq!(error.field.expect("field location").name, "side");
}

#[test]
fn quarantines_zero_prices_and_order_quantity() {
    let order = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);

    let mut zero_order_quantity = order;
    zero_order_quantity[50..60].fill(b'0');
    let error = parse_trim_order(&zero_order_quantity).expect_err("zero order quantity");
    assert_eq!(error.field.expect("field location").name, "quantity");
    assert_eq!(error.reason, QuarantineReason::ZeroNotAllowed);

    let mut zero_order_price = order;
    zero_order_price[60..68].fill(b'0');
    let error = parse_trim_order(&zero_order_price).expect_err("zero order price");
    assert_eq!(error.field.expect("field location").name, "price_paise");
    assert_eq!(error.reason, QuarantineReason::ZeroNotAllowed);

    let mut zero_trade_price = record_without_lf::<TRIM_TRADE_RECORD_LEN>(TRADE_FIXTURE);
    zero_trade_price[32..40].fill(b'0');
    let error = parse_trim_trade(&zero_trade_price).expect_err("zero trade price");
    assert_eq!(error.field.expect("field location").name, "price_paise");
    assert_eq!(error.reason, QuarantineReason::ZeroNotAllowed);
}

#[test]
fn rejects_non_ascii_text_without_losing_the_field() {
    let mut symbol = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);
    symbol[38] = 0xff;
    let error = parse_trim_order(&symbol).expect_err("non-ASCII symbol");
    assert_eq!(error.field.expect("field location").name, "symbol");
    assert_eq!(error.reason, QuarantineReason::NonAscii);

    let mut series = record_without_lf::<TRIM_TRADE_RECORD_LEN>(TRADE_FIXTURE);
    series[30] = 0xff;
    let error = parse_trim_trade(&series).expect_err("non-ASCII series");
    assert_eq!(error.field.expect("field location").name, "series");
    assert_eq!(error.reason, QuarantineReason::NonAscii);
}

#[test]
fn literal_b_padding_is_preserved_and_not_guessed_as_blank() {
    let mut raw = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);
    raw[38..48].copy_from_slice(b"bbRELIANCE");
    let parsed = parse_trim_order(&raw).expect("literal b is valid ASCII");

    assert_eq!(parsed.event.symbol_raw, *b"bbRELIANCE");
    assert_eq!(parsed.event.symbol, "bbRELIANCE");
}

#[test]
fn every_single_byte_order_mutation_preserves_the_exact_input() {
    let base = record_without_lf::<TRIM_ORDER_RECORD_LEN>(ORDER_FIXTURE);

    for position in 0..TRIM_ORDER_RECORD_LEN {
        for value in u8::MIN..=u8::MAX {
            let mut raw = base;
            raw[position] = value;
            match parse_trim_order(&raw) {
                Ok(parsed) => assert_eq!(parsed.raw, raw),
                Err(quarantined) => assert_eq!(quarantined.raw, raw),
            }
        }
    }
}

#[test]
fn every_single_byte_trade_mutation_preserves_the_exact_input() {
    let base = record_without_lf::<TRIM_TRADE_RECORD_LEN>(TRADE_FIXTURE);

    for position in 0..TRIM_TRADE_RECORD_LEN {
        for value in u8::MIN..=u8::MAX {
            let mut raw = base;
            raw[position] = value;
            match parse_trim_trade(&raw) {
                Ok(parsed) => assert_eq!(parsed.raw, raw),
                Err(quarantined) => assert_eq!(quarantined.raw, raw),
            }
        }
    }
}

#[test]
fn every_tested_wrong_order_width_is_losslessly_quarantined_before_field_access() {
    for actual in 0..=TRIM_ORDER_RECORD_LEN + 32 {
        if actual == TRIM_ORDER_RECORD_LEN {
            continue;
        }

        let raw = vec![b'X'; actual];
        let error = parse_trim_order(&raw).expect_err("wrong width must fail");
        assert_eq!(error.raw, raw);
        assert_eq!(error.field, None);
        assert_eq!(
            error.reason,
            QuarantineReason::WrongRecordLength {
                expected: TRIM_ORDER_RECORD_LEN,
                actual,
            }
        );
    }
}

#[test]
fn every_tested_wrong_trade_width_is_losslessly_quarantined_before_field_access() {
    for actual in 0..=TRIM_TRADE_RECORD_LEN + 32 {
        if actual == TRIM_TRADE_RECORD_LEN {
            continue;
        }

        let raw = vec![b'X'; actual];
        let error = parse_trim_trade(&raw).expect_err("wrong width must fail");
        assert_eq!(error.raw, raw);
        assert_eq!(error.field, None);
        assert_eq!(
            error.reason,
            QuarantineReason::WrongRecordLength {
                expected: TRIM_TRADE_RECORD_LEN,
                actual,
            }
        );
    }
}

#[test]
fn rejects_a_record_that_still_contains_the_lf_delimiter() {
    let error = parse_trim_order(ORDER_FIXTURE).expect_err("parser accepts payload only");

    assert_eq!(
        error.reason,
        QuarantineReason::WrongRecordLength {
            expected: TRIM_ORDER_RECORD_LEN,
            actual: TRIM_ORDER_RECORD_LEN + 1,
        }
    );
    assert_eq!(error.raw, ORDER_FIXTURE);
}
