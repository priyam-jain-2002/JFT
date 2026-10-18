use jft_nse_cm::framing::FramingErrorReason;
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::SourceDate;
use jft_nse_cm::layouts::{TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind,
};
use jft_nse_cm::reconcile::{
    CmStreamSource, CmStreamSourceSetError, JiffyRegression, JiffyStreamRangeOverlap,
    SourceCoordinate, profile_trim_order_jiffies, profile_trim_order_stream_set,
    profile_trim_trade_jiffies, profile_trim_trade_stream_set,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const MODIFY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_modify.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

fn delivery(record_kind: CmRecordKind, members: Vec<Option<u8>>) -> CmPairedStreamSet {
    CmPairedStreamSet {
        prefix: CmFilePrefix::Cm,
        key: CmLogicalGroupKey {
            product: CmProduct::Trim,
            record_kind,
            source_date: SourceDate::new(2026, 5, 18).expect("valid source date"),
        },
        members,
    }
}

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
    assert_eq!(profile.payload_width, TRIM_ORDER_RECORD_LEN);
    assert_eq!(profile.source_bytes, source.len());
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
    assert_eq!(profile.quarantines[0].raw_jiffy, None);
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
    assert_eq!(trades.payload_width, TRIM_TRADE_RECORD_LEN);
    assert_eq!(trades.source_bytes, TRADE_FIXTURE.len());
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

#[test]
fn profiles_complete_paired_stream_set_in_member_order() {
    let expected = delivery(CmRecordKind::Orders, vec![Some(1), Some(2)]);
    let sources = [
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let profile = profile_trim_order_stream_set(&expected, &sources).expect("profile");

    assert!(profile.counts_reconcile());
    assert_eq!(
        profile
            .streams
            .iter()
            .map(|stream| stream.member)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    assert_eq!(
        profile.range_overlaps,
        vec![JiffyStreamRangeOverlap {
            left_member: Some(1),
            right_member: Some(2),
        }]
    );
}

#[test]
fn stream_set_profile_is_identical_for_reversed_source_input() {
    let expected = delivery(CmRecordKind::Orders, vec![Some(1), Some(2)]);
    let forward = [
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(2),
            source: MODIFY_FIXTURE,
        },
    ];
    let reverse = [forward[1], forward[0]];

    assert_eq!(
        profile_trim_order_stream_set(&expected, &forward),
        profile_trim_order_stream_set(&expected, &reverse)
    );
}

#[test]
fn profiles_an_unsplit_trade_delivery_without_inventing_a_stream_number() {
    let expected = delivery(CmRecordKind::Trades, vec![None]);
    let sources = [CmStreamSource {
        member: None,
        source: TRADE_FIXTURE,
    }];
    let profile = profile_trim_trade_stream_set(&expected, &sources).expect("profile");

    assert!(profile.counts_reconcile());
    assert_eq!(profile.streams.len(), 1);
    assert_eq!(profile.streams[0].member, None);
    assert_eq!(profile.streams[0].source.parsed_records, 1);
    assert!(profile.range_overlaps.is_empty());
}

#[test]
fn stream_set_profile_rejects_missing_duplicate_and_extra_members_before_parsing() {
    let expected = delivery(CmRecordKind::Orders, vec![Some(1), Some(2)]);
    let sources = [
        CmStreamSource {
            member: Some(1),
            source: b"not parsed",
        },
        CmStreamSource {
            member: Some(1),
            source: b"not parsed either",
        },
        CmStreamSource {
            member: Some(3),
            source: b"still not parsed",
        },
    ];

    assert_eq!(
        profile_trim_order_stream_set(&expected, &sources),
        Err(CmStreamSourceSetError::StreamMembersMismatch {
            expected: vec![Some(1), Some(2)],
            actual: vec![Some(1), Some(1), Some(3)],
        })
    );
}

#[test]
fn stream_set_profile_rejects_wrong_product_and_record_kind() {
    let mut full = delivery(CmRecordKind::Orders, vec![None]);
    full.key.product = CmProduct::Full;
    assert_eq!(
        profile_trim_order_stream_set(&full, &[]),
        Err(CmStreamSourceSetError::ExpectedTrim {
            actual: CmProduct::Full,
        })
    );

    let trades = delivery(CmRecordKind::Trades, vec![None]);
    assert_eq!(
        profile_trim_order_stream_set(&trades, &[]),
        Err(CmStreamSourceSetError::RecordKindMismatch {
            expected: CmRecordKind::Orders,
            actual: CmRecordKind::Trades,
        })
    );
}
