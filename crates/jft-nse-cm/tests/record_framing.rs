use jft_nse_cm::framing::{FixedWidthLfRecords, FramingErrorReason};
use jft_nse_cm::layouts::{TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::parser::parse_trim_trade;

const ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

#[test]
fn frames_multiple_records_with_exact_indices_offsets_and_bytes() {
    let source = [ORDER_FIXTURE, ORDER_FIXTURE].concat();
    let records = FixedWidthLfRecords::<TRIM_ORDER_RECORD_LEN>::new(&source)
        .collect::<Result<Vec<_>, _>>()
        .expect("two exact records");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].record_index, 0);
    assert_eq!(records[0].byte_offset, 0);
    assert_eq!(records[0].raw.as_slice(), &ORDER_FIXTURE[..68]);
    assert_eq!(records[1].record_index, 1);
    assert_eq!(records[1].byte_offset, 69);
    assert_eq!(records[1].raw.as_slice(), &ORDER_FIXTURE[..68]);

    let accounted_bytes = records.len() * (TRIM_ORDER_RECORD_LEN + 1);
    assert_eq!(accounted_bytes, source.len());
}

#[test]
fn frames_trade_payload_without_passing_the_lf_to_the_parser() {
    let record = FixedWidthLfRecords::<TRIM_TRADE_RECORD_LEN>::new(TRADE_FIXTURE)
        .next()
        .expect("one result")
        .expect("valid framing");
    let parsed = parse_trim_trade(record.raw).expect("framed payload must parse");

    assert_eq!(record.raw.len(), TRIM_TRADE_RECORD_LEN);
    assert_eq!(record.raw.as_slice(), &TRADE_FIXTURE[..82]);
    assert_eq!(TRADE_FIXTURE[82], b'\n');
    assert_eq!(parsed.raw.as_slice(), record.raw.as_slice());
    assert_eq!(parsed.event.exchange_jiffy.raw(), 91_200_611_817_222);
}

#[test]
fn accepts_an_empty_source_as_zero_records() {
    assert_eq!(FixedWidthLfRecords::<68>::new(&[]).count(), 0);
}

#[test]
fn preserves_a_trailing_payload_that_has_no_lf() {
    let source = &ORDER_FIXTURE[..TRIM_ORDER_RECORD_LEN];
    let mut records = FixedWidthLfRecords::<TRIM_ORDER_RECORD_LEN>::new(source);
    let error = records
        .next()
        .expect("one result")
        .expect_err("missing LF must fail");

    assert_eq!(error.record_index, 0);
    assert_eq!(error.byte_offset, 0);
    assert_eq!(error.unconsumed_raw, source);
    assert_eq!(
        error.reason,
        FramingErrorReason::TrailingPartialRecord {
            expected_payload_bytes: 68,
            expected_delimiter_bytes: 1,
            actual_remaining_bytes: 68,
        }
    );
    assert!(records.next().is_none(), "must not guess how to resync");
}

#[test]
fn rejects_crlf_and_preserves_the_unconsumed_tail() {
    let mut source = ORDER_FIXTURE.to_vec();
    source[TRIM_ORDER_RECORD_LEN] = b'\r';
    source.push(b'\n');
    let mut records = FixedWidthLfRecords::<TRIM_ORDER_RECORD_LEN>::new(&source);
    let error = records
        .next()
        .expect("one result")
        .expect_err("CRLF is not the documented framing");

    assert_eq!(error.record_index, 0);
    assert_eq!(error.byte_offset, 0);
    assert_eq!(error.unconsumed_raw, source);
    assert_eq!(
        error.reason,
        FramingErrorReason::UnexpectedDelimiter {
            expected: b'\n',
            actual: b'\r',
            relative_offset: 68,
        }
    );
    assert!(records.next().is_none(), "must not guess how to resync");
}

#[test]
fn reports_the_first_unaccounted_byte_after_valid_records() {
    let mut source = ORDER_FIXTURE.to_vec();
    source.extend_from_slice(b"partial");
    let mut records = FixedWidthLfRecords::<TRIM_ORDER_RECORD_LEN>::new(&source);

    let valid = records.next().expect("valid result").expect("first record");
    assert_eq!(valid.record_index, 0);
    assert_eq!(valid.byte_offset, 0);

    let error = records
        .next()
        .expect("partial result")
        .expect_err("partial tail must fail");
    assert_eq!(error.record_index, 1);
    assert_eq!(error.byte_offset, ORDER_FIXTURE.len());
    assert_eq!(error.unconsumed_raw, b"partial");
    assert_eq!(
        error.reason,
        FramingErrorReason::TrailingPartialRecord {
            expected_payload_bytes: 68,
            expected_delimiter_bytes: 1,
            actual_remaining_bytes: 7,
        }
    );
}
