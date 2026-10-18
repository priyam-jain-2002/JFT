use jft_nse_cm::events::OrderActivity;
use jft_nse_cm::framing::FramingErrorReason;
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::{SourceDate, TRIM_ORDER_RECORD_LEN};
use jft_nse_cm::lifecycle::LifecycleAnomaly;
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind,
};
use jft_nse_cm::reconcile::{
    CmStreamSource, CmStreamSourceSetError, OrderReplaySummary, OrderReplaySummaryError,
    SessionObservationCountError, SessionObservationCounts, SourceCoordinate, TrimOrderReplay,
    replay_trim_order_source, replay_trim_order_source_with_observer, replay_trim_order_stream_set,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const MODIFY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_modify.dat");
const CANCEL_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_cancel.dat");

fn assert_counts_reconcile(replay: &TrimOrderReplay<'_>) {
    assert!(
        replay.counts_reconcile(),
        "every framed record is accounted"
    );
}

fn split_order_delivery() -> CmPairedStreamSet {
    CmPairedStreamSet {
        prefix: CmFilePrefix::Cm,
        key: CmLogicalGroupKey {
            product: CmProduct::Trim,
            record_kind: CmRecordKind::Orders,
            source_date: SourceDate::new(2026, 5, 18).expect("valid source date"),
        },
        members: vec![Some(1), Some(2)],
    }
}

#[test]
fn replays_a_complete_entry_modify_cancel_source_with_exact_counts() {
    let source = [ENTRY_FIXTURE, MODIFY_FIXTURE, CANCEL_FIXTURE].concat();
    let replay = replay_trim_order_source(&source);
    assert_counts_reconcile(&replay);

    assert_eq!(replay.framed_records, 3);
    assert_eq!(replay.parsed_records, 3);
    assert_eq!(replay.applied_events, 3);
    assert!(replay.quarantines.is_empty());
    assert!(replay.lifecycle_anomalies.is_empty());
    assert!(replay.framing_error.is_none());
    assert!(replay.lifecycle.is_empty());
    assert!(replay.active_provenance.is_empty());
    assert!(replay.book_projection().expect("book").is_empty());
}

#[test]
fn parser_quarantine_retains_raw_bytes_and_replay_continues() {
    let mut invalid = ENTRY_FIXTURE.to_vec();
    invalid[37] = b'2';
    let source = [invalid.as_slice(), ENTRY_FIXTURE].concat();
    let replay = replay_trim_order_source(&source);
    assert_counts_reconcile(&replay);

    assert_eq!(replay.framed_records, 2);
    assert_eq!(replay.parsed_records, 1);
    assert_eq!(replay.applied_events, 1);
    assert_eq!(replay.quarantines.len(), 1);
    assert_eq!(
        replay.quarantines[0].coordinate,
        SourceCoordinate {
            record_index: 0,
            byte_offset: 0,
        }
    );
    assert_eq!(
        replay.quarantines[0].quarantined.raw,
        invalid[..TRIM_ORDER_RECORD_LEN]
    );
    assert_eq!(replay.lifecycle.len(), 1);
}

#[test]
fn duplicate_entry_anomaly_retains_event_raw_bytes_and_coordinates() {
    let source = [ENTRY_FIXTURE, ENTRY_FIXTURE].concat();
    let replay = replay_trim_order_source(&source);
    assert_counts_reconcile(&replay);

    assert_eq!(replay.framed_records, 2);
    assert_eq!(replay.parsed_records, 2);
    assert_eq!(replay.applied_events, 1);
    assert_eq!(replay.lifecycle_anomalies.len(), 1);
    let anomaly = &replay.lifecycle_anomalies[0];
    assert_eq!(
        anomaly.coordinate,
        SourceCoordinate {
            record_index: 1,
            byte_offset: TRIM_ORDER_RECORD_LEN + 1,
        }
    );
    assert_eq!(anomaly.raw, ENTRY_FIXTURE[..TRIM_ORDER_RECORD_LEN]);
    assert!(matches!(
        anomaly.anomaly,
        LifecycleAnomaly::DuplicateEntry { .. }
    ));
    assert_eq!(replay.lifecycle.len(), 1);
}

#[test]
fn unknown_modify_is_reported_without_creating_order_or_provenance() {
    let replay = replay_trim_order_source(MODIFY_FIXTURE);
    assert_counts_reconcile(&replay);

    assert_eq!(replay.applied_events, 0);
    assert_eq!(replay.lifecycle_anomalies.len(), 1);
    assert!(matches!(
        replay.lifecycle_anomalies[0].anomaly,
        LifecycleAnomaly::UnknownModify { .. }
    ));
    assert!(replay.lifecycle.is_empty());
    assert!(replay.active_provenance.is_empty());
}

#[test]
fn regressing_modify_is_reported_and_preserves_entry_state() {
    let mut regressing = MODIFY_FIXTURE.to_vec();
    regressing[22..36].copy_from_slice(b"91200611817220");
    let source = [ENTRY_FIXTURE, regressing.as_slice()].concat();
    let replay = replay_trim_order_source(&source);
    assert_counts_reconcile(&replay);

    assert_eq!(replay.applied_events, 1);
    assert_eq!(replay.lifecycle_anomalies.len(), 1);
    assert!(matches!(
        replay.lifecycle_anomalies[0].anomaly,
        LifecycleAnomaly::RegressingJiffy {
            activity: OrderActivity::Modify,
            previous_jiffy,
            incoming_jiffy,
            ..
        } if previous_jiffy == ExchangeJiffy::new(91_200_611_817_221)
            && incoming_jiffy == ExchangeJiffy::new(91_200_611_817_220)
    ));
    let (_, active) = replay
        .lifecycle
        .active_orders()
        .next()
        .expect("entry remains active");
    assert_eq!(active.quantity, 100);
}

#[test]
fn framing_failure_stops_replay_and_borrows_the_exact_unconsumed_tail() {
    let mut source = ENTRY_FIXTURE.to_vec();
    source.extend_from_slice(b"partial-tail");
    let replay = replay_trim_order_source(&source);
    assert_counts_reconcile(&replay);
    let error = replay.framing_error.expect("framing error");

    assert_eq!(replay.applied_events, 1);
    assert_eq!(error.record_index, 1);
    assert_eq!(error.byte_offset, ENTRY_FIXTURE.len());
    assert_eq!(error.unconsumed_raw, b"partial-tail");
    assert_eq!(
        error.unconsumed_raw.as_ptr(),
        source[ENTRY_FIXTURE.len()..].as_ptr()
    );
    assert_eq!(
        error.reason,
        FramingErrorReason::TrailingPartialRecord {
            expected_payload_bytes: TRIM_ORDER_RECORD_LEN,
            expected_delimiter_bytes: 1,
            actual_remaining_bytes: 12,
        }
    );
}

#[test]
fn repeated_replay_has_identical_state_evidence_and_active_provenance() {
    let source = [ENTRY_FIXTURE, MODIFY_FIXTURE].concat();
    let first = replay_trim_order_source(&source);
    let second = replay_trim_order_source(&source);
    assert_counts_reconcile(&first);
    assert_counts_reconcile(&second);

    assert_eq!(first, second);
    let (key, active) = first
        .lifecycle
        .active_orders()
        .next()
        .expect("modified order remains active");
    assert_eq!(active.quantity, 75);
    assert_eq!(active.price_paise, 250_100);
    assert_eq!(
        first.active_provenance.get(key),
        Some(&jft_nse_cm::reconcile::ActiveOrderProvenance {
            entry: SourceCoordinate {
                record_index: 0,
                byte_offset: 0,
            },
            last_mutation: SourceCoordinate {
                record_index: 1,
                byte_offset: TRIM_ORDER_RECORD_LEN + 1,
            },
        })
    );
    let projection = first.book_projection().expect("book projection");
    assert_eq!(projection.len(), 1);
}

#[test]
fn transition_observer_receives_every_success_in_exact_source_order() {
    let source = [ENTRY_FIXTURE, MODIFY_FIXTURE, CANCEL_FIXTURE].concat();
    let mut transitions = Vec::new();
    let replay = replay_trim_order_source_with_observer(&source, |transition| {
        transitions.push(transition);
    });

    assert_eq!(transitions.len(), 3);
    for (index, transition) in transitions.iter().enumerate() {
        assert_eq!(transition.coordinate.record_index, index);
        assert_eq!(
            transition.coordinate.byte_offset,
            index * (TRIM_ORDER_RECORD_LEN + 1)
        );
        assert_eq!(transition.raw.len(), TRIM_ORDER_RECORD_LEN);
    }
    assert!(matches!(
        transitions[0].transition,
        jft_nse_cm::lifecycle::LifecycleTransition::Entered { .. }
    ));
    assert!(matches!(
        transitions[1].transition,
        jft_nse_cm::lifecycle::LifecycleTransition::Modified { .. }
    ));
    assert!(matches!(
        transitions[2].transition,
        jft_nse_cm::lifecycle::LifecycleTransition::Cancelled { .. }
    ));
    assert_eq!(replay.applied_events, transitions.len());
}

#[test]
fn transition_observer_excludes_parser_and_lifecycle_failures() {
    let mut invalid = ENTRY_FIXTURE.to_vec();
    invalid[37] = b'2';
    let source = [
        invalid.as_slice(),
        ENTRY_FIXTURE,
        ENTRY_FIXTURE,
        MODIFY_FIXTURE,
    ]
    .concat();
    let mut transitions = Vec::new();
    let replay = replay_trim_order_source_with_observer(&source, |transition| {
        transitions.push(transition);
    });

    assert_eq!(replay.quarantines.len(), 1);
    assert_eq!(replay.lifecycle_anomalies.len(), 1);
    assert_eq!(replay.applied_events, 2);
    assert_eq!(transitions.len(), 2);
    assert_eq!(transitions[0].coordinate.record_index, 1);
    assert_eq!(transitions[1].coordinate.record_index, 3);
}

#[test]
fn observed_raw_bytes_and_events_are_the_exact_parsed_records() {
    let mut observed = Vec::new();
    let replay = replay_trim_order_source_with_observer(ENTRY_FIXTURE, |transition| {
        observed.push(transition);
    });

    assert_eq!(replay.applied_events, 1);
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].raw, ENTRY_FIXTURE[..TRIM_ORDER_RECORD_LEN]);
    assert_eq!(observed[0].event.activity, OrderActivity::Entry);
    assert_eq!(observed[0].event.order_number, 1);
}

