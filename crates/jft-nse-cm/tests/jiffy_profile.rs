use jft_nse_cm::framing::FramingErrorReason;
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::{TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::reconcile::{
    JiffyRegression, SourceCoordinate, profile_trim_order_jiffies, profile_trim_trade_jiffies,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const MODIFY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_modify.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

#[test]
fn profiles_empty_and_strictly_increasing_order_sources() {
    let empty = profile_trim_order_jiffies(&[]);
    assert!(empty.counts_reconcile());
    assert_eq!(empty.sequence.observation_count, 0);
    assert_eq!(empty.sequence.minimum, None);
    assert_eq!(empty.sequence.maximum, None);
    assert_eq!(empty.sequence.largest_equal_run, 0);

    let source = [ENTRY_FIXTURE, MODIFY_FIXTURE].concat();
    let increasing = profile_trim_order_jiffies(&source);
    assert!(increasing.counts_reconcile());
    assert_eq!(increasing.sequence.observation_count, 2);
    assert_eq!(
        increasing.sequence.minimum,
        Some(ExchangeJiffy::new(91_200_611_817_221))
    );
    assert_eq!(
        increasing.sequence.maximum,
        Some(ExchangeJiffy::new(91_200_611_817_223))
    );
    assert_eq!(increasing.sequence.equal_consecutive_observations, 0);
    assert_eq!(increasing.sequence.largest_equal_run, 1);
    assert!(increasing.sequence.regressions.is_empty());
}

#[test]
fn counts_adjacent_ties_and_the_largest_equal_jiffy_run() {
    let source = [ENTRY_FIXTURE, ENTRY_FIXTURE, ENTRY_FIXTURE].concat();
    let profile = profile_trim_order_jiffies(&source);

    assert!(profile.counts_reconcile());
    assert_eq!(profile.sequence.observation_count, 3);
    assert_eq!(profile.sequence.equal_consecutive_observations, 2);
    assert_eq!(profile.sequence.largest_equal_run, 3);
    assert!(profile.sequence.regressions.is_empty());
}

#[test]
fn records_every_regression_with_exact_adjacent_coordinates() {
    let mut lower = ENTRY_FIXTURE.to_vec();
    lower[22..36].copy_from_slice(b"91200611817220");
    let source = [ENTRY_FIXTURE, lower.as_slice(), MODIFY_FIXTURE].concat();
    let profile = profile_trim_order_jiffies(&source);

    assert!(profile.counts_reconcile());
    assert_eq!(
        profile.sequence.regressions,
        vec![JiffyRegression {
            previous_coordinate: SourceCoordinate {
                record_index: 0,
                byte_offset: 0,
            },
            previous_jiffy: ExchangeJiffy::new(91_200_611_817_221),
            current_coordinate: SourceCoordinate {
                record_index: 1,
                byte_offset: TRIM_ORDER_RECORD_LEN + 1,
            },
            current_jiffy: ExchangeJiffy::new(91_200_611_817_220),
        }]
    );
}

#[test]
fn quarantined_records_are_accounted_but_excluded_from_jiffy_observations() {
    let mut invalid = ENTRY_FIXTURE.to_vec();
    invalid[22] = b'X';
    let source = [ENTRY_FIXTURE, invalid.as_slice(), MODIFY_FIXTURE].concat();
    let profile = profile_trim_order_jiffies(&source);

    assert!(profile.counts_reconcile());
    assert_eq!(profile.framed_records, 3);
    assert_eq!(profile.parsed_records, 2);
    assert_eq!(profile.quarantines.len(), 1);
    assert_eq!(profile.sequence.observation_count, 2);
    assert_eq!(profile.quarantines[0].coordinate.record_index, 1);
}

#[test]
fn framing_loss_preserves_the_unconsumed_tail_and_stops_profiling() {
    let mut source = ENTRY_FIXTURE.to_vec();
    source.extend_from_slice(b"tail");
    let profile = profile_trim_order_jiffies(&source);
    let error = profile.framing_error.expect("framing error");

    assert!(profile.counts_reconcile());
    assert_eq!(profile.sequence.observation_count, 1);
    assert_eq!(error.unconsumed_raw, b"tail");
    assert_eq!(
        error.reason,
        FramingErrorReason::TrailingPartialRecord {
            expected_payload_bytes: TRIM_ORDER_RECORD_LEN,
            expected_delimiter_bytes: 1,
            actual_remaining_bytes: 4,
        }
    );
}

#[test]
fn order_and_trade_ranges_report_overlap_without_selecting_merge_order() {
    let order_source = [ENTRY_FIXTURE, MODIFY_FIXTURE].concat();
    let orders = profile_trim_order_jiffies(&order_source);
    let trades = profile_trim_trade_jiffies(TRADE_FIXTURE);

    assert!(orders.counts_reconcile());
    assert!(trades.counts_reconcile());
    assert_eq!(trades.framed_records, 1);
    assert_eq!(trades.parsed_records, 1);
    assert!(orders.sequence.range_overlaps(&trades.sequence));
    assert_eq!(TRIM_TRADE_RECORD_LEN + 1, TRADE_FIXTURE.len());
}

#[test]
fn repeated_profiles_are_identical() {
    let source = [ENTRY_FIXTURE, MODIFY_FIXTURE].concat();

    assert_eq!(
        profile_trim_order_jiffies(&source),
        profile_trim_order_jiffies(&source)
    );
}
