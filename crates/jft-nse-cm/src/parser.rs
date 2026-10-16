//! Byte-accounting fixed-width parser entry points.

use crate::events::{
    MarketSegment, OrderActivity, OrderEvent, ParsedRecord, Session, Side, TradeEvent,
};
use crate::framing::{FixedWidthLfRecords, FramingError};
use crate::jiffy::ExchangeJiffy;
use crate::layouts::{
    FieldLayout, TRIM_ORDER_FIELDS, TRIM_ORDER_RECORD_LEN, TRIM_TRADE_FIELDS, TRIM_TRADE_RECORD_LEN,
};
use crate::quarantine::{QuarantineReason, QuarantinedRecord};

pub type ParsedTrimOrder = ParsedRecord<OrderEvent, TRIM_ORDER_RECORD_LEN>;
pub type ParsedTrimTrade = ParsedRecord<TradeEvent, TRIM_TRADE_RECORD_LEN>;

/// The parse result for one exactly framed source record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedParse<T, const N: usize> {
    pub record_index: usize,
    pub byte_offset: usize,
    pub result: Result<ParsedRecord<T, N>, QuarantinedRecord>,
}

/// Streaming, ordered, byte-addressable parse outcomes for one source.
///
/// Field-level parse failures stay inside `LocatedParse::result`, so iteration
/// can continue to later exactly framed records. A framing failure is the outer
/// error and terminates the underlying framing iterator without guessing.
pub struct ParsedRecords<'a, T, const N: usize> {
    framed: FixedWidthLfRecords<'a, N>,
    parse_record: fn(&[u8]) -> Result<ParsedRecord<T, N>, QuarantinedRecord>,
}

impl<'a, T, const N: usize> Iterator for ParsedRecords<'a, T, N> {
    type Item = Result<LocatedParse<T, N>, FramingError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.framed.next().map(|framed| {
            framed.map(|framed| LocatedParse {
                record_index: framed.record_index,
                byte_offset: framed.byte_offset,
                result: (self.parse_record)(framed.raw),
            })
        })
    }
}

pub type ParsedTrimOrderSource<'a> = ParsedRecords<'a, OrderEvent, TRIM_ORDER_RECORD_LEN>;
pub type ParsedTrimTradeSource<'a> = ParsedRecords<'a, TradeEvent, TRIM_TRADE_RECORD_LEN>;

#[must_use]
pub fn parse_trim_order_source(source: &[u8]) -> ParsedTrimOrderSource<'_> {
    parse_source(source, parse_trim_order)
}

#[must_use]
pub fn parse_trim_trade_source(source: &[u8]) -> ParsedTrimTradeSource<'_> {
    parse_source(source, parse_trim_trade)
}

pub fn parse_trim_order(record: &[u8]) -> Result<ParsedTrimOrder, QuarantinedRecord> {
    let raw = exact_record::<TRIM_ORDER_RECORD_LEN>(record)?;
    let session = parse_session(record, TRIM_ORDER_FIELDS[0])?;
    let segment = parse_cash_segment(record, TRIM_ORDER_FIELDS[1])?;
    let order_number = parse_unsigned(record, TRIM_ORDER_FIELDS[2], true)?;
    let exchange_jiffy = ExchangeJiffy::new(parse_unsigned(record, TRIM_ORDER_FIELDS[3], true)?);
    let side = parse_side(record, TRIM_ORDER_FIELDS[4])?;
    let activity = parse_activity(record, TRIM_ORDER_FIELDS[5])?;
    let (symbol_raw, symbol) = parse_symbol(record, TRIM_ORDER_FIELDS[6])?;
    let (series_raw, series) = parse_series(record, TRIM_ORDER_FIELDS[7])?;
    let quantity = parse_unsigned(record, TRIM_ORDER_FIELDS[8], false)?;
    let price_paise = parse_unsigned(record, TRIM_ORDER_FIELDS[9], false)?;

    Ok(ParsedRecord {
        event: OrderEvent {
            session,
            segment,
            order_number,
            exchange_jiffy,
            side,
            activity,
            symbol_raw,
            symbol,
            series_raw,
            series,
            quantity,
            price_paise,
        },
        raw,
    })
}

