use jft_nse_cm::events::{Session, Side};
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::{SourceDate, TRIM_ORDER_RECORD_LEN, TRIM_TRADE_RECORD_LEN};
use jft_nse_cm::manifest::{
    CmFilePrefix, CmLogicalGroupKey, CmPairedStreamSet, CmProduct, CmRecordKind,
};
use jft_nse_cm::parser::parse_trim_trade;
use jft_nse_cm::reconcile::{
    CmStreamSource, OrderEntryEvidence, OrderSourceReference, SourceCoordinate,
    TradeCandidateStreamSetError, TradeOrderCandidateProfile, TradeValueSummary,
    TradeValueSummaryError, collect_trim_trade_stream_candidates,
    profile_trim_trade_candidate_stream_set, profile_trim_trade_candidate_stream_set_with_observer,
    profile_trim_trade_stream_candidates, reconcile_trim_trade_references,
    replay_trim_order_source, replay_trim_order_stream_set,
};

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const CANCEL_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_cancel.dat");
const TRADE_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_trade.dat");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateSpec {
    order_number: u64,
    side: Side,
    jiffy: u64,
    record_index: usize,
}

fn order_entry_record(order_number: u64, side: Side, jiffy: u64) -> Vec<u8> {
    let mut record = ENTRY_FIXTURE.to_vec();
    record[6..22].copy_from_slice(format!("{order_number:016}").as_bytes());
    record[22..36].copy_from_slice(format!("{jiffy:014}").as_bytes());
    record[36] = match side {
        Side::Buy => b'B',
        Side::Sell => b'S',
    };
    record
}

fn candidate_source(state: u32) -> (Vec<u8>, Vec<CandidateSpec>) {
    match state {
        0 => (Vec::new(), Vec::new()),
        1 => (
            order_entry_record(1, Side::Buy, 91_200_611_817_221),
            vec![CandidateSpec {
                order_number: 1,
                side: Side::Buy,
                jiffy: 91_200_611_817_221,
                record_index: 0,
            }],
        ),
        2 => (
            order_entry_record(1, Side::Sell, 91_200_611_817_222),
            vec![CandidateSpec {
                order_number: 1,
                side: Side::Sell,
                jiffy: 91_200_611_817_222,
                record_index: 0,
            }],
        ),
        3 => (
            order_entry_record(2, Side::Sell, 91_200_611_817_223),
            vec![CandidateSpec {
                order_number: 2,
                side: Side::Sell,
                jiffy: 91_200_611_817_223,
                record_index: 0,
            }],
        ),
        4 => (
            order_entry_record(2, Side::Buy, 91_200_611_817_221),
            vec![CandidateSpec {
                order_number: 2,
                side: Side::Buy,
                jiffy: 91_200_611_817_221,
                record_index: 0,
            }],
        ),
        5 => (
            [
                order_entry_record(1, Side::Buy, 91_200_611_817_222),
                order_entry_record(2, Side::Sell, 91_200_611_817_222),
            ]
            .concat(),
            vec![
                CandidateSpec {
                    order_number: 1,
                    side: Side::Buy,
                    jiffy: 91_200_611_817_222,
                    record_index: 0,
                },
                CandidateSpec {
                    order_number: 2,
                    side: Side::Sell,
                    jiffy: 91_200_611_817_222,
                    record_index: 1,
                },
            ],
        ),
        _ => unreachable!("six candidate-source states"),
    }
}

fn sell_entry() -> Vec<u8> {
    let mut sell = ENTRY_FIXTURE.to_vec();
    sell[6..22].copy_from_slice(b"0000000000000002");
    sell[22..36].copy_from_slice(b"91200611817222");
    sell[36] = b'S';
    sell
}

