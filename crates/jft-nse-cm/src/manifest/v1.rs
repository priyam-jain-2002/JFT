//! Deterministic, language-neutral NSE CM manifest v1.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    CmFilePrefix, CmProduct, CmRecordKind, Md5Digest, RecordIdentity, Sha256Digest, md5_digest,
    sha256_digest,
};
use crate::layouts::{SourceDate, SourceDateError};

pub const MANIFEST_SCHEMA_ID: &str = "jft.nse_cm.manifest";
pub const MANIFEST_SCHEMA_MAJOR: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ManifestSourceDate(String);

impl ManifestSourceDate {
    #[must_use]
    pub fn from_source_date(value: SourceDate) -> Self {
        Self(format!(
            "{:04}-{:02}-{:02}",
            value.year(),
            value.month(),
            value.day()
        ))
    }

    pub fn parse(value: &str) -> Result<Self, ManifestValidationError> {
        parse_source_date(value)?;
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_source_date(&self) -> Result<SourceDate, ManifestValidationError> {
        parse_source_date(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestSource {
    pub logical_path: String,
    pub prefix: CmFilePrefix,
    pub product: CmProduct,
    pub record_kind: CmRecordKind,
    pub source_date: ManifestSourceDate,
    pub stream: Option<u8>,
    pub layout_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompressedIntegrity {
    pub byte_count: u64,
    pub md5: Md5Digest,
    pub sha256: Sha256Digest,
}

impl CompressedIntegrity {
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            byte_count: bytes.len() as u64,
            md5: md5_digest(bytes),
            sha256: sha256_digest(bytes),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerDeclaration {
    pub byte_count: u64,
    pub md5: Md5Digest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerVerificationStatus {
    NotProvided,
    Verified,
    SizeMismatch,
    Md5Mismatch,
    SizeAndMd5Mismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerEvidence {
    pub status: TriggerVerificationStatus,
    pub declared_byte_count: Option<u64>,
    pub declared_md5: Option<Md5Digest>,
}

impl TriggerEvidence {
    #[must_use]
    pub fn from_declaration(
        compressed: CompressedIntegrity,
        declaration: Option<TriggerDeclaration>,
    ) -> Self {
        match declaration {
            None => Self {
                status: TriggerVerificationStatus::NotProvided,
                declared_byte_count: None,
                declared_md5: None,
            },
            Some(declaration) => Self {
                status: trigger_status(compressed, declaration),
                declared_byte_count: Some(declaration.byte_count),
                declared_md5: Some(declaration.md5),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordDelimiter {
    Lf,
}

impl RecordDelimiter {
    const fn byte_count(self) -> u64 {
        match self {
            Self::Lf => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadIntegrity {
    pub byte_count: u64,
    pub sha256: Sha256Digest,
    pub payload_width: u32,
    pub delimiter: RecordDelimiter,
    pub framed_records: u64,
    pub parsed_records: u64,
    pub quarantined_records: u64,
    pub residue_bytes: u64,
    pub raw_jiffy_minimum: Option<u64>,
    pub raw_jiffy_maximum: Option<u64>,
}

impl PayloadIntegrity {
    #[must_use]
    pub fn from_bytes(
        bytes: &[u8],
        payload_width: u32,
        framed_records: u64,
        parsed_records: u64,
        quarantined_records: u64,
        residue_bytes: u64,
        raw_jiffy_bounds: Option<(u64, u64)>,
    ) -> Self {
        Self {
            byte_count: bytes.len() as u64,
            sha256: sha256_digest(bytes),
            payload_width,
            delimiter: RecordDelimiter::Lf,
            framed_records,
            parsed_records,
            quarantined_records,
            residue_bytes,
            raw_jiffy_minimum: raw_jiffy_bounds.map(|(minimum, _)| minimum),
            raw_jiffy_maximum: raw_jiffy_bounds.map(|(_, maximum)| maximum),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessingIdentity {
    pub configuration_id: String,
    pub code_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestV1 {
    pub schema_id: String,
    pub schema_major: u32,
    pub source: ManifestSource,
    pub compressed: CompressedIntegrity,
    pub trigger: TriggerEvidence,
    pub payload: PayloadIntegrity,
    pub processing: ProcessingIdentity,
}

impl ManifestV1 {
    #[must_use]
    pub fn new(
        source: ManifestSource,
        compressed: CompressedIntegrity,
        trigger: TriggerEvidence,
        payload: PayloadIntegrity,
        processing: ProcessingIdentity,
    ) -> Self {
        Self {
            schema_id: MANIFEST_SCHEMA_ID.to_owned(),
            schema_major: MANIFEST_SCHEMA_MAJOR,
            source,
            compressed,
            trigger,
            payload,
            processing,
        }
    }

    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_id != MANIFEST_SCHEMA_ID {
            return Err(ManifestValidationError::UnknownSchemaId);
        }
        if self.schema_major != MANIFEST_SCHEMA_MAJOR {
            return Err(ManifestValidationError::UnsupportedSchemaMajor {
                actual: self.schema_major,
            });
        }
        validate_logical_path(&self.source.logical_path)?;
        self.source.source_date.to_source_date()?;
        if matches!(self.source.stream, Some(0 | 100..=u8::MAX)) {
            return Err(ManifestValidationError::InvalidStream);
        }
        if self.source.layout_id.is_empty() {
            return Err(ManifestValidationError::EmptyLayoutId);
        }
        if self.processing.configuration_id.is_empty() {
            return Err(ManifestValidationError::EmptyConfigurationId);
        }
        if self.processing.code_identity.is_empty() {
            return Err(ManifestValidationError::EmptyCodeIdentity);
        }
        validate_trigger(self.compressed, self.trigger)?;
        validate_payload(self.payload)?;
        Ok(())
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, ManifestJsonError> {
        self.validate().map_err(ManifestJsonError::Validation)?;
        let mut bytes =
            serde_json::to_vec(self).map_err(|error| ManifestJsonError::Json(error.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, ManifestJsonError> {
        let manifest: Self = serde_json::from_slice(bytes)
            .map_err(|error| ManifestJsonError::Json(error.to_string()))?;
        manifest.validate().map_err(ManifestJsonError::Validation)?;
        Ok(manifest)
    }

    #[must_use]
    pub fn record_identity(&self, zero_based_record_index: u64) -> Option<RecordIdentity> {
        (zero_based_record_index < self.payload.framed_records).then(|| {
            RecordIdentity::new(
                self.payload.sha256,
                self.source.record_kind,
                zero_based_record_index,
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestJsonError {
    Json(String),
    Validation(ManifestValidationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestValidationError {
    UnknownSchemaId,
    UnsupportedSchemaMajor {
        actual: u32,
    },
    InvalidLogicalPath,
    InvalidSourceDate,
    InvalidSourceDateValue(SourceDateError),
    InvalidStream,
    EmptyLayoutId,
    EmptyConfigurationId,
    EmptyCodeIdentity,
    TriggerDeclarationShape,
    TriggerStatusMismatch {
        expected: TriggerVerificationStatus,
        actual: TriggerVerificationStatus,
    },
    PayloadWidthZero,
    RecordCountOverflow,
    RecordCountMismatch,
    PayloadByteCountOverflow,
    PayloadByteCountMismatch,
    JiffyBoundsPresenceMismatch,
    JiffyBoundsWithoutParsedRecords,
    MissingJiffyBounds,
    ReversedJiffyBounds,
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid manifest v1: {self:?}")
    }
}

impl std::error::Error for ManifestValidationError {}

impl fmt::Display for ManifestJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid manifest JSON: {error}"),
            Self::Validation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ManifestJsonError {}

fn parse_source_date(value: &str) -> Result<SourceDate, ManifestValidationError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..].iter().all(u8::is_ascii_digit)
    {
        return Err(ManifestValidationError::InvalidSourceDate);
    }

    let year = value[..4]
        .parse::<u16>()
        .map_err(|_| ManifestValidationError::InvalidSourceDate)?;
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| ManifestValidationError::InvalidSourceDate)?;
    let day = value[8..]
        .parse::<u8>()
        .map_err(|_| ManifestValidationError::InvalidSourceDate)?;
    SourceDate::new(year, month, day).map_err(ManifestValidationError::InvalidSourceDateValue)
}

fn validate_logical_path(value: &str) -> Result<(), ManifestValidationError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(ManifestValidationError::InvalidLogicalPath);
    }
    Ok(())
}

fn trigger_status(
    compressed: CompressedIntegrity,
    declaration: TriggerDeclaration,
) -> TriggerVerificationStatus {
    match (
        compressed.byte_count == declaration.byte_count,
        compressed.md5 == declaration.md5,
    ) {
        (true, true) => TriggerVerificationStatus::Verified,
        (false, true) => TriggerVerificationStatus::SizeMismatch,
        (true, false) => TriggerVerificationStatus::Md5Mismatch,
        (false, false) => TriggerVerificationStatus::SizeAndMd5Mismatch,
    }
}

fn validate_trigger(
    compressed: CompressedIntegrity,
    trigger: TriggerEvidence,
) -> Result<(), ManifestValidationError> {
    let declaration = match (trigger.declared_byte_count, trigger.declared_md5) {
        (None, None) => None,
        (Some(byte_count), Some(md5)) => Some(TriggerDeclaration { byte_count, md5 }),
        _ => return Err(ManifestValidationError::TriggerDeclarationShape),
    };
    let expected = declaration.map_or(TriggerVerificationStatus::NotProvided, |declaration| {
        trigger_status(compressed, declaration)
    });
    if trigger.status != expected {
        return Err(ManifestValidationError::TriggerStatusMismatch {
            expected,
            actual: trigger.status,
        });
    }
    Ok(())
}

fn validate_payload(payload: PayloadIntegrity) -> Result<(), ManifestValidationError> {
    if payload.payload_width == 0 {
        return Err(ManifestValidationError::PayloadWidthZero);
    }
    let accounted_records = payload
        .parsed_records
        .checked_add(payload.quarantined_records)
        .ok_or(ManifestValidationError::RecordCountOverflow)?;
    if accounted_records != payload.framed_records {
        return Err(ManifestValidationError::RecordCountMismatch);
    }
    let framed_width = u64::from(payload.payload_width)
        .checked_add(payload.delimiter.byte_count())
        .ok_or(ManifestValidationError::PayloadByteCountOverflow)?;
    let accounted_bytes = payload
        .framed_records
        .checked_mul(framed_width)
        .and_then(|framed| framed.checked_add(payload.residue_bytes))
        .ok_or(ManifestValidationError::PayloadByteCountOverflow)?;
    if accounted_bytes != payload.byte_count {
        return Err(ManifestValidationError::PayloadByteCountMismatch);
    }

    match (payload.raw_jiffy_minimum, payload.raw_jiffy_maximum) {
        (None, None) if payload.parsed_records == 0 => Ok(()),
        (None, None) => Err(ManifestValidationError::MissingJiffyBounds),
        (Some(_), Some(_)) if payload.parsed_records == 0 => {
            Err(ManifestValidationError::JiffyBoundsWithoutParsedRecords)
        }
        (Some(minimum), Some(maximum)) if minimum <= maximum => Ok(()),
        (Some(_), Some(_)) => Err(ManifestValidationError::ReversedJiffyBounds),
        _ => Err(ManifestValidationError::JiffyBoundsPresenceMismatch),
    }
}
