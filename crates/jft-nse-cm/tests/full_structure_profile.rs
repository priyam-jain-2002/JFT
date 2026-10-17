use std::fs;

use jft_nse_cm::framing::{FramingErrorReason, profile_fixed_width_lf};
use jft_nse_cm::layouts::{
    CURRENT_FULL_ORDER_RECORD_LEN, CURRENT_FULL_TRADE_RECORD_LEN, FieldLayout,
    FullOrderLayoutVersion, FullTradeLayoutVersion, LEGACY_FULL_ORDER_RECORD_LEN,
    LEGACY_FULL_TRADE_100_RECORD_LEN, LEGACY_FULL_TRADE_101_RECORD_LEN, SourceDate,
};
use jft_nse_cm::reconcile::{
    FieldByteProfileError, profile_fixed_width_field_bytes, profile_full_order_field_bytes,
    profile_full_order_framing, profile_full_trade_field_bytes, profile_full_trade_framing,
};

const CURRENT_ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/full_order_entry.dat");
const CURRENT_TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/full_trade.dat");

fn date(year: u16, month: u8, day: u8) -> SourceDate {
    SourceDate::new(year, month, day).expect("valid source date")
}

fn zero_record(width: usize) -> Vec<u8> {
    let mut source = vec![b'0'; width];
    source.push(b'\n');
    source
}

#[test]
fn profiles_current_full_fixtures_without_semantic_claims() {
    let order = profile_full_order_framing(CURRENT_ORDER_FIXTURE, date(2026, 5, 18));
    let trade = profile_full_trade_framing(CURRENT_TRADE_FIXTURE, date(2026, 5, 18));

    assert_eq!(order.payload_width, CURRENT_FULL_ORDER_RECORD_LEN);
    assert_eq!(trade.payload_width, CURRENT_FULL_TRADE_RECORD_LEN);
    assert_eq!(order.framed_records, 1);
    assert_eq!(trade.framed_records, 1);
    assert!(order.is_complete());
    assert!(trade.is_complete());
}

#[test]
fn dispatches_every_historical_full_width_for_structural_profiling() {
    let order = zero_record(LEGACY_FULL_ORDER_RECORD_LEN);
    let trade_100 = zero_record(LEGACY_FULL_TRADE_100_RECORD_LEN);
    let trade_101 = zero_record(LEGACY_FULL_TRADE_101_RECORD_LEN);
    let trade_103 = zero_record(CURRENT_FULL_TRADE_RECORD_LEN);

    assert!(profile_full_order_framing(&order, date(2024, 6, 30)).is_complete());
    assert!(profile_full_trade_framing(&trade_100, date(2024, 6, 30)).is_complete());
    assert!(profile_full_trade_framing(&trade_101, date(2024, 7, 1)).is_complete());
    assert!(profile_full_trade_framing(&trade_103, date(2025, 3, 28)).is_complete());
    assert!(profile_full_trade_framing(&trade_101, date(2025, 3, 29)).is_complete());
    assert!(profile_full_trade_framing(&trade_103, date(2025, 6, 16)).is_complete());
}

#[test]
fn structural_profile_preserves_and_reconciles_the_first_unconsumed_tail() {
    let mut source = zero_record(LEGACY_FULL_TRADE_101_RECORD_LEN);
    source.extend_from_slice(b"partial");
    let profile = profile_full_trade_framing(&source, date(2024, 7, 1));
    let error = profile.framing_error.expect("partial tail");

    assert_eq!(profile.framed_records, 1);
    assert_eq!(error.record_index, 1);
    assert_eq!(error.byte_offset, LEGACY_FULL_TRADE_101_RECORD_LEN + 1);
    assert_eq!(error.unconsumed_raw, b"partial");
    assert_eq!(
        error.reason,
        FramingErrorReason::TrailingPartialRecord {
            expected_payload_bytes: LEGACY_FULL_TRADE_101_RECORD_LEN,
            expected_delimiter_bytes: 1,
            actual_remaining_bytes: 7,
        }
    );
    assert!(profile.bytes_reconcile());
    assert!(!profile.is_complete());
}

#[test]
fn structural_profile_is_repeatable_and_counts_every_source_byte() {
    let source = [CURRENT_ORDER_FIXTURE, CURRENT_ORDER_FIXTURE].concat();
    let first = profile_full_order_framing(&source, date(2026, 5, 18));
    let second = profile_full_order_framing(&source, date(2026, 5, 18));

    assert_eq!(first, second);
    assert_eq!(first.framed_records, 2);
    assert_eq!(first.framed_bytes(), Some(source.len()));
    assert!(first.bytes_reconcile());
    assert!(first.is_complete());
}

