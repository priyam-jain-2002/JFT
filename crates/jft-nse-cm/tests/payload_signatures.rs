use std::fs;
use std::path::PathBuf;

use jft_nse_cm::manifest::{PayloadSignature, detect_payload_signature};

#[test]
fn empty_payload_has_a_distinct_signature() {
    assert_eq!(detect_payload_signature(b""), PayloadSignature::Empty);
}

#[test]
fn gzip_magic_is_recognized_without_using_a_file_name() {
    assert_eq!(
        detect_payload_signature(&[0x1f, 0x8b, 0x08, 0x00]),
        PayloadSignature::Gzip
    );
}

#[test]
fn standard_zip_start_signatures_are_recognized() {
    for signature in [b"PK\x03\x04", b"PK\x05\x06", b"PK\x07\x08"] {
        assert_eq!(detect_payload_signature(signature), PayloadSignature::Zip);
    }
}

#[test]
fn printable_ascii_and_line_controls_are_text() {
    assert_eq!(
        detect_payload_signature(b"field|value\r\nnext\tvalue\n"),
        PayloadSignature::AsciiText
    );
}

#[test]
fn binary_and_truncated_magic_are_not_guessed() {
    assert_eq!(
        detect_payload_signature(&[0x00, 0x01, 0x02]),
        PayloadSignature::UnknownBinary
    );
    assert_eq!(
        detect_payload_signature(&[0xff, 0xfe]),
        PayloadSignature::UnknownBinary
    );
    assert_eq!(
        detect_payload_signature(b"PK\x03"),
        PayloadSignature::UnknownBinary
    );
}

#[test]
#[ignore = "requires locally extracted legacy CM EOD sample payloads"]
fn detects_supplied_legacy_cm_suffix_content_mismatches() {
    let root = PathBuf::from(
        std::env::var_os("JFT_LEGACY_CM_EOD_SAMPLE_DIR")
            .expect("JFT_LEGACY_CM_EOD_SAMPLE_DIR must point to extracted nested payloads"),
    );

    for name in [
        "CM_INDEX_ddmmyyyy.gz",
        "CM_SNAPSHOT_ddmmyyyy.gz",
        "CM_TRADES_ddmmyyyy.gz",
    ] {
        let payload = fs::read(root.join(name)).expect("read supplied plain-text payload");
        assert_eq!(
            detect_payload_signature(&payload),
            PayloadSignature::AsciiText,
            "{name}"
        );
    }

    for name in ["CM_MASTERS_012024.gz", "CM_CIRCULARS_012024.gz"] {
        let payload = fs::read(root.join(name)).expect("read supplied nested ZIP payload");
        assert_eq!(
            detect_payload_signature(&payload),
            PayloadSignature::Zip,
            "{name}"
        );
    }
}
