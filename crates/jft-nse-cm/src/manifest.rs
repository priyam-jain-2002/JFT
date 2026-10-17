//! Immutable input discovery, integrity evidence, and lineage manifests.

use crate::layouts::{SourceDate, SourceDateError};

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
