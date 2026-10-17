use jft_nse_cm::layouts::{SourceDate, SourceDateError};
use jft_nse_cm::manifest::{
    CmDeliveryError, CmFileGroupKey, CmFileNameError, CmFilePrefix, CmLogicalGroupKey,
    CmPairingError, CmProduct, CmRecordKind, CmSourceFileName, CmStreamForm, CmStreamSet,
    CmStreamSetError, discover_cm_delivery, discover_cm_inventory, discover_cm_stream_set,
    pair_cm_data_and_triggers, parse_cm_source_file_name,
};

fn parse_all(names: &[&str]) -> Vec<CmSourceFileName> {
    names
        .iter()
        .map(|name| parse_cm_source_file_name(name).expect("valid test name"))
        .collect()
}

fn discover(names: &[&str]) -> CmStreamSet {
    discover_cm_stream_set(&parse_all(names)).expect("valid test set")
}

#[test]
fn parses_current_trim_data_name_without_collapsing_cm_prefix() {
    assert_eq!(
        parse_cm_source_file_name("CM_Orders_T_18052026_01.DAT.gz"),
        Ok(CmSourceFileName {
            prefix: CmFilePrefix::Cm,
            product: CmProduct::Trim,
            record_kind: CmRecordKind::Orders,
            source_date: SourceDate::new(2026, 5, 18).expect("valid date"),
            stream: Some(1),
            is_trigger: false,
        })
    );

    let unsplit =
        parse_cm_source_file_name("CM_Trades_T_18052026.DAT.gz").expect("valid unsplit Trim name");
    assert_eq!(unsplit.product, CmProduct::Trim);
    assert_eq!(unsplit.record_kind, CmRecordKind::Trades);
    assert_eq!(unsplit.stream, None);
}

#[test]
fn parses_trigger_table_cash_prefix_as_distinct_evidence() {
    assert_eq!(
        parse_cm_source_file_name("CASH_Trades_T_18052026_12.DAT.gz.trg"),
        Ok(CmSourceFileName {
            prefix: CmFilePrefix::Cash,
            product: CmProduct::Trim,
            record_kind: CmRecordKind::Trades,
            source_date: SourceDate::new(2026, 5, 18).expect("valid date"),
            stream: Some(12),
            is_trigger: true,
        })
    );
}

#[test]
fn parses_full_split_and_unsplit_names() {
    let split =
        parse_cm_source_file_name("CASH_Orders_16062025_02.DAT.gz").expect("valid split Full name");
    assert_eq!(split.product, CmProduct::Full);
    assert_eq!(split.record_kind, CmRecordKind::Orders);
    assert_eq!(split.stream, Some(2));

    let unsplit =
        parse_cm_source_file_name("CM_Trades_30062024.DAT.gz").expect("valid unsplit Full name");
    assert_eq!(unsplit.product, CmProduct::Full);
    assert_eq!(unsplit.record_kind, CmRecordKind::Trades);
    assert_eq!(unsplit.stream, None);
}

#[test]
fn rejects_invalid_calendar_date_with_exact_reason() {
    assert_eq!(
        parse_cm_source_file_name("CM_Orders_29022025_01.DAT.gz"),
        Err(CmFileNameError::InvalidDate(SourceDateError::InvalidDay {
            year: 2025,
            month: 2,
            day: 29,
            maximum: 28,
        }))
    );
}

#[test]
fn rejects_zero_wide_and_non_numeric_streams() {
    for name in [
        "CM_Orders_16062025_00.DAT.gz",
        "CM_Orders_16062025_1.DAT.gz",
        "CM_Orders_16062025_001.DAT.gz",
        "CM_Orders_16062025_A1.DAT.gz",
    ] {
        assert_eq!(
            parse_cm_source_file_name(name),
            Err(CmFileNameError::InvalidStream),
            "{name}"
        );
    }
}

#[test]
fn rejects_unknown_prefix_kind_shape_and_suffix() {
    assert_eq!(
        parse_cm_source_file_name("NSE_Orders_16062025_01.DAT.gz"),
        Err(CmFileNameError::UnknownPrefix)
    );
    assert_eq!(
        parse_cm_source_file_name("CM_Index_16062025_01.DAT.gz"),
        Err(CmFileNameError::UnknownRecordKind)
    );
    assert_eq!(
        parse_cm_source_file_name("CM_Orders_X_16062025_01.DAT.gz"),
        Err(CmFileNameError::InvalidShape)
    );
    assert_eq!(
        parse_cm_source_file_name("CM_Orders_16062025_01.dat.gz"),
        Err(CmFileNameError::MissingDataSuffix)
    );
}