fn trade() -> jft_nse_cm::events::TradeEvent {
    parse_trim_trade(&TRADE_FIXTURE[..TRIM_TRADE_RECORD_LEN])
        .expect("synthetic trade")
        .event
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

fn split_trade_delivery(source_date: SourceDate) -> CmPairedStreamSet {
    CmPairedStreamSet {
        prefix: CmFilePrefix::Cm,
        key: CmLogicalGroupKey {
            product: CmProduct::Trim,
            record_kind: CmRecordKind::Trades,
            source_date,
        },
        members: vec![Some(1), Some(2)],
    }
}

#[test]
fn matches_both_trade_sides_to_exact_entry_coordinates() {
    let sell = sell_entry();
    let source = [ENTRY_FIXTURE, sell.as_slice()].concat();
    let orders = replay_trim_order_source(&source);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(matches!(
        references.buy,
        OrderSourceReference::EntryMatch {
            entry: OrderEntryEvidence {
                side: Side::Buy,
                coordinate: SourceCoordinate {
                    record_index: 0,
                    byte_offset: 0,
                },
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::EntryMatch {
            entry: OrderEntryEvidence {
                side: Side::Sell,
                coordinate: SourceCoordinate {
                    record_index: 1,
                    byte_offset,
                },
                ..
            },
            ..
        } if byte_offset == TRIM_ORDER_RECORD_LEN + 1
    ));
}

#[test]
fn missing_sell_reference_does_not_hide_the_matched_buy_reference() {
    let orders = replay_trim_order_source(ENTRY_FIXTURE);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(matches!(
        references.buy,
        OrderSourceReference::EntryMatch { .. }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::Missing {
            expected_side: Side::Sell,
            ..
        }
    ));
}

#[test]
fn entry_side_mismatch_is_distinct_from_a_missing_reference() {
    let mut wrong_side = sell_entry();
    wrong_side[36] = b'B';
    let source = [ENTRY_FIXTURE, wrong_side.as_slice()].concat();
    let orders = replay_trim_order_source(&source);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(matches!(
        references.sell,
        OrderSourceReference::EntrySideMismatch {
            expected_side: Side::Sell,
            entry: OrderEntryEvidence {
                side: Side::Buy,
                ..
            },
            ..
        }
    ));
}

#[test]
fn session_and_raw_instrument_identity_are_required_for_a_match() {
    let sell = sell_entry();
    let source = [ENTRY_FIXTURE, sell.as_slice()].concat();
    let orders = replay_trim_order_source(&source);

    let mut wrong_session = trade();
    wrong_session.session = Session::PreOpen;
    let references = reconcile_trim_trade_references(&orders, &wrong_session);
    assert!(matches!(
        references.buy,
        OrderSourceReference::Missing { .. }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::Missing { .. }
    ));

    let mut wrong_symbol = trade();
    wrong_symbol.symbol_raw = *b"       TCS";
    wrong_symbol.symbol = "TCS".to_owned();
    let references = reconcile_trim_trade_references(&orders, &wrong_symbol);
    assert!(matches!(
        references.buy,
        OrderSourceReference::Missing { .. }
    ));
    assert!(matches!(
        references.sell,
        OrderSourceReference::Missing { .. }
    ));
}

#[test]
fn cancelled_order_remains_an_entry_source_match_without_temporal_claim() {
    let source = [ENTRY_FIXTURE, CANCEL_FIXTURE].concat();
    let orders = replay_trim_order_source(&source);
    let references = reconcile_trim_trade_references(&orders, &trade());

    assert!(orders.lifecycle.is_empty());
    assert!(matches!(
        references.buy,
        OrderSourceReference::EntryMatch { .. }
    ));
}

#[test]
fn repeated_reference_reconciliation_is_identical() {
    let sell = sell_entry();
    let source = [ENTRY_FIXTURE, sell.as_slice()].concat();
    let orders = replay_trim_order_source(&source);
    let event = trade();

    assert_eq!(
        reconcile_trim_trade_references(&orders, &event),
        reconcile_trim_trade_references(&orders, &event)
    );
}

#[test]
fn collects_exact_buy_and_sell_candidates_from_separate_streams() {
    let sell = sell_entry();
    let sources = [
        CmStreamSource {
            member: Some(2),
            source: &sell,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &sources)
        .expect("independent stream replay");
    let candidates = collect_trim_trade_stream_candidates(&orders, &trade());

    assert_eq!(candidates.buy.candidates.len(), 1);
    assert_eq!(candidates.buy.candidates[0].member, Some(1));
    assert_eq!(candidates.buy.candidates[0].entry.side, Side::Buy);
    assert_eq!(candidates.sell.candidates.len(), 1);
    assert_eq!(candidates.sell.candidates[0].member, Some(2));
    assert_eq!(candidates.sell.candidates[0].entry.side, Side::Sell);
}

#[test]
fn preserves_multiple_cross_stream_candidates_in_member_order() {
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
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &sources)
        .expect("independent stream replay");
    let candidates = collect_trim_trade_stream_candidates(&orders, &trade());

    assert_eq!(
        candidates
            .buy
            .candidates
            .iter()
            .map(|candidate| candidate.member)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    assert!(candidates.sell.candidates.is_empty());
}

#[test]
fn retains_side_mismatch_as_candidate_evidence_instead_of_filtering_it() {
    let mut wrong_side = sell_entry();
    wrong_side[36] = b'B';
    let sources = [
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(2),
            source: &wrong_side,
        },
    ];
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &sources)
        .expect("independent stream replay");
    let candidates = collect_trim_trade_stream_candidates(&orders, &trade());

    assert_eq!(candidates.sell.expected_side, Side::Sell);
    assert_eq!(candidates.sell.candidates.len(), 1);
    assert_eq!(candidates.sell.candidates[0].member, Some(2));
    assert_eq!(candidates.sell.candidates[0].entry.side, Side::Buy);
}

#[test]
fn profiles_missing_and_unique_references_without_temporal_claims() {
    let sources = [CmStreamSource {
        member: Some(1),
        source: ENTRY_FIXTURE,
    }];
    let mut delivery = split_order_delivery();
    delivery.members = vec![Some(1)];
    let orders = replay_trim_order_stream_set(&delivery, &sources).expect("independent replay");
    let event = trade();
    let profile = profile_trim_trade_stream_candidates(&orders, [&event]);

    assert_eq!(profile.trade_count, 1);
    assert_eq!(profile.reference_count, 2);
    assert_eq!(profile.candidate_count, 1);
    assert_eq!(profile.zero_candidate_references, 1);
    assert_eq!(profile.single_candidate_references, 1);
    assert_eq!(profile.multiple_candidate_references, 0);
    assert_eq!(profile.expected_side_candidates, 1);
    assert_eq!(profile.side_mismatch_candidates, 0);
    assert_eq!(profile.entry_before_trade_candidates, 1);
    assert_eq!(profile.entry_equal_trade_candidates, 0);
    assert_eq!(profile.entry_after_trade_candidates, 0);
    assert_eq!(profile.maximum_candidates_per_reference, 1);
    assert!(profile.counts_reconcile());
}

#[test]
fn profiles_multiple_and_side_mismatched_candidates_separately() {
    let mut wrong_side = sell_entry();
    wrong_side[36] = b'B';
    let sources = [
        CmStreamSource {
            member: Some(3),
            source: &wrong_side,
        },
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let mut delivery = split_order_delivery();
    delivery.members = vec![Some(1), Some(2), Some(3)];
    let orders = replay_trim_order_stream_set(&delivery, &sources).expect("independent replay");
    let event = trade();
    let profile = profile_trim_trade_stream_candidates(&orders, [&event]);

    assert_eq!(profile.trade_count, 1);
    assert_eq!(profile.reference_count, 2);
    assert_eq!(profile.candidate_count, 3);
    assert_eq!(profile.zero_candidate_references, 0);
    assert_eq!(profile.single_candidate_references, 1);
    assert_eq!(profile.multiple_candidate_references, 1);
    assert_eq!(profile.expected_side_candidates, 2);
    assert_eq!(profile.side_mismatch_candidates, 1);
    assert_eq!(profile.entry_before_trade_candidates, 2);
    assert_eq!(profile.entry_equal_trade_candidates, 1);
    assert_eq!(profile.entry_after_trade_candidates, 0);
    assert_eq!(profile.maximum_candidates_per_reference, 2);
    assert!(profile.counts_reconcile());
}

#[test]
fn profiles_entry_jiffy_relation_without_resolving_equal_jiffies() {
    let mut equal = ENTRY_FIXTURE.to_vec();
    equal[22..36].copy_from_slice(b"91200611817222");
    let mut after = ENTRY_FIXTURE.to_vec();
    after[22..36].copy_from_slice(b"91200611817223");
    let sources = [
        CmStreamSource {
            member: Some(3),
            source: &after,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(2),
            source: &equal,
        },
    ];
    let mut delivery = split_order_delivery();
    delivery.members = vec![Some(1), Some(2), Some(3)];
    let orders = replay_trim_order_stream_set(&delivery, &sources).expect("independent replay");
    let event = trade();
    let profile = profile_trim_trade_stream_candidates(&orders, [&event]);

    assert_eq!(profile.candidate_count, 3);
    assert_eq!(profile.entry_before_trade_candidates, 1);
    assert_eq!(profile.entry_equal_trade_candidates, 1);
    assert_eq!(profile.entry_after_trade_candidates, 1);
    assert!(profile.counts_reconcile());
}

#[test]
fn every_three_stream_candidate_configuration_matches_reference_counts() {
    const SOURCE_STATES: u32 = 6;
    const STREAM_COUNT: u32 = 3;
    let event = trade();
    let mut observed_dimensions = [false; 8];

    for encoded_configuration in 0..SOURCE_STATES.pow(STREAM_COUNT) {
        let mut encoded = encoded_configuration;
        let mut state = [0_u32; STREAM_COUNT as usize];
        for stream_state in &mut state {
            *stream_state = encoded % SOURCE_STATES;
            encoded /= SOURCE_STATES;
        }
        let generated = state.map(candidate_source);
        let sources = [
            CmStreamSource {
                member: Some(3),
                source: &generated[2].0,
            },
            CmStreamSource {
                member: Some(1),
                source: &generated[0].0,
            },
            CmStreamSource {
                member: Some(2),
                source: &generated[1].0,
            },
        ];
        let mut delivery = split_order_delivery();
        delivery.members = vec![Some(1), Some(2), Some(3)];
        let orders = replay_trim_order_stream_set(&delivery, &sources)
            .expect("three-stream candidate replay");
        let candidates = collect_trim_trade_stream_candidates(&orders, &event);
        let profile = profile_trim_trade_stream_candidates(&orders, [&event]);

        let mut expected_buy = Vec::new();
        let mut expected_sell = Vec::new();
        for (member_index, (_, specs)) in generated.iter().enumerate() {
            let member = u8::try_from(member_index + 1).expect("three members");
            for spec in specs {
                let expected = (
                    Some(member),
                    spec.side,
                    ExchangeJiffy::new(spec.jiffy),
                    SourceCoordinate {
                        record_index: spec.record_index,
                        byte_offset: spec.record_index * (TRIM_ORDER_RECORD_LEN + 1),
                    },
                );
                match spec.order_number {
                    1 => expected_buy.push(expected),
                    2 => expected_sell.push(expected),
                    _ => unreachable!("synthetic Trade references only Orders 1 and 2"),
                }
            }
        }
        let actual_buy = candidates
            .buy
            .candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.member,
                    candidate.entry.side,
                    candidate.entry.exchange_jiffy,
                    candidate.entry.coordinate,
                )
            })
            .collect::<Vec<_>>();
        let actual_sell = candidates
            .sell
            .candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.member,
                    candidate.entry.side,
                    candidate.entry.exchange_jiffy,
                    candidate.entry.coordinate,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual_buy, expected_buy,
            "configuration {encoded_configuration}"
        );
        assert_eq!(
            actual_sell, expected_sell,
            "configuration {encoded_configuration}"
        );

        let reference_lengths = [expected_buy.len(), expected_sell.len()];
        let all_candidates = expected_buy.iter().chain(&expected_sell);
        let expected_side_candidates = expected_buy
            .iter()
            .filter(|(_, side, _, _)| *side == Side::Buy)
            .count()
            + expected_sell
                .iter()
                .filter(|(_, side, _, _)| *side == Side::Sell)
                .count();
        let candidate_count = reference_lengths.iter().sum::<usize>();
        let before = all_candidates
            .clone()
            .filter(|(_, _, jiffy, _)| *jiffy < event.exchange_jiffy)
            .count();
        let equal = all_candidates
            .clone()
            .filter(|(_, _, jiffy, _)| *jiffy == event.exchange_jiffy)
            .count();
        let after = all_candidates
            .filter(|(_, _, jiffy, _)| *jiffy > event.exchange_jiffy)
            .count();
        let expected_profile = TradeOrderCandidateProfile {
            trade_count: 1,
            reference_count: 2,
            candidate_count,
            zero_candidate_references: reference_lengths
                .iter()
                .filter(|count| **count == 0)
                .count(),
            single_candidate_references: reference_lengths
                .iter()
                .filter(|count| **count == 1)
                .count(),
            multiple_candidate_references: reference_lengths
                .iter()
                .filter(|count| **count > 1)
                .count(),
            expected_side_candidates,
            side_mismatch_candidates: candidate_count - expected_side_candidates,
            entry_before_trade_candidates: before,
            entry_equal_trade_candidates: equal,
            entry_after_trade_candidates: after,
            maximum_candidates_per_reference: reference_lengths.into_iter().max().unwrap_or(0),
        };
        assert_eq!(
            profile, expected_profile,
            "configuration {encoded_configuration}"
        );
        assert!(profile.counts_reconcile());
        observed_dimensions[0] |= profile.zero_candidate_references > 0;
        observed_dimensions[1] |= profile.single_candidate_references > 0;
        observed_dimensions[2] |= profile.multiple_candidate_references > 0;
        observed_dimensions[3] |= profile.expected_side_candidates > 0;
        observed_dimensions[4] |= profile.side_mismatch_candidates > 0;
        observed_dimensions[5] |= profile.entry_before_trade_candidates > 0;
        observed_dimensions[6] |= profile.entry_equal_trade_candidates > 0;
        observed_dimensions[7] |= profile.entry_after_trade_candidates > 0;
    }

    assert!(observed_dimensions.into_iter().all(|observed| observed));
}

