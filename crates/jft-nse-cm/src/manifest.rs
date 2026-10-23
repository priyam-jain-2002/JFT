//! Immutable input discovery, integrity evidence, and lineage manifests.

use std::fmt;

use md5::Md5;
use sha2::{Digest, Sha256};

use crate::layouts::{SourceDate, SourceDateError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    pub const HEX_LEN: usize = 64;

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        encode_lower_hex(&self.0)
    }

    pub fn from_hex(value: &str) -> Result<Self, HexDigestError> {
        decode_lower_or_upper_hex(value).map(Self)
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Md5Digest([u8; 16]);

impl Md5Digest {
    pub const HEX_LEN: usize = 32;

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        encode_lower_hex(&self.0)
    }

    pub fn from_hex(value: &str) -> Result<Self, HexDigestError> {
        decode_lower_or_upper_hex(value).map(Self)
    }
}

impl fmt::Display for Md5Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexDigestError {
    InvalidLength { expected: usize, actual: usize },
    InvalidCharacter { index: usize, byte: u8 },
}

#[must_use]
pub fn sha256_digest(payload: &[u8]) -> Sha256Digest {
    Sha256Digest::from_bytes(Sha256::digest(payload).into())
}

#[must_use]
pub fn md5_digest(payload: &[u8]) -> Md5Digest {
    Md5Digest::from_bytes(Md5::digest(payload).into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordIdentity {
    pub decompressed_payload_sha256: Sha256Digest,
    pub record_kind: CmRecordKind,
    pub zero_based_record_index: u64,
}

impl RecordIdentity {
    #[must_use]
    pub const fn new(
        decompressed_payload_sha256: Sha256Digest,
        record_kind: CmRecordKind,
        zero_based_record_index: u64,
    ) -> Self {
        Self {
            decompressed_payload_sha256,
            record_kind,
            zero_based_record_index,
        }
    }
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn decode_lower_or_upper_hex<const N: usize>(value: &str) -> Result<[u8; N], HexDigestError> {
    let expected = N * 2;
    if value.len() != expected {
        return Err(HexDigestError::InvalidLength {
            expected,
            actual: value.len(),
        });
    }

    let mut decoded = [0_u8; N];
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    debug_assert!(remainder.is_empty());
    for (index, pair) in pairs.iter().enumerate() {
        let high = decode_hex_nibble(pair[0], index * 2)?;
        let low = decode_hex_nibble(pair[1], index * 2 + 1)?;
        decoded[index] = (high << 4) | low;
    }
    Ok(decoded)
}

fn decode_hex_nibble(byte: u8, index: usize) -> Result<u8, HexDigestError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(HexDigestError::InvalidCharacter { index, byte }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadSignature {
    Empty,
    Gzip,
    Zip,
    AsciiText,
    UnknownBinary,
}

#[must_use]
pub fn detect_payload_signature(payload: &[u8]) -> PayloadSignature {
    if payload.is_empty() {
        return PayloadSignature::Empty;
    }
    if payload.starts_with(&[0x1f, 0x8b]) {
        return PayloadSignature::Gzip;
    }
    if payload.starts_with(b"PK\x03\x04")
        || payload.starts_with(b"PK\x05\x06")
        || payload.starts_with(b"PK\x07\x08")
    {
        return PayloadSignature::Zip;
    }
    if payload.iter().copied().all(is_ascii_text_byte) {
        return PayloadSignature::AsciiText;
    }
    PayloadSignature::UnknownBinary
}

const fn is_ascii_text_byte(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\r' | 0x20..=0x7e)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmFilePrefix {
    Cm,
    Cash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmProduct {
    Full,
    Trim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmRecordKind {
    Orders,
    Trades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmSourceFileName {
    pub prefix: CmFilePrefix,
    pub product: CmProduct,
    pub record_kind: CmRecordKind,
    pub source_date: SourceDate,
    pub stream: Option<u8>,
    pub is_trigger: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmFileNameError {
    MissingDataSuffix,
    UnknownPrefix,
    UnknownRecordKind,
    InvalidShape,
    InvalidDate(SourceDateError),
    InvalidStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmFileGroupKey {
    pub prefix: CmFilePrefix,
    pub product: CmProduct,
    pub record_kind: CmRecordKind,
    pub source_date: SourceDate,
    pub is_trigger: bool,
}

impl From<CmSourceFileName> for CmFileGroupKey {
    fn from(file: CmSourceFileName) -> Self {
        Self {
            prefix: file.prefix,
            product: file.product,
            record_kind: file.record_kind,
            source_date: file.source_date,
            is_trigger: file.is_trigger,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmStreamForm {
    Unsplit,
    Split { streams: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmStreamSet {
    pub key: CmFileGroupKey,
    pub form: CmStreamForm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmStreamSetError {
    Empty,
    MixedGroup {
        expected: CmFileGroupKey,
        actual: CmFileGroupKey,
    },
    MixedSplitMode,
    DuplicateStream {
        stream: Option<u8>,
    },
    MissingStream {
        stream: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmLogicalGroupKey {
    pub product: CmProduct,
    pub record_kind: CmRecordKind,
    pub source_date: SourceDate,
}

impl From<CmFileGroupKey> for CmLogicalGroupKey {
    fn from(key: CmFileGroupKey) -> Self {
        Self {
            product: key.product,
            record_kind: key.record_kind,
            source_date: key.source_date,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmPairedStreamSet {
    pub prefix: CmFilePrefix,
    pub key: CmLogicalGroupKey,
    pub members: Vec<Option<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmPairingError {
    DataSetHasTriggerRole,
    TriggerSetHasDataRole,
    PrefixMismatch {
        data: CmFilePrefix,
        trigger: CmFilePrefix,
    },
    LogicalGroupMismatch {
        data: CmLogicalGroupKey,
        trigger: CmLogicalGroupKey,
    },
    StreamFormMismatch,
    StreamMembersMismatch {
        data: Vec<u8>,
        trigger: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmDeliveryError {
    Empty,
    LogicalGroupMismatch {
        expected: CmLogicalGroupKey,
        actual: CmLogicalGroupKey,
    },
    MissingDataSet,
    MissingTriggerSet,
    InvalidDataSet(CmStreamSetError),
    InvalidTriggerSet(CmStreamSetError),
    Pairing(CmPairingError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmDeliveryDiscovery {
    pub key: CmLogicalGroupKey,
    pub outcome: Result<CmPairedStreamSet, CmDeliveryError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmRejectedSourceFileName<'a> {
    pub name: &'a str,
    pub error: CmFileNameError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmInventoryDiscovery<'a> {
    pub deliveries: Vec<CmDeliveryDiscovery>,
    pub rejected_names: Vec<CmRejectedSourceFileName<'a>>,
}

pub fn parse_cm_source_file_name(name: &str) -> Result<CmSourceFileName, CmFileNameError> {
    let (data_name, is_trigger) = match name.strip_suffix(".trg") {
        Some(data_name) => (data_name, true),
        None => (name, false),
    };
    let stem = data_name
        .strip_suffix(".DAT.gz")
        .ok_or(CmFileNameError::MissingDataSuffix)?;

    let (prefix, body) = if let Some(body) = stem.strip_prefix("CM_") {
        (CmFilePrefix::Cm, body)
    } else if let Some(body) = stem.strip_prefix("CASH_") {
        (CmFilePrefix::Cash, body)
    } else {
        return Err(CmFileNameError::UnknownPrefix);
    };

    let components = body.split('_').collect::<Vec<_>>();
    let (kind, product, date, stream) = match components.as_slice() {
        [kind, date] => (*kind, CmProduct::Full, *date, None),
        [kind, "T", date] => (*kind, CmProduct::Trim, *date, None),
        [kind, date, stream] => (*kind, CmProduct::Full, *date, Some(*stream)),
        [kind, "T", date, stream] => (*kind, CmProduct::Trim, *date, Some(*stream)),
        _ => return Err(CmFileNameError::InvalidShape),
    };

    let record_kind = match kind {
        "Orders" => CmRecordKind::Orders,
        "Trades" => CmRecordKind::Trades,
        _ => return Err(CmFileNameError::UnknownRecordKind),
    };

    Ok(CmSourceFileName {
        prefix,
        product,
        record_kind,
        source_date: parse_ddmmyyyy(date)?,
        stream: stream.map(parse_stream).transpose()?,
        is_trigger,
    })
}

pub fn discover_cm_stream_set(files: &[CmSourceFileName]) -> Result<CmStreamSet, CmStreamSetError> {
    let first = *files.first().ok_or(CmStreamSetError::Empty)?;
    let key = CmFileGroupKey::from(first);

    for file in files.iter().copied().skip(1) {
        let actual = CmFileGroupKey::from(file);
        if actual != key {
            return Err(CmStreamSetError::MixedGroup {
                expected: key,
                actual,
            });
        }
    }

    let has_unsplit = files.iter().any(|file| file.stream.is_none());
    let has_split = files.iter().any(|file| file.stream.is_some());
    if has_unsplit && has_split {
        return Err(CmStreamSetError::MixedSplitMode);
    }
    if has_unsplit {
        if files.len() != 1 {
            return Err(CmStreamSetError::DuplicateStream { stream: None });
        }
        return Ok(CmStreamSet {
            key,
            form: CmStreamForm::Unsplit,
        });
    }

    let mut seen = [false; 100];
    let mut maximum = 0_u8;
    for file in files {
        let stream = file.stream.expect("split mode was established");
        if seen[usize::from(stream)] {
            return Err(CmStreamSetError::DuplicateStream {
                stream: Some(stream),
            });
        }
        seen[usize::from(stream)] = true;
        maximum = maximum.max(stream);
    }

    for stream in 1..=maximum {
        if !seen[usize::from(stream)] {
            return Err(CmStreamSetError::MissingStream { stream });
        }
    }

    Ok(CmStreamSet {
        key,
        form: CmStreamForm::Split {
            streams: (1..=maximum).collect(),
        },
    })
}

pub fn pair_cm_data_and_triggers(
    data: &CmStreamSet,
    triggers: &CmStreamSet,
) -> Result<CmPairedStreamSet, CmPairingError> {
    if data.key.is_trigger {
        return Err(CmPairingError::DataSetHasTriggerRole);
    }
    if !triggers.key.is_trigger {
        return Err(CmPairingError::TriggerSetHasDataRole);
    }
    if data.key.prefix != triggers.key.prefix {
        return Err(CmPairingError::PrefixMismatch {
            data: data.key.prefix,
            trigger: triggers.key.prefix,
        });
    }

    let data_key = CmLogicalGroupKey::from(data.key);
    let trigger_key = CmLogicalGroupKey::from(triggers.key);
    if data_key != trigger_key {
        return Err(CmPairingError::LogicalGroupMismatch {
            data: data_key,
            trigger: trigger_key,
        });
    }

    let members = match (&data.form, &triggers.form) {
        (CmStreamForm::Unsplit, CmStreamForm::Unsplit) => vec![None],
        (CmStreamForm::Split { streams: data }, CmStreamForm::Split { streams: trigger }) => {
            if data != trigger {
                return Err(CmPairingError::StreamMembersMismatch {
                    data: data.clone(),
                    trigger: trigger.clone(),
                });
            }
            data.iter().copied().map(Some).collect()
        }
        _ => return Err(CmPairingError::StreamFormMismatch),
    };

    Ok(CmPairedStreamSet {
        prefix: data.key.prefix,
        key: data_key,
        members,
    })
}

pub fn discover_cm_delivery(
    files: &[CmSourceFileName],
) -> Result<CmPairedStreamSet, CmDeliveryError> {
    let first = *files.first().ok_or(CmDeliveryError::Empty)?;
    let expected = CmLogicalGroupKey::from(CmFileGroupKey::from(first));

    let mut data = Vec::new();
    let mut triggers = Vec::new();
    for file in files.iter().copied() {
        let actual = CmLogicalGroupKey::from(CmFileGroupKey::from(file));
        if actual != expected {
            return Err(CmDeliveryError::LogicalGroupMismatch { expected, actual });
        }
        if file.is_trigger {
            triggers.push(file);
        } else {
            data.push(file);
        }
    }

    if data.is_empty() {
        return Err(CmDeliveryError::MissingDataSet);
    }
    if triggers.is_empty() {
        return Err(CmDeliveryError::MissingTriggerSet);
    }

    let data = discover_cm_stream_set(&data).map_err(CmDeliveryError::InvalidDataSet)?;
    let triggers = discover_cm_stream_set(&triggers).map_err(CmDeliveryError::InvalidTriggerSet)?;
    pair_cm_data_and_triggers(&data, &triggers).map_err(CmDeliveryError::Pairing)
}

pub fn discover_cm_inventory<'a>(names: &[&'a str]) -> CmInventoryDiscovery<'a> {
    let mut files = Vec::new();
    let mut rejected_names = Vec::new();

    for name in names {
        match parse_cm_source_file_name(name) {
            Ok(file) => files.push(file),
            Err(error) => rejected_names.push(CmRejectedSourceFileName { name, error }),
        }
    }

    files.sort_by(compare_source_file_names);
    rejected_names.sort_by_key(|rejected| rejected.name);

    let mut groups: Vec<(CmLogicalGroupKey, Vec<CmSourceFileName>)> = Vec::new();
    for file in files {
        let key = CmLogicalGroupKey::from(CmFileGroupKey::from(file));
        match groups.last_mut() {
            Some((last_key, group)) if *last_key == key => group.push(file),
            _ => groups.push((key, vec![file])),
        }
    }

    let deliveries = groups
        .into_iter()
        .map(|(key, files)| CmDeliveryDiscovery {
            key,
            outcome: discover_cm_delivery(&files),
        })
        .collect();

    CmInventoryDiscovery {
        deliveries,
        rejected_names,
    }
}

fn compare_source_file_names(
    left: &CmSourceFileName,
    right: &CmSourceFileName,
) -> std::cmp::Ordering {
    let left_key = CmLogicalGroupKey::from(CmFileGroupKey::from(*left));
    let right_key = CmLogicalGroupKey::from(CmFileGroupKey::from(*right));
    compare_logical_group_keys(&left_key, &right_key)
        .then_with(|| left.is_trigger.cmp(&right.is_trigger))
        .then_with(|| prefix_order(left.prefix).cmp(&prefix_order(right.prefix)))
        .then_with(|| left.stream.cmp(&right.stream))
}

fn compare_logical_group_keys(
    left: &CmLogicalGroupKey,
    right: &CmLogicalGroupKey,
) -> std::cmp::Ordering {
    left.source_date
        .cmp(&right.source_date)
        .then_with(|| product_order(left.product).cmp(&product_order(right.product)))
        .then_with(|| {
            record_kind_order(left.record_kind).cmp(&record_kind_order(right.record_kind))
        })
}

const fn prefix_order(prefix: CmFilePrefix) -> u8 {
    match prefix {
        CmFilePrefix::Cm => 0,
        CmFilePrefix::Cash => 1,
    }
}

const fn product_order(product: CmProduct) -> u8 {
    match product {
        CmProduct::Full => 0,
        CmProduct::Trim => 1,
    }
}

const fn record_kind_order(record_kind: CmRecordKind) -> u8 {
    match record_kind {
        CmRecordKind::Orders => 0,
        CmRecordKind::Trades => 1,
    }
}

fn parse_ddmmyyyy(value: &str) -> Result<SourceDate, CmFileNameError> {
    if value.len() != 8 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CmFileNameError::InvalidShape);
    }

    let day = value[0..2]
        .parse::<u8>()
        .map_err(|_| CmFileNameError::InvalidShape)?;
    let month = value[2..4]
        .parse::<u8>()
        .map_err(|_| CmFileNameError::InvalidShape)?;
    let year = value[4..8]
        .parse::<u16>()
        .map_err(|_| CmFileNameError::InvalidShape)?;
    SourceDate::new(year, month, day).map_err(CmFileNameError::InvalidDate)
}

fn parse_stream(value: &str) -> Result<u8, CmFileNameError> {
    if value.len() != 2 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CmFileNameError::InvalidStream);
    }

    let stream = value
        .parse::<u8>()
        .map_err(|_| CmFileNameError::InvalidStream)?;
    if stream == 0 {
        return Err(CmFileNameError::InvalidStream);
    }
    Ok(stream)
}
