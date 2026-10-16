use jft_nse_cm::layouts::{SourceDate, SourceDateError};
use jft_nse_cm::manifest::{
    CmFileNameError, CmFilePrefix, CmProduct, CmRecordKind, CmSourceFileName,
    parse_cm_source_file_name,
};

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