#[test]
fn empty_candidate_profile_is_zero_and_reconciled() {
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
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &sources)
        .expect("independent stream replay");
    let profile = profile_trim_trade_stream_candidates(&orders, std::iter::empty());

    assert_eq!(profile, Default::default());
    assert!(profile.counts_reconcile());
}

#[test]
fn profiles_each_raw_trade_stream_independently_in_member_order() {
    let sell = sell_entry();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: &sell,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &order_sources)
        .expect("independent order replay");
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: TRADE_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: TRADE_FIXTURE,
        },
    ];
    let delivery = split_trade_delivery(SourceDate::new(2026, 5, 18).expect("valid date"));
    let profile = profile_trim_trade_candidate_stream_set(&orders, &delivery, &trade_sources)
        .expect("trade candidate profile");

    assert!(profile.counts_reconcile());
    assert_eq!(
        profile
            .streams
            .iter()
            .map(|stream| stream.member)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    assert_eq!(profile.combined_value_summary.trade_count, 2);
    assert_eq!(profile.combined_value_summary.total_quantity, 50);
    assert_eq!(
        profile.combined_value_summary.total_notional_paise,
        12_500_000
    );
    for stream in profile.streams {
        assert_eq!(stream.source.framed_records, 1);
        assert_eq!(stream.source.parsed_records, 1);
        assert_eq!(stream.source.candidates.reference_count, 2);
        assert_eq!(stream.source.candidates.single_candidate_references, 2);
        assert_eq!(stream.source.candidates.expected_side_candidates, 2);
        assert_eq!(stream.source.candidates.entry_before_trade_candidates, 1);
        assert_eq!(stream.source.candidates.entry_equal_trade_candidates, 1);
        assert_eq!(stream.source.candidates.entry_after_trade_candidates, 0);
        assert_eq!(stream.source.value_summary.trade_count, 1);
        assert_eq!(stream.source.value_summary.total_quantity, 25);
        assert_eq!(stream.source.value_summary.total_notional_paise, 6_250_000);
        assert!(stream.source.quarantines.is_empty());
        assert!(stream.source.framing_error.is_none());
    }
}

