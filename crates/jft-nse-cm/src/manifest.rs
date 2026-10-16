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