#[test]
fn observing_transitions_does_not_change_the_replay_report() {
    let source = [ENTRY_FIXTURE, MODIFY_FIXTURE].concat();
    let default = replay_trim_order_source(&source);
    let observed = replay_trim_order_source_with_observer(&source, |_| {});

    assert_eq!(default, observed);
}

#[test]
fn paired_stream_replay_is_deterministic_but_does_not_merge_streams() {
    let delivery = split_order_delivery();
    let forward = [
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
    ];
    let reverse = [forward[1], forward[0]];
    let replay = replay_trim_order_stream_set(&delivery, &forward).expect("stream replay");

    assert_eq!(
        replay,
        replay_trim_order_stream_set(&delivery, &reverse).expect("reversed stream replay")
    );
    assert!(replay.counts_reconcile());
    assert_eq!(replay.streams.len(), 2);
    assert_eq!(replay.streams[0].member, Some(1));
    assert_eq!(replay.streams[1].member, Some(2));
    for stream in &replay.streams {
        assert_eq!(stream.replay.lifecycle.len(), 1);
        assert!(stream.replay.lifecycle_anomalies.is_empty());
    }
}

#[test]
fn paired_stream_replay_retains_stream_context_for_quarantine() {
    let delivery = split_order_delivery();
    let mut invalid = ENTRY_FIXTURE.to_vec();
    invalid[37] = b'2';
    let sources = [
        CmStreamSource {
            member: Some(2),
            source: &invalid,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let replay = replay_trim_order_stream_set(&delivery, &sources).expect("stream replay");

    assert!(replay.counts_reconcile());
    assert!(replay.streams[0].replay.quarantines.is_empty());
    assert_eq!(replay.streams[1].member, Some(2));
    assert_eq!(replay.streams[1].replay.quarantines.len(), 1);
    assert_eq!(
        replay.streams[1].replay.quarantines[0].quarantined.raw,
        invalid[..TRIM_ORDER_RECORD_LEN]
    );
}

#[test]
fn paired_stream_replay_rejects_member_mismatch_before_parsing() {
    let delivery = split_order_delivery();
    let sources = [CmStreamSource {
        member: Some(1),
        source: b"not parsed",
    }];

    assert_eq!(
        replay_trim_order_stream_set(&delivery, &sources),
        Err(CmStreamSourceSetError::StreamMembersMismatch {
            expected: vec![Some(1), Some(2)],
            actual: vec![Some(1)],
        })
    );
}

#[test]
fn paired_stream_summary_reconciles_without_merging_active_orders() {
    let member_one = [ENTRY_FIXTURE, MODIFY_FIXTURE].concat();
    let sources = [
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one,
        },
    ];
    let replay = replay_trim_order_stream_set(&split_order_delivery(), &sources)
        .expect("independent stream replay");
    let summary = replay.combined_summary().expect("checked count summary");

    assert!(summary.counts_reconcile());
    assert_eq!(summary.framed_records, 3);
    assert_eq!(summary.parsed_records, 3);
    assert_eq!(summary.applied_events, 3);
    assert_eq!(summary.quarantine_count, 0);
    assert_eq!(summary.lifecycle_anomaly_count, 0);
    assert_eq!(summary.active_order_observations, 2);
}

#[test]
fn order_replay_summary_fails_closed_on_every_count_overflow() {
    let one = OrderReplaySummary {
        framed_records: 1,
        parsed_records: 1,
        applied_events: 1,
        quarantine_count: 1,
        lifecycle_anomaly_count: 1,
        active_order_observations: 1,
        session_counts: Default::default(),
    };

    assert_eq!(
        OrderReplaySummary {
            framed_records: usize::MAX,
            ..Default::default()
        }
        .checked_merge(one),
        Err(OrderReplaySummaryError::FramedRecordOverflow)
    );
    assert_eq!(
        OrderReplaySummary {
            parsed_records: usize::MAX,
            ..Default::default()
        }
        .checked_merge(one),
        Err(OrderReplaySummaryError::ParsedRecordOverflow)
    );
    assert_eq!(
        OrderReplaySummary {
            applied_events: usize::MAX,
            ..Default::default()
        }
        .checked_merge(one),
        Err(OrderReplaySummaryError::AppliedEventOverflow)
    );
    assert_eq!(
        OrderReplaySummary {
            quarantine_count: usize::MAX,
            ..Default::default()
        }
        .checked_merge(one),
        Err(OrderReplaySummaryError::QuarantineCountOverflow)
    );
    assert_eq!(
        OrderReplaySummary {
            lifecycle_anomaly_count: usize::MAX,
            ..Default::default()
        }
        .checked_merge(one),
        Err(OrderReplaySummaryError::LifecycleAnomalyCountOverflow)
    );
    assert_eq!(
        OrderReplaySummary {
            active_order_observations: usize::MAX,
            ..Default::default()
        }
        .checked_merge(one),
        Err(OrderReplaySummaryError::ActiveOrderObservationOverflow)
    );
    assert_eq!(
        SessionObservationCounts {
            pre_open: usize::MAX,
            ..Default::default()
        }
        .checked_merge(SessionObservationCounts {
            pre_open: 1,
            ..Default::default()
        }),
        Err(SessionObservationCountError::PreOpenOverflow)
    );
    assert_eq!(
        SessionObservationCounts {
            regular_market: usize::MAX,
            ..Default::default()
        }
        .checked_merge(SessionObservationCounts {
            regular_market: 1,
            ..Default::default()
        }),
        Err(SessionObservationCountError::RegularMarketOverflow)
    );
    assert_eq!(
        SessionObservationCounts {
            closing_auction: usize::MAX,
            ..Default::default()
        }
        .checked_merge(SessionObservationCounts {
            closing_auction: 1,
            ..Default::default()
        }),
        Err(SessionObservationCountError::ClosingAuctionOverflow)
    );
}