#[test]
fn observes_every_successful_trade_with_complete_candidates_in_member_order() {
    let sell = sell_entry();
    let order_sources = [
        CmStreamSource {
            member: Some(2),
            source: &sell,
        },
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
    ];
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &order_sources)
        .expect("independent order replay");

    let mut invalid_trade = TRADE_FIXTURE.to_vec();
    invalid_trade[36] = b'X';
    let mut member_one_trades = [TRADE_FIXTURE, invalid_trade.as_slice()].concat();
    member_one_trades.extend_from_slice(b"partial-tail");
    let trade_sources = [
        CmStreamSource {
            member: Some(2),
            source: TRADE_FIXTURE,
        },
        CmStreamSource {
            member: Some(1),
            source: &member_one_trades,
        },
    ];
    let delivery = split_trade_delivery(SourceDate::new(2026, 5, 18).expect("valid date"));

    let mut observed = Vec::new();
    let profile = profile_trim_trade_candidate_stream_set_with_observer(
        &orders,
        &delivery,
        &trade_sources,
        |evidence| observed.push(evidence),
    )
    .expect("trade candidate profile");

    assert!(profile.counts_reconcile());
    assert_eq!(profile.streams[0].source.parsed_records, 1);
    assert_eq!(profile.streams[0].source.quarantines.len(), 1);
    assert!(profile.streams[0].source.framing_error.is_some());
    assert_eq!(profile.streams[1].source.parsed_records, 1);
    assert_eq!(observed.len(), 2);
    assert_eq!(
        observed
            .iter()
            .map(|evidence| evidence.member)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );

    let expected_event = trade();
    let expected_candidates = collect_trim_trade_stream_candidates(&orders, &expected_event);
    let expected_raw: [u8; TRIM_TRADE_RECORD_LEN] = TRADE_FIXTURE[..TRIM_TRADE_RECORD_LEN]
        .try_into()
        .expect("fixed-width trade record");
    for evidence in &observed {
        assert_eq!(
            evidence.coordinate,
            SourceCoordinate {
                record_index: 0,
                byte_offset: 0,
            }
        );
        assert_eq!(evidence.event, expected_event);
        assert_eq!(evidence.raw, expected_raw);
        assert_eq!(evidence.candidates, expected_candidates);
        assert_eq!(
            evidence
                .candidates
                .buy
                .candidates
                .iter()
                .map(|candidate| candidate.member)
                .collect::<Vec<_>>(),
            vec![Some(1)]
        );
        assert_eq!(
            evidence
                .candidates
                .sell
                .candidates
                .iter()
                .map(|candidate| candidate.member)
                .collect::<Vec<_>>(),
            vec![Some(2)]
        );
    }

    let reversed_sources = [trade_sources[1], trade_sources[0]];
    let mut reversed_observed = Vec::new();
    profile_trim_trade_candidate_stream_set_with_observer(
        &orders,
        &delivery,
        &reversed_sources,
        |evidence| reversed_observed.push(evidence),
    )
    .expect("reversed trade candidate profile");
    assert_eq!(reversed_observed, observed);
}