#[test]
fn discovers_all_streams_in_numeric_order_without_a_fixed_count() {
    let files = parse_all(&[
        "CASH_Orders_16062025_03.DAT.gz",
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Orders_16062025_02.DAT.gz",
    ]);
    let discovered = discover_cm_stream_set(&files).expect("complete stream set");

    assert_eq!(
        discovered.form,
        CmStreamForm::Split {
            streams: vec![1, 2, 3]
        }
    );
    assert_eq!(discovered.key.prefix, CmFilePrefix::Cash);
    assert_eq!(discovered.key.record_kind, CmRecordKind::Orders);
}

#[test]
fn rejects_the_first_missing_stream() {
    let files = parse_all(&[
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Orders_16062025_03.DAT.gz",
        "CASH_Orders_16062025_04.DAT.gz",
    ]);

    assert_eq!(
        discover_cm_stream_set(&files),
        Err(CmStreamSetError::MissingStream { stream: 2 })
    );
}

#[test]
fn rejects_duplicate_split_and_unsplit_members() {
    let duplicate_split = parse_all(&[
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Orders_16062025_01.DAT.gz",
    ]);
    assert_eq!(
        discover_cm_stream_set(&duplicate_split),
        Err(CmStreamSetError::DuplicateStream { stream: Some(1) })
    );

    let duplicate_unsplit = parse_all(&["CM_Trades_30062024.DAT.gz", "CM_Trades_30062024.DAT.gz"]);
    assert_eq!(
        discover_cm_stream_set(&duplicate_unsplit),
        Err(CmStreamSetError::DuplicateStream { stream: None })
    );
}

#[test]
fn rejects_mixed_split_mode_and_mixed_group_metadata() {
    let mixed_mode = parse_all(&[
        "CASH_Orders_16062025.DAT.gz",
        "CASH_Orders_16062025_01.DAT.gz",
    ]);
    assert_eq!(
        discover_cm_stream_set(&mixed_mode),
        Err(CmStreamSetError::MixedSplitMode)
    );

    let mixed_group = parse_all(&[
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Trades_16062025_01.DAT.gz",
    ]);
    let expected = CmFileGroupKey::from(mixed_group[0]);
    let actual = CmFileGroupKey::from(mixed_group[1]);
    assert_eq!(
        discover_cm_stream_set(&mixed_group),
        Err(CmStreamSetError::MixedGroup { expected, actual })
    );
}

#[test]
fn accepts_exactly_one_unsplit_member() {
    let files = parse_all(&["CM_Trades_30062024.DAT.gz"]);
    assert_eq!(
        discover_cm_stream_set(&files)
            .expect("one unsplit file")
            .form,
        CmStreamForm::Unsplit
    );
}

#[test]
fn rejects_an_empty_discovery_input() {
    assert_eq!(discover_cm_stream_set(&[]), Err(CmStreamSetError::Empty));
}

#[test]
fn pairs_matching_split_data_and_trigger_members() {
    let data = discover(&[
        "CASH_Orders_16062025_02.DAT.gz",
        "CASH_Orders_16062025_01.DAT.gz",
    ]);
    let triggers = discover(&[
        "CASH_Orders_16062025_01.DAT.gz.trg",
        "CASH_Orders_16062025_02.DAT.gz.trg",
    ]);

    let paired = pair_cm_data_and_triggers(&data, &triggers).expect("matching delivery");
    assert_eq!(paired.prefix, CmFilePrefix::Cash);
    assert_eq!(paired.key.product, CmProduct::Full);
    assert_eq!(paired.key.record_kind, CmRecordKind::Orders);
    assert_eq!(paired.members, vec![Some(1), Some(2)]);
}

#[test]
fn pairs_matching_unsplit_data_and_trigger_members() {
    let data = discover(&["CM_Trades_30062024.DAT.gz"]);
    let triggers = discover(&["CM_Trades_30062024.DAT.gz.trg"]);

    assert_eq!(
        pair_cm_data_and_triggers(&data, &triggers)
            .expect("matching unsplit delivery")
            .members,
        vec![None]
    );
}

#[test]
fn rejects_data_and_trigger_member_mismatch() {
    let data = discover(&[
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Orders_16062025_02.DAT.gz",
    ]);
    let triggers = discover(&["CASH_Orders_16062025_01.DAT.gz.trg"]);

    assert_eq!(
        pair_cm_data_and_triggers(&data, &triggers),
        Err(CmPairingError::StreamMembersMismatch {
            data: vec![1, 2],
            trigger: vec![1],
        })
    );
}

