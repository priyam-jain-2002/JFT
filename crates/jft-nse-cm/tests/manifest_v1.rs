use jft_nse_cm::layouts::SourceDate;
use jft_nse_cm::manifest::v1::{
    CompressedIntegrity, MANIFEST_SCHEMA_ID, MANIFEST_SCHEMA_MAJOR, ManifestJsonError,
    ManifestSource, ManifestSourceDate, ManifestV1, ManifestValidationError, PayloadIntegrity,
    ProcessingIdentity, TriggerDeclaration, TriggerEvidence, TriggerVerificationStatus,
};
use jft_nse_cm::manifest::{CmFilePrefix, CmProduct, CmRecordKind, md5_digest, sha256_digest};

const ORDER_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const GOLDEN_MANIFEST: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/manifest_v1.json");
const MANIFEST_SCHEMA: &[u8] = include_bytes!("../../../schemas/nse_cm/manifest.schema.json");

fn valid_manifest() -> ManifestV1 {
    let compressed = CompressedIntegrity::from_bytes(b"compressed bytes");
    ManifestV1::new(
        ManifestSource {
            logical_path: "CM_Orders_T_18052026_01.DAT.gz".to_owned(),
            prefix: CmFilePrefix::Cm,
            product: CmProduct::Trim,
            record_kind: CmRecordKind::Orders,
            source_date: ManifestSourceDate::from_source_date(
                SourceDate::new(2026, 5, 18).expect("valid date"),
            ),
            stream: Some(1),
            layout_id: "nse_cm_trim_v1_19_order_68".to_owned(),
        },
        compressed,
        TriggerEvidence::from_declaration(
            compressed,
            Some(TriggerDeclaration {
                byte_count: compressed.byte_count,
                md5: compressed.md5,
            }),
        ),
        PayloadIntegrity::from_bytes(
            ORDER_FIXTURE,
            68,
            1,
            1,
            0,
            0,
            Some((91_200_611_817_221, 91_200_611_817_221)),
        ),
        ProcessingIdentity {
            configuration_id: "fixture-config-v1".to_owned(),
            code_identity: "test-code".to_owned(),
        },
    )
}

#[test]
fn emits_byte_identical_golden_json_and_round_trips() {
    let manifest = valid_manifest();
    let first = manifest.to_json_bytes().expect("valid manifest JSON");
    let second = manifest.to_json_bytes().expect("repeat manifest JSON");

    assert_eq!(first, GOLDEN_MANIFEST);
    assert_eq!(second, first);
    assert_eq!(ManifestV1::from_json_bytes(&first), Ok(manifest.clone()));
    assert_eq!(ManifestV1::from_json_bytes(GOLDEN_MANIFEST), Ok(manifest));
}

#[test]
fn explicit_schema_is_valid_json_and_freezes_the_manifest_discriminator() {
    let schema: serde_json::Value = serde_json::from_slice(MANIFEST_SCHEMA).expect("valid schema");

    assert_eq!(
        schema["properties"]["schema_id"]["const"],
        MANIFEST_SCHEMA_ID
    );
    assert_eq!(
        schema["properties"]["schema_major"]["const"],
        MANIFEST_SCHEMA_MAJOR
    );
    assert_eq!(schema["additionalProperties"], false);
}

#[test]
fn derives_every_trigger_verification_status_from_compressed_evidence() {
    let compressed = CompressedIntegrity::from_bytes(b"compressed bytes");
    let wrong_md5 = md5_digest(b"different bytes");
    let cases = [
        (None, TriggerVerificationStatus::NotProvided),
        (
            Some(TriggerDeclaration {
                byte_count: compressed.byte_count,
                md5: compressed.md5,
            }),
            TriggerVerificationStatus::Verified,
        ),
        (
            Some(TriggerDeclaration {
                byte_count: compressed.byte_count + 1,
                md5: compressed.md5,
            }),
            TriggerVerificationStatus::SizeMismatch,
        ),
        (
            Some(TriggerDeclaration {
                byte_count: compressed.byte_count,
                md5: wrong_md5,
            }),
            TriggerVerificationStatus::Md5Mismatch,
        ),
        (
            Some(TriggerDeclaration {
                byte_count: compressed.byte_count + 1,
                md5: wrong_md5,
            }),
            TriggerVerificationStatus::SizeAndMd5Mismatch,
        ),
    ];

    for (declaration, expected) in cases {
        let evidence = TriggerEvidence::from_declaration(compressed, declaration);
        assert_eq!(evidence.status, expected);
        let mut manifest = valid_manifest();
        manifest.trigger = evidence;
        assert!(manifest.validate().is_ok());
    }
}

