use jft_nse_cm::framing::FixedWidthLfRecords;
use jft_nse_cm::layouts::{
    CURRENT_FULL_ORDER_FIELDS, CURRENT_FULL_ORDER_RECORD_LEN, CURRENT_FULL_TRADE_FIELDS,
    CURRENT_FULL_TRADE_RECORD_LEN, FieldLayout,
};

const FULL_ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/full_order_entry.dat");
const FULL_TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/full_trade.dat");

fn assert_field_bytes<const N: usize>(raw: &[u8; N], fields: &[FieldLayout], expected: &[&[u8]]) {
    assert_eq!(fields.len(), expected.len());
    for (field, expected_bytes) in fields.iter().zip(expected) {
        assert_eq!(
            &raw[field.offset..field.end()],
            *expected_bytes,
            "wrong bytes for {}",
            field.name
        );
    }
}

#[test]
fn full_order_fixture_exercises_every_documented_field_boundary() {
    assert_eq!(FULL_ORDER_FIXTURE.len(), CURRENT_FULL_ORDER_RECORD_LEN + 1);
    assert_eq!(FULL_ORDER_FIXTURE[CURRENT_FULL_ORDER_RECORD_LEN], b'\n');

    let framed = FixedWidthLfRecords::<CURRENT_FULL_ORDER_RECORD_LEN>::new(FULL_ORDER_FIXTURE)
        .next()
        .expect("one fixture record")
        .expect("valid Full Order framing");

    assert_field_bytes(
        framed.raw,
        &CURRENT_FULL_ORDER_FIELDS,
        &[
            b"RM",
            b"CASH",
            b"0000000000000001",
            b"91200611817221",
            b"B",
            b"1",
            b"  RELIANCE",
            b"EQ",
            b"0000000000",
            b"0000000100",
            b"00250000",
            b"00000000",
            b"N",
            b"N",
            b"N",
            b"0",
            b"2",
        ],
    );
}

#[test]
fn full_trade_fixture_exercises_every_documented_field_boundary() {
    assert_eq!(FULL_TRADE_FIXTURE.len(), CURRENT_FULL_TRADE_RECORD_LEN + 1);
    assert_eq!(FULL_TRADE_FIXTURE[CURRENT_FULL_TRADE_RECORD_LEN], b'\n');

    let framed = FixedWidthLfRecords::<CURRENT_FULL_TRADE_RECORD_LEN>::new(FULL_TRADE_FIXTURE)
        .next()
        .expect("one fixture record")
        .expect("valid Full Trade framing");

    assert_field_bytes(
        framed.raw,
        &CURRENT_FULL_TRADE_FIELDS,
        &[
            b"RM",
            b"CASH",
            b"00000000000000001",
            b"91200611817222",
            b"  RELIANCE",
            b"EQ",
            b"00250000",
            b"0000000025",
            b"0000000000000001",
            b"0",
            b"2",
            b"0000000000000002",
            b"1",
            b"3",
        ],
    );
}