#[test]
fn reports_prefix_conflict_instead_of_aliasing_it() {
    let data = discover(&["CM_Orders_T_18052026_01.DAT.gz"]);
    let triggers = discover(&["CASH_Orders_T_18052026_01.DAT.gz.trg"]);

    assert_eq!(
        pair_cm_data_and_triggers(&data, &triggers),
        Err(CmPairingError::PrefixMismatch {
            data: CmFilePrefix::Cm,
            trigger: CmFilePrefix::Cash,
        })
    );
}

#[test]
fn rejects_logical_group_and_stream_form_mismatches() {
    let data = discover(&["CASH_Orders_16062025_01.DAT.gz"]);
    let wrong_kind = discover(&["CASH_Trades_16062025_01.DAT.gz.trg"]);
    assert_eq!(
        pair_cm_data_and_triggers(&data, &wrong_kind),
        Err(CmPairingError::LogicalGroupMismatch {
            data: CmLogicalGroupKey::from(data.key),
            trigger: CmLogicalGroupKey::from(wrong_kind.key),
        })
    );

    let unsplit_trigger = discover(&["CASH_Orders_16062025.DAT.gz.trg"]);
    assert_eq!(
        pair_cm_data_and_triggers(&data, &unsplit_trigger),
        Err(CmPairingError::StreamFormMismatch)
    );
}

#[test]
fn rejects_reversed_data_and_trigger_roles() {
    let data = discover(&["CASH_Orders_16062025_01.DAT.gz"]);
    let triggers = discover(&["CASH_Orders_16062025_01.DAT.gz.trg"]);

    assert_eq!(
        pair_cm_data_and_triggers(&triggers, &data),
        Err(CmPairingError::DataSetHasTriggerRole)
    );
    assert_eq!(
        pair_cm_data_and_triggers(&data, &data),
        Err(CmPairingError::TriggerSetHasDataRole)
    );
}

#[test]
fn discovers_a_complete_delivery_from_arbitrary_member_order() {
    let files = parse_all(&[
        "CASH_Trades_T_18052026_02.DAT.gz.trg",
        "CASH_Trades_T_18052026_01.DAT.gz",
        "CASH_Trades_T_18052026_01.DAT.gz.trg",
        "CASH_Trades_T_18052026_02.DAT.gz",
    ]);

    let delivery = discover_cm_delivery(&files).expect("complete paired delivery");
    assert_eq!(delivery.prefix, CmFilePrefix::Cash);
    assert_eq!(delivery.key.product, CmProduct::Trim);
    assert_eq!(delivery.key.record_kind, CmRecordKind::Trades);
    assert_eq!(delivery.members, vec![Some(1), Some(2)]);
}

#[test]
fn delivery_rejects_empty_and_missing_roles() {
    assert_eq!(discover_cm_delivery(&[]), Err(CmDeliveryError::Empty));

    let only_data = parse_all(&["CASH_Orders_16062025_01.DAT.gz"]);
    assert_eq!(
        discover_cm_delivery(&only_data),
        Err(CmDeliveryError::MissingTriggerSet)
    );

    let only_trigger = parse_all(&["CASH_Orders_16062025_01.DAT.gz.trg"]);
    assert_eq!(
        discover_cm_delivery(&only_trigger),
        Err(CmDeliveryError::MissingDataSet)
    );
}

#[test]
fn delivery_rejects_mixed_logical_groups() {
    let files = parse_all(&[
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Trades_16062025_01.DAT.gz.trg",
    ]);
    let expected = CmLogicalGroupKey::from(CmFileGroupKey::from(files[0]));
    let actual = CmLogicalGroupKey::from(CmFileGroupKey::from(files[1]));

    assert_eq!(
        discover_cm_delivery(&files),
        Err(CmDeliveryError::LogicalGroupMismatch { expected, actual })
    );
}

#[test]
fn delivery_preserves_nested_stream_set_failure_evidence() {
    let files = parse_all(&[
        "CASH_Orders_16062025_01.DAT.gz",
        "CASH_Orders_16062025_03.DAT.gz",
        "CASH_Orders_16062025_01.DAT.gz.trg",
        "CASH_Orders_16062025_02.DAT.gz.trg",
        "CASH_Orders_16062025_03.DAT.gz.trg",
    ]);

    assert_eq!(
        discover_cm_delivery(&files),
        Err(CmDeliveryError::InvalidDataSet(
            CmStreamSetError::MissingStream { stream: 2 }
        ))
    );
}

