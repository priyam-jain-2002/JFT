use jft_nse_cm::events::OrderActivity;
use jft_nse_cm::framing::FramingErrorReason;
use jft_nse_cm::layouts::{TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::parser::{parse_trim_order_source, parse_trim_trade_source};

const ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

#[test]
fn accounts_for_every_framed_record_and_continues_after_field_quarantine() {
    let mut invalid = ORDER_FIXTURE.to_vec();
    invalid[37] = b'2';
    let source = [ORDER_FIXTURE, invalid.as_slice(), ORDER_FIXTURE].concat();

    let records = parse_trim_order_source(&source)
        .collect::<Result<Vec<_>, _>>()
        .expect("the complete source is framed");

    assert_eq!(records.len(), 3);
    assert_eq!(
        records
            .iter()
            .filter(|record| record.result.is_ok())
            .count(),
        2
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.result.is_err())
            .count(),
        1
    );

    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.record_index, index);
        assert_eq!(record.byte_offset, index * (TRIM_ORDER_RECORD_LEN + 1));
    }

    assert_eq!(
        records[0]
            .result
            .as_ref()
            .expect("first record")
            .event
            .activity,
        OrderActivity::Entry
    );
    assert_eq!(
        records[1]
            .result
            .as_ref()
            .expect_err("invalid activity")
            .raw,
        invalid[..TRIM_ORDER_RECORD_LEN]
    );
    assert_eq!(
        records[2]
            .result
            .as_ref()
            .expect("third record")
            .raw
            .as_slice(),
        &ORDER_FIXTURE[..TRIM_ORDER_RECORD_LEN]
    );
}

#[test]
fn stops_at_a_framing_error_and_preserves_the_unknown_tail() {
    let mut source = ORDER_FIXTURE.to_vec();
    source.extend_from_slice(b"unknown-tail");

    let mut records = parse_trim_order_source(&source);
    let valid = records.next().expect("first result").expect("first frame");
    assert!(valid.result.is_ok());

    let error = records
        .next()
        .expect("tail result")
        .expect_err("partial tail must be reported");
    assert_eq!(error.record_index, 1);
    assert_eq!(error.byte_offset, ORDER_FIXTURE.len());
    assert_eq!(error.unconsumed_raw, b"unknown-tail");
    assert_eq!(
        error.reason,
        FramingErrorReason::TrailingPartialRecord {
            expected_payload_bytes: TRIM_ORDER_RECORD_LEN,
            expected_delimiter_bytes: 1,
            actual_remaining_bytes: 12,
        }
    );
    assert!(records.next().is_none(), "must stop after framing loss");
}

#[test]
fn parses_a_complete_trade_source_with_exact_coordinates() {
    let records = parse_trim_trade_source(TRADE_FIXTURE)
        .collect::<Result<Vec<_>, _>>()
        .expect("complete framing");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].record_index, 0);
    assert_eq!(records[0].byte_offset, 0);
    assert_eq!(
        records[0].result.as_ref().expect("trade").raw.len(),
        TRIM_TRADE_RECORD_LEN
    );
}

#[test]
fn repeated_parsing_produces_identical_outcomes() {
    let source = [ORDER_FIXTURE, ORDER_FIXTURE].concat();

    assert_eq!(
        parse_trim_order_source(&source).collect::<Vec<_>>(),
        parse_trim_order_source(&source).collect::<Vec<_>>()
    );
}
