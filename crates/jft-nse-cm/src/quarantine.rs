//! Lossless quarantine records for rejected or ambiguous input.

use crate::layouts::FieldLayout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuarantineReason {
    WrongRecordLength { expected: usize, actual: usize },
    NonAscii,
    InvalidUnsignedInteger,
    ZeroNotAllowed,
    UnknownValue { expected: &'static str },
}

/// A rejected record with its original bytes and exact failure location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinedRecord {
    pub raw: Vec<u8>,
    pub field: Option<FieldLayout>,
    pub reason: QuarantineReason,
}

impl QuarantinedRecord {
    #[must_use]
    pub fn whole_record(raw: &[u8], reason: QuarantineReason) -> Self {
        Self {
            raw: raw.to_vec(),
            field: None,
            reason,
        }
    }

    #[must_use]
    pub fn field(raw: &[u8], field: FieldLayout, reason: QuarantineReason) -> Self {
        Self {
            raw: raw.to_vec(),
            field: Some(field),
            reason,
        }
    }
}