#[test]
fn delivery_preserves_prefix_conflict_evidence() {
    let files = parse_all(&[
        "CM_Orders_T_18052026_01.DAT.gz",
        "CASH_Orders_T_18052026_01.DAT.gz.trg",
    ]);

    assert_eq!(
        discover_cm_delivery(&files),
        Err(CmDeliveryError::Pairing(CmPairingError::PrefixMismatch {
            data: CmFilePrefix::Cm,
            trigger: CmFilePrefix::Cash,
        }))
    );
}

#[test]
fn inventory_partitions_and_orders_multiple_deliveries_deterministically() {
    let inventory = [
        "CASH_Trades_T_18052026.DAT.gz.trg",
        "CASH_Orders_16062025.DAT.gz",
        "CASH_Trades_T_18052026.DAT.gz",
        "CASH_Orders_16062025.DAT.gz.trg",
        "CASH_Trades_16062025.DAT.gz.trg",
        "CASH_Trades_16062025.DAT.gz",
    ];

    let discovery = discover_cm_inventory(&inventory);
    let keys = discovery
        .deliveries
        .iter()
        .map(|delivery| delivery.key)
        .collect::<Vec<_>>();

    assert_eq!(discovery.rejected_names, vec![]);
    assert!(discovery.deliveries.iter().all(|item| item.outcome.is_ok()));
    assert_eq!(keys[0].source_date, SourceDate::new(2025, 6, 16).unwrap());
    assert_eq!(keys[0].product, CmProduct::Full);
    assert_eq!(keys[0].record_kind, CmRecordKind::Orders);
    assert_eq!(keys[1].source_date, SourceDate::new(2025, 6, 16).unwrap());
    assert_eq!(keys[1].product, CmProduct::Full);
    assert_eq!(keys[1].record_kind, CmRecordKind::Trades);
    assert_eq!(keys[2].source_date, SourceDate::new(2026, 5, 18).unwrap());
    assert_eq!(keys[2].product, CmProduct::Trim);
    assert_eq!(keys[2].record_kind, CmRecordKind::Trades);
}

#[test]
fn inventory_reports_a_bad_group_without_hiding_a_good_group() {
    let inventory = [
        "CASH_Orders_16062025.DAT.gz",
        "CASH_Trades_16062025.DAT.gz",
        "CASH_Trades_16062025.DAT.gz.trg",
    ];

    let discovery = discover_cm_inventory(&inventory);
    assert_eq!(discovery.deliveries.len(), 2);
    assert_eq!(
        discovery.deliveries[0].outcome,
        Err(CmDeliveryError::MissingTriggerSet)
    );
    assert!(discovery.deliveries[1].outcome.is_ok());
}

#[test]
fn inventory_retains_and_sorts_every_rejected_file_name() {
    let inventory = [
        "Z_BAD.DAT.gz",
        "CASH_Orders_31022025.DAT.gz",
        "CASH_Orders_16062025.DAT.gz.trg",
        "CASH_Orders_16062025.DAT.gz",
        "A_BAD",
    ];

    let discovery = discover_cm_inventory(&inventory);
    let rejected = discovery
        .rejected_names
        .iter()
        .map(|item| (item.name, item.error))
        .collect::<Vec<_>>();

    assert_eq!(discovery.deliveries.len(), 1);
    assert!(discovery.deliveries[0].outcome.is_ok());
    assert_eq!(
        rejected,
        vec![
            ("A_BAD", CmFileNameError::MissingDataSuffix),
            (
                "CASH_Orders_31022025.DAT.gz",
                CmFileNameError::InvalidDate(SourceDateError::InvalidDay {
                    year: 2025,
                    month: 2,
                    day: 31,
                    maximum: 28,
                })
            ),
            ("Z_BAD.DAT.gz", CmFileNameError::UnknownPrefix),
        ]
    );
}

#[test]
fn inventory_result_is_identical_when_input_order_is_reversed() {
    let inventory = [
        "CASH_Orders_16062025_02.DAT.gz.trg",
        "CM_Trades_T_18052026_01.DAT.gz",
        "CASH_Orders_16062025_01.DAT.gz",
        "NOT_CM.gz",
        "CASH_Orders_16062025_01.DAT.gz.trg",
        "CM_Trades_T_18052026_01.DAT.gz.trg",
        "CASH_Orders_16062025_02.DAT.gz",
        "CASH_Trades_T_18052026_01.DAT.gz",
    ];
    let mut reversed = inventory;
    reversed.reverse();

    assert_eq!(
        discover_cm_inventory(&inventory),
        discover_cm_inventory(&reversed)
    );
}

#[test]
fn empty_inventory_has_no_deliveries_or_rejections() {
    let discovery = discover_cm_inventory(&[]);

    assert_eq!(discovery.deliveries, vec![]);
    assert_eq!(discovery.rejected_names, vec![]);
}
