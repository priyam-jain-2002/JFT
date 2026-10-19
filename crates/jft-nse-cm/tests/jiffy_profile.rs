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
fn exhaustively_profiles_short_raw_jiffy_sequences() {
    const BASE_JIFFY: u64 = 91_200_611_817_200;
    const MAX_SEQUENCE_LEN: usize = 7;

    let mut sequence_count = 0_usize;
    let mut record_count = 0_usize;
    let mut saw_equal = false;
    let mut saw_increase = false;
    let mut saw_regression = false;
    let mut largest_runs_seen = [false; MAX_SEQUENCE_LEN + 1];

    for sequence_len in 1..=MAX_SEQUENCE_LEN {
        let configuration_count = 3_usize.pow((sequence_len - 1) as u32);
        for mut configuration in 0..configuration_count {
            let mut raw_jiffies = Vec::with_capacity(sequence_len);
            raw_jiffies.push(BASE_JIFFY);

            for _ in 1..sequence_len {
                let step = match configuration % 3 {
                    0 => 0_i64,
                    1 => 1_i64,
                    2 => -1_i64,
                    _ => unreachable!("remainder is always in 0..3"),
                };
                configuration /= 3;
                let previous = *raw_jiffies.last().expect("initial jiffy exists") as i64;
                raw_jiffies.push((previous + step) as u64);
            }

            let mut source = Vec::with_capacity(sequence_len * ENTRY_FIXTURE.len());
            for raw_jiffy in &raw_jiffies {
                let mut record = ENTRY_FIXTURE.to_vec();
                let encoded = format!("{raw_jiffy:014}");
                assert_eq!(encoded.len(), 14);
                record[22..36].copy_from_slice(encoded.as_bytes());
                source.extend_from_slice(&record);
            }

            let mut expected_equal_count = 0_usize;
            let mut expected_largest_run = 1_usize;
            let mut current_run = 1_usize;
            let mut expected_regressions = Vec::new();

            for index in 1..raw_jiffies.len() {
                let previous = raw_jiffies[index - 1];
                let current = raw_jiffies[index];
                if current == previous {
                    saw_equal = true;
                    expected_equal_count += 1;
                    current_run += 1;
                    expected_largest_run = expected_largest_run.max(current_run);
                } else {
                    current_run = 1;
                    if current > previous {
                        saw_increase = true;
                    } else {
                        saw_regression = true;
                        expected_regressions.push(JiffyRegression {
                            previous_coordinate: SourceCoordinate {
                                record_index: index - 1,
                                byte_offset: (index - 1) * ENTRY_FIXTURE.len(),
                            },
                            previous_jiffy: ExchangeJiffy::new(previous),
                            current_coordinate: SourceCoordinate {
                                record_index: index,
                                byte_offset: index * ENTRY_FIXTURE.len(),
                            },
                            current_jiffy: ExchangeJiffy::new(current),
                        });
                    }
                }
            }

            let profile = profile_trim_order_jiffies(&source);
            assert!(profile.counts_reconcile());
            assert_eq!(profile.payload_width, TRIM_ORDER_RECORD_LEN);
            assert_eq!(profile.source_bytes, source.len());
            assert_eq!(profile.framed_records, sequence_len);
            assert_eq!(profile.parsed_records, sequence_len);
            assert!(profile.quarantines.is_empty());
            assert!(profile.framing_error.is_none());
            assert_eq!(profile.sequence.observation_count, sequence_len);
            assert_eq!(
                profile.sequence.minimum,
                raw_jiffies.iter().copied().min().map(ExchangeJiffy::new)
            );
            assert_eq!(
                profile.sequence.maximum,
                raw_jiffies.iter().copied().max().map(ExchangeJiffy::new)
            );
            assert_eq!(
                profile.sequence.equal_consecutive_observations,
                expected_equal_count
            );
            assert_eq!(profile.sequence.largest_equal_run, expected_largest_run);
            assert_eq!(profile.sequence.regressions, expected_regressions);

            largest_runs_seen[expected_largest_run] = true;
            sequence_count += 1;
            record_count += sequence_len;
        }
    }

    assert_eq!(sequence_count, 1_093);
    assert_eq!(record_count, 7_108);
    assert!(saw_equal);
    assert!(saw_increase);
    assert!(saw_regression);
    assert!(largest_runs_seen[1..].iter().all(|seen| *seen));
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