#[test]
fn rejects_order_and_trade_delivery_identity_mismatch_before_parsing() {
    let order_sources = [
        CmStreamSource {
            member: Some(1),
            source: ENTRY_FIXTURE,
        },
        CmStreamSource {
            member: Some(2),
            source: ENTRY_FIXTURE,
        },
    ];
    let orders = replay_trim_order_stream_set(&split_order_delivery(), &order_sources)
        .expect("independent order replay");
    let trade_sources = [
        CmStreamSource {
            member: Some(1),
            source: b"not parsed",
        },
        CmStreamSource {
            member: Some(2),
            source: b"not parsed",
        },
    ];
    let delivery = split_trade_delivery(SourceDate::new(2026, 5, 19).expect("valid date"));

    assert_eq!(
        profile_trim_trade_candidate_stream_set(&orders, &delivery, &trade_sources),
        Err(TradeCandidateStreamSetError::OrderTradeDeliveryMismatch {
            order_prefix: CmFilePrefix::Cm,
            trade_prefix: CmFilePrefix::Cm,
            order_source_date: SourceDate::new(2026, 5, 18).expect("valid date"),
            trade_source_date: SourceDate::new(2026, 5, 19).expect("valid date"),
        })
    );
}

#[test]
fn raw_trade_profile_retains_quarantine_and_framing_evidence() {
    let order_sources = [CmStreamSource {
        member: Some(1),
        source: ENTRY_FIXTURE,
    }];
    let mut order_delivery = split_order_delivery();
    order_delivery.members = vec![Some(1)];
    let orders = replay_trim_order_stream_set(&order_delivery, &order_sources)
        .expect("independent order replay");

    let mut invalid_trade = TRADE_FIXTURE.to_vec();
    invalid_trade[36] = b'X';
    let mut raw_trades = [TRADE_FIXTURE, invalid_trade.as_slice()].concat();
    raw_trades.extend_from_slice(b"partial-tail");
    let trade_sources = [CmStreamSource {
        member: Some(1),
        source: &raw_trades,
    }];
    let mut trade_delivery =
        split_trade_delivery(SourceDate::new(2026, 5, 18).expect("valid date"));
    trade_delivery.members = vec![Some(1)];
    let profile = profile_trim_trade_candidate_stream_set(&orders, &trade_delivery, &trade_sources)
        .expect("trade candidate profile");
    let source = &profile.streams[0].source;

    assert!(profile.counts_reconcile());
    assert_eq!(source.framed_records, 2);
    assert_eq!(source.source_bytes, raw_trades.len());
    assert_eq!(source.parsed_records, 1);
    assert_eq!(source.quarantines.len(), 1);
    assert_eq!(source.quarantines[0].coordinate.record_index, 1);
    assert_eq!(
        source.quarantines[0].raw_jiffy,
        Some(ExchangeJiffy::new(91_200_611_817_222))
    );
    assert_eq!(source.candidates.trade_count, 1);
    assert_eq!(source.value_summary.trade_count, 1);
    assert_eq!(source.value_summary.total_quantity, 25);
    assert_eq!(source.value_summary.total_notional_paise, 6_250_000);
    assert_eq!(
        source
            .framing_error
            .expect("partial tail must be preserved")
            .unconsumed_raw,
        b"partial-tail"
    );

    let mut invalid_profile = profile.clone();
    invalid_profile.streams[0].source.source_bytes += 1;
    assert!(!invalid_profile.counts_reconcile());
}

#[test]
fn trade_value_summary_fails_closed_on_every_accumulator_overflow() {
    let event = trade();

    assert_eq!(
        TradeValueSummary {
            trade_count: usize::MAX,
            ..Default::default()
        }
        .checked_add_trade(&event),
        Err(TradeValueSummaryError::TradeCountOverflow)
    );
    assert_eq!(
        TradeValueSummary {
            total_quantity: u128::MAX,
            ..Default::default()
        }
        .checked_add_trade(&event),
        Err(TradeValueSummaryError::TotalQuantityOverflow)
    );
    assert_eq!(
        TradeValueSummary {
            total_notional_paise: u128::MAX,
            ..Default::default()
        }
        .checked_add_trade(&event),
        Err(TradeValueSummaryError::TotalNotionalOverflow)
    );
    assert_eq!(
        TradeValueSummary {
            total_quantity: u128::MAX,
            ..Default::default()
        }
        .checked_merge(TradeValueSummary {
            total_quantity: 1,
            ..Default::default()
        }),
        Err(TradeValueSummaryError::TotalQuantityOverflow)
    );
}