#[test]
fn rejects_absolute_parent_backslash_and_empty_logical_paths() {
    for path in [
        "",
        "/absolute/file.dat",
        "../escape.dat",
        "nested/../escape.dat",
        "nested//file.dat",
        "nested\\file.dat",
        "./file.dat",
    ] {
        let mut manifest = valid_manifest();
        manifest.source.logical_path = path.to_owned();
        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidLogicalPath),
            "{path}"
        );
    }
}

#[test]
fn rejects_unknown_schema_versions_and_unknown_json_fields() {
    let mut manifest = valid_manifest();
    manifest.schema_id = "another.schema".to_owned();
    assert_eq!(
        manifest.validate(),
        Err(ManifestValidationError::UnknownSchemaId)
    );

    manifest.schema_id = MANIFEST_SCHEMA_ID.to_owned();
    manifest.schema_major = 2;
    assert_eq!(
        manifest.validate(),
        Err(ManifestValidationError::UnsupportedSchemaMajor { actual: 2 })
    );

    let unknown_field = GOLDEN_MANIFEST
        .strip_suffix(b"}\n")
        .expect("golden ends with object and LF");
    let mut unknown_field = unknown_field.to_vec();
    unknown_field.extend_from_slice(b",\"unknown\":true}\n");
    assert!(matches!(
        ManifestV1::from_json_bytes(&unknown_field),
        Err(ManifestJsonError::Json(_))
    ));
}

#[test]
fn rejects_invalid_dates_streams_and_forged_trigger_status() {
    for stream in [Some(0), Some(100), Some(u8::MAX)] {
        let mut manifest = valid_manifest();
        manifest.source.stream = stream;
        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidStream)
        );
    }

    let invalid_date = GOLDEN_MANIFEST
        .split(|byte| *byte == b'\n')
        .next()
        .expect("golden JSON line");
    let invalid_date = String::from_utf8(invalid_date.to_vec())
        .expect("golden UTF-8")
        .replace("2026-05-18", "2025-02-29");
    assert!(matches!(
        ManifestV1::from_json_bytes(invalid_date.as_bytes()),
        Err(ManifestJsonError::Validation(
            ManifestValidationError::InvalidSourceDateValue(_)
        ))
    ));

    let mut manifest = valid_manifest();
    manifest.trigger.status = TriggerVerificationStatus::Md5Mismatch;
    assert!(matches!(
        manifest.validate(),
        Err(ManifestValidationError::TriggerStatusMismatch { .. })
    ));
}

#[test]
fn payload_validation_fails_closed_on_count_byte_and_jiffy_inconsistency() {
    let mut manifest = valid_manifest();
    manifest.payload.quarantined_records = 1;
    assert_eq!(
        manifest.validate(),
        Err(ManifestValidationError::RecordCountMismatch)
    );

    let mut manifest = valid_manifest();
    manifest.payload.byte_count += 1;
    assert_eq!(
        manifest.validate(),
        Err(ManifestValidationError::PayloadByteCountMismatch)
    );

    let mut manifest = valid_manifest();
    manifest.payload.raw_jiffy_maximum = None;
    assert_eq!(
        manifest.validate(),
        Err(ManifestValidationError::JiffyBoundsPresenceMismatch)
    );

    let mut manifest = valid_manifest();
    manifest.payload.raw_jiffy_minimum = Some(2);
    manifest.payload.raw_jiffy_maximum = Some(1);
    assert_eq!(
        manifest.validate(),
        Err(ManifestValidationError::ReversedJiffyBounds)
    );
}

#[test]
fn record_identity_is_payload_based_and_bounded_by_framed_records() {
    let manifest = valid_manifest();
    let identity = manifest.record_identity(0).expect("first record identity");

    assert_eq!(
        identity.decompressed_payload_sha256,
        sha256_digest(ORDER_FIXTURE)
    );
    assert_eq!(identity.record_kind, CmRecordKind::Orders);
    assert_eq!(identity.zero_based_record_index, 0);
    assert_eq!(manifest.record_identity(1), None);
}

#[test]
fn recompression_or_rename_does_not_change_record_identity() {
    let first = valid_manifest();
    let mut second = valid_manifest();
    second.source.logical_path = "renamed/container.dat.gz".to_owned();
    second.compressed = CompressedIntegrity::from_bytes(b"another compressed representation");
    second.trigger = TriggerEvidence::from_declaration(second.compressed, None);

    assert!(second.validate().is_ok());
    assert_ne!(first.compressed, second.compressed);
    assert_ne!(first.source.logical_path, second.source.logical_path);
    assert_eq!(first.record_identity(0), second.record_identity(0));
}
