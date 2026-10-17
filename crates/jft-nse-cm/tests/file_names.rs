use jft_nse_cm::layouts::{SourceDate, SourceDateError};
use jft_nse_cm::manifest::{
    CmFileGroupKey, CmFileNameError, CmFilePrefix, CmProduct, CmRecordKind, CmSourceFileName,
    CmStreamForm, CmStreamSetError, discover_cm_stream_set, parse_cm_source_file_name,
};

fn parse_all(names: &[&str]) -> Vec<CmSourceFileName> {
    names
        .iter()
        .map(|name| parse_cm_source_file_name(name).expect("valid test name"))
        .collect()
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
