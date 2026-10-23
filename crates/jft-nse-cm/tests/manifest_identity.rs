use jft_nse_cm::manifest::{
    CmRecordKind, HexDigestError, Md5Digest, RecordIdentity, Sha256Digest, md5_digest,
    sha256_digest,
};

#[test]
fn matches_published_empty_and_abc_digest_vectors() {
    assert_eq!(
        sha256_digest(b"").to_hex(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_digest(b"abc").to_hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(md5_digest(b"").to_hex(), "d41d8cd98f00b204e9800998ecf8427e");
    assert_eq!(
        md5_digest(b"abc").to_hex(),
        "900150983cd24fb0d6963f7d28e17f72"
    );
}

#[test]
fn digest_hex_round_trips_and_normalizes_case() {
    let sha = sha256_digest(b"manifest identity");
    let md5 = md5_digest(b"manifest identity");

    assert_eq!(Sha256Digest::from_hex(&sha.to_hex()), Ok(sha));
    assert_eq!(Md5Digest::from_hex(&md5.to_hex()), Ok(md5));
    assert_eq!(
        Sha256Digest::from_hex(&sha.to_hex().to_ascii_uppercase()),
        Ok(sha)
    );
    assert_eq!(
        Md5Digest::from_hex(&md5.to_hex().to_ascii_uppercase()),
        Ok(md5)
    );
}

#[test]
fn digest_hex_rejects_wrong_lengths_and_reports_the_exact_bad_byte() {
    assert_eq!(
        Sha256Digest::from_hex("00"),
        Err(HexDigestError::InvalidLength {
            expected: Sha256Digest::HEX_LEN,
            actual: 2,
        })
    );
    assert_eq!(
        Md5Digest::from_hex("0000000000000000000000000000000X"),
        Err(HexDigestError::InvalidCharacter {
            index: 31,
            byte: b'X',
        })
    );
}

#[test]
fn one_byte_tampering_changes_both_integrity_digests() {
    let original = b"immutable payload";
    let tampered = b"immutable payloae";

    assert_ne!(sha256_digest(original), sha256_digest(tampered));
    assert_ne!(md5_digest(original), md5_digest(tampered));
}

#[test]
fn record_identity_is_exactly_payload_kind_and_zero_based_index() {
    let payload = sha256_digest(b"same decompressed bytes");
    let baseline = RecordIdentity::new(payload, CmRecordKind::Orders, 0);

    assert_eq!(
        baseline,
        RecordIdentity::new(payload, CmRecordKind::Orders, 0)
    );
    assert_ne!(
        baseline,
        RecordIdentity::new(payload, CmRecordKind::Trades, 0)
    );
    assert_ne!(
        baseline,
        RecordIdentity::new(payload, CmRecordKind::Orders, 1)
    );
    assert_ne!(
        baseline,
        RecordIdentity::new(
            sha256_digest(b"different decompressed bytes"),
            CmRecordKind::Orders,
            0,
        )
    );
}