#[test]
fn field_byte_profile_counts_every_current_fixture_position() {
    let profile =
        profile_full_order_field_bytes(CURRENT_ORDER_FIXTURE, FullOrderLayoutVersion::Bytes91)
            .expect("field profile");

    assert!(profile.framing.is_complete());
    assert_eq!(profile.fields.len(), 17);
    for field in &profile.fields {
        assert_eq!(
            field.total_observations(),
            Some(field.field.width as u64),
            "{}",
            field.field.name
        );
    }
    assert!(
        profile
            .field("order_number")
            .expect("order number")
            .observed_bytes_are_ascii_digits()
    );
    let symbol = profile.field("symbol").expect("symbol");
    assert_eq!(symbol.count(b' '), 2);
    assert_eq!(symbol.count(b'R'), 1);
    assert_eq!(symbol.total_observations(), Some(10));
}

#[test]
fn field_byte_profile_rejects_an_out_of_bounds_layout_before_framing() {
    let invalid = FieldLayout {
        name: "invalid",
        offset: CURRENT_FULL_ORDER_RECORD_LEN,
        width: 1,
    };

    assert_eq!(
        profile_fixed_width_field_bytes::<CURRENT_FULL_ORDER_RECORD_LEN>(
            CURRENT_ORDER_FIXTURE,
            &[invalid]
        ),
        Err(FieldByteProfileError::FieldOutOfBounds {
            field: invalid,
            payload_width: CURRENT_FULL_ORDER_RECORD_LEN,
        })
    );
}

#[test]
fn field_byte_profile_retains_framing_residue_and_only_counts_complete_records() {
    let mut source = CURRENT_TRADE_FIXTURE.to_vec();
    source.extend_from_slice(b"partial");
    let profile = profile_full_trade_field_bytes(&source, FullTradeLayoutVersion::Bytes103)
        .expect("field profile");

    assert_eq!(profile.framing.framed_records, 1);
    assert!(profile.framing.bytes_reconcile());
    assert!(!profile.framing.is_complete());
    assert_eq!(
        profile
            .field("trade_number")
            .expect("trade number")
            .total_observations(),
        Some(17)
    );
}

#[test]
#[ignore = "requires locally supplied public NSE sample files"]
fn profiles_the_locally_supplied_public_legacy_sample() {
    let order_path = std::env::var_os("JFT_PUBLIC_CM_ORDER_SAMPLE")
        .expect("JFT_PUBLIC_CM_ORDER_SAMPLE must point to the extracted Order sample");
    let trade_path = std::env::var_os("JFT_PUBLIC_CM_TRADE_SAMPLE")
        .expect("JFT_PUBLIC_CM_TRADE_SAMPLE must point to the extracted Trade sample");
    let orders = fs::read(order_path).expect("read public Order sample");
    let trades = fs::read(trade_path).expect("read public Trade sample");
    let order_profile = profile_fixed_width_lf::<LEGACY_FULL_ORDER_RECORD_LEN>(&orders);
    let trade_profile = profile_fixed_width_lf::<LEGACY_FULL_TRADE_100_RECORD_LEN>(&trades);

    assert_eq!(order_profile.framed_records, 300_000);
    assert_eq!(trade_profile.framed_records, 300_000);
    assert!(order_profile.is_complete());
    assert!(trade_profile.is_complete());

    let order_fields = profile_full_order_field_bytes(&orders, FullOrderLayoutVersion::Bytes87)
        .expect("profile legacy Order fields");
    let trade_fields = profile_full_trade_field_bytes(&trades, FullTradeLayoutVersion::Bytes100)
        .expect("profile legacy Trade fields");
    for name in [
        "order_number",
        "exchange_jiffy",
        "disclosed_quantity",
        "original_quantity",
        "limit_price_paise",
        "trigger_price_paise",
    ] {
        assert!(
            order_fields
                .field(name)
                .expect("documented Order field")
                .observed_bytes_are_ascii_digits(),
            "Order field {name} contained a non-digit byte"
        );
    }
    for name in [
        "trade_number",
        "exchange_jiffy",
        "price_paise",
        "quantity",
        "buy_order_number",
        "sell_order_number",
    ] {
        assert!(
            trade_fields
                .field(name)
                .expect("documented Trade field")
                .observed_bytes_are_ascii_digits(),
            "Trade field {name} contained a non-digit byte"
        );
    }
    assert!(order_fields.field("symbol").expect("symbol").count(b'b') > 0);
}