pub fn parse_trim_trade(record: &[u8]) -> Result<ParsedTrimTrade, QuarantinedRecord> {
    let raw = exact_record::<TRIM_TRADE_RECORD_LEN>(record)?;
    let session = parse_session(record, TRIM_TRADE_FIELDS[0])?;
    let segment = parse_cash_segment(record, TRIM_TRADE_FIELDS[1])?;
    let exchange_jiffy = ExchangeJiffy::new(parse_unsigned(record, TRIM_TRADE_FIELDS[2], true)?);
    let (symbol_raw, symbol) = parse_symbol(record, TRIM_TRADE_FIELDS[3])?;
    let (series_raw, series) = parse_series(record, TRIM_TRADE_FIELDS[4])?;
    let price_paise = parse_unsigned(record, TRIM_TRADE_FIELDS[5], false)?;
    let quantity = parse_unsigned(record, TRIM_TRADE_FIELDS[6], false)?;
    let buy_order_number = parse_unsigned(record, TRIM_TRADE_FIELDS[7], true)?;
    let sell_order_number = parse_unsigned(record, TRIM_TRADE_FIELDS[8], true)?;

    Ok(ParsedRecord {
        event: TradeEvent {
            session,
            segment,
            exchange_jiffy,
            symbol_raw,
            symbol,
            series_raw,
            series,
            price_paise,
            quantity,
            buy_order_number,
            sell_order_number,
        },
        raw,
    })
}

fn parse_source<T, const N: usize>(
    source: &[u8],
    parse_record: fn(&[u8]) -> Result<ParsedRecord<T, N>, QuarantinedRecord>,
) -> ParsedRecords<'_, T, N> {
    ParsedRecords {
        framed: FixedWidthLfRecords::new(source),
        parse_record,
    }
}

fn exact_record<const N: usize>(record: &[u8]) -> Result<[u8; N], QuarantinedRecord> {
    record.try_into().map_err(|_| {
        QuarantinedRecord::whole_record(
            record,
            QuarantineReason::WrongRecordLength {
                expected: N,
                actual: record.len(),
            },
        )
    })
}

fn bytes(record: &[u8], field: FieldLayout) -> &[u8] {
    &record[field.offset..field.end()]
}

fn parse_unsigned(
    record: &[u8],
    field: FieldLayout,
    allow_zero: bool,
) -> Result<u64, QuarantinedRecord> {
    let value_bytes = bytes(record, field);
    if !value_bytes.iter().all(u8::is_ascii_digit) {
        return Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::InvalidUnsignedInteger,
        ));
    }

    let mut value = 0_u64;
    for digit in value_bytes {
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(digit - b'0')))
            .ok_or_else(|| {
                QuarantinedRecord::field(record, field, QuarantineReason::InvalidUnsignedInteger)
            })?;
    }

    if !allow_zero && value == 0 {
        return Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::ZeroNotAllowed,
        ));
    }
    Ok(value)
}

fn parse_session(record: &[u8], field: FieldLayout) -> Result<Session, QuarantinedRecord> {
    match bytes(record, field) {
        b"PO" => Ok(Session::PreOpen),
        b"RM" => Ok(Session::RegularMarket),
        b"CA" => Ok(Session::ClosingAuction),
        _ => Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::UnknownValue {
                expected: "PO, RM, or CA",
            },
        )),
    }
}

fn parse_cash_segment(
    record: &[u8],
    field: FieldLayout,
) -> Result<MarketSegment, QuarantinedRecord> {
    if bytes(record, field) == b"CASH" {
        Ok(MarketSegment::Cash)
    } else {
        Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::UnknownValue { expected: "CASH" },
        ))
    }
}

fn parse_side(record: &[u8], field: FieldLayout) -> Result<Side, QuarantinedRecord> {
    match bytes(record, field) {
        b"B" => Ok(Side::Buy),
        b"S" => Ok(Side::Sell),
        _ => Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::UnknownValue { expected: "B or S" },
        )),
    }
}

fn parse_activity(record: &[u8], field: FieldLayout) -> Result<OrderActivity, QuarantinedRecord> {
    match bytes(record, field) {
        b"1" => Ok(OrderActivity::Entry),
        b"3" => Ok(OrderActivity::Cancel),
        b"4" => Ok(OrderActivity::Modify),
        _ => Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::UnknownValue {
                expected: "1, 3, or 4",
            },
        )),
    }
}

fn parse_symbol(
    record: &[u8],
    field: FieldLayout,
) -> Result<([u8; 10], String), QuarantinedRecord> {
    let symbol_bytes = bytes(record, field);
    if !symbol_bytes.is_ascii() {
        return Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::NonAscii,
        ));
    }

    let raw = exact_field::<10>(record, field);
    let normalized = String::from_utf8(raw.to_vec())
        .expect("ASCII was checked")
        .trim_start_matches(' ')
        .to_owned();
    Ok((raw, normalized))
}

fn parse_series(record: &[u8], field: FieldLayout) -> Result<([u8; 2], String), QuarantinedRecord> {
    let series_bytes = bytes(record, field);
    if !series_bytes.is_ascii() {
        return Err(QuarantinedRecord::field(
            record,
            field,
            QuarantineReason::NonAscii,
        ));
    }

    let raw = exact_field::<2>(record, field);
    let normalized = String::from_utf8(raw.to_vec()).expect("ASCII was checked");
    Ok((raw, normalized))
}

fn exact_field<const N: usize>(record: &[u8], field: FieldLayout) -> [u8; N] {
    bytes(record, field)
        .try_into()
        .expect("field layout width must match destination width")
}
