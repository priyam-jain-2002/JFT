//! Date- and product-version-aware fixed-width layouts.

/// One zero-based, fixed-width byte field in an exchange record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldLayout {
    pub name: &'static str,
    pub offset: usize,
    pub width: usize,
}

impl FieldLayout {
    #[must_use]
    pub const fn end(self) -> usize {
        self.offset + self.width
    }
}

pub const TRIM_ORDER_RECORD_LEN: usize = 68;
pub const TRIM_TRADE_RECORD_LEN: usize = 82;
pub const LEGACY_FULL_ORDER_RECORD_LEN: usize = 87;
pub const LEGACY_FULL_TRADE_100_RECORD_LEN: usize = 100;
pub const LEGACY_FULL_TRADE_101_RECORD_LEN: usize = 101;
pub const CURRENT_FULL_ORDER_RECORD_LEN: usize = 91;
pub const CURRENT_FULL_TRADE_RECORD_LEN: usize = 103;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceDate {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceDateError {
    YearZero,
    InvalidMonth {
        month: u8,
    },
    InvalidDay {
        year: u16,
        month: u8,
        day: u8,
        maximum: u8,
    },
}

impl SourceDate {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, SourceDateError> {
        if year == 0 {
            return Err(SourceDateError::YearZero);
        }
        if !(1..=12).contains(&month) {
            return Err(SourceDateError::InvalidMonth { month });
        }

        let maximum = days_in_month(year, month);
        if day == 0 || day > maximum {
            return Err(SourceDateError::InvalidDay {
                year,
                month,
                day,
                maximum,
            });
        }

        Ok(Self { year, month, day })
    }

    const fn documented(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullOrderLayoutVersion {
    Bytes87,
    Bytes91,
}

impl FullOrderLayoutVersion {
    #[must_use]
    pub const fn record_len(self) -> usize {
        match self {
            Self::Bytes87 => LEGACY_FULL_ORDER_RECORD_LEN,
            Self::Bytes91 => CURRENT_FULL_ORDER_RECORD_LEN,
        }
    }

    #[must_use]
    pub const fn fields(self) -> &'static [FieldLayout] {
        match self {
            Self::Bytes87 => &LEGACY_FULL_ORDER_FIELDS,
            Self::Bytes91 => &CURRENT_FULL_ORDER_FIELDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullTradeLayoutVersion {
    Bytes100,
    Bytes101,
    Bytes103,
}

impl FullTradeLayoutVersion {
    #[must_use]
    pub const fn record_len(self) -> usize {
        match self {
            Self::Bytes100 => LEGACY_FULL_TRADE_100_RECORD_LEN,
            Self::Bytes101 => LEGACY_FULL_TRADE_101_RECORD_LEN,
            Self::Bytes103 => CURRENT_FULL_TRADE_RECORD_LEN,
        }
    }

    #[must_use]
    pub const fn fields(self) -> &'static [FieldLayout] {
        match self {
            Self::Bytes100 => &LEGACY_FULL_TRADE_100_FIELDS,
            Self::Bytes101 => &LEGACY_FULL_TRADE_101_FIELDS,
            Self::Bytes103 => &CURRENT_FULL_TRADE_FIELDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullCmLayoutVersion {
    pub order: FullOrderLayoutVersion,
    pub trade: FullTradeLayoutVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimCmLayoutVersion {
    Bytes68And82,
}

impl TrimCmLayoutVersion {
    #[must_use]
    pub const fn order_record_len(self) -> usize {
        match self {
            Self::Bytes68And82 => TRIM_ORDER_RECORD_LEN,
        }
    }

    #[must_use]
    pub const fn order_fields(self) -> &'static [FieldLayout] {
        match self {
            Self::Bytes68And82 => &TRIM_ORDER_FIELDS,
        }
    }

    #[must_use]
    pub const fn trade_record_len(self) -> usize {
        match self {
            Self::Bytes68And82 => TRIM_TRADE_RECORD_LEN,
        }
    }

    #[must_use]
    pub const fn trade_fields(self) -> &'static [FieldLayout] {
        match self {
            Self::Bytes68And82 => &TRIM_TRADE_FIELDS,
        }
    }
}

const TRADE_NUMBER_EXPANSION: SourceDate = SourceDate::documented(2024, 7, 1);
const ONE_DAY_VOLUME_EXCEPTION: SourceDate = SourceDate::documented(2025, 3, 28);
const PERMANENT_VOLUME_EXPANSION: SourceDate = SourceDate::documented(2025, 6, 16);
const TRIM_INTRODUCTION: SourceDate = SourceDate::documented(2026, 5, 18);

#[must_use]
pub fn full_cm_layout_for_date(date: SourceDate) -> FullCmLayoutVersion {
    if date == ONE_DAY_VOLUME_EXCEPTION || date >= PERMANENT_VOLUME_EXPANSION {
        FullCmLayoutVersion {
            order: FullOrderLayoutVersion::Bytes91,
            trade: FullTradeLayoutVersion::Bytes103,
        }
    } else if date >= TRADE_NUMBER_EXPANSION {
        FullCmLayoutVersion {
            order: FullOrderLayoutVersion::Bytes87,
            trade: FullTradeLayoutVersion::Bytes101,
        }
    } else {
        FullCmLayoutVersion {
            order: FullOrderLayoutVersion::Bytes87,
            trade: FullTradeLayoutVersion::Bytes100,
        }
    }
}

#[must_use]
pub fn trim_cm_layout_for_date(date: SourceDate) -> Option<TrimCmLayoutVersion> {
    (date >= TRIM_INTRODUCTION).then_some(TrimCmLayoutVersion::Bytes68And82)
}

pub const TRIM_ORDER_FIELDS: [FieldLayout; 10] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "order_number",
        offset: 6,
        width: 16,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 22,
        width: 14,
    },
    FieldLayout {
        name: "side",
        offset: 36,
        width: 1,
    },
    FieldLayout {
        name: "activity",
        offset: 37,
        width: 1,
    },
    FieldLayout {
        name: "symbol",
        offset: 38,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 48,
        width: 2,
    },
    FieldLayout {
        name: "quantity",
        offset: 50,
        width: 10,
    },
    FieldLayout {
        name: "price_paise",
        offset: 60,
        width: 8,
    },
];

pub const TRIM_TRADE_FIELDS: [FieldLayout; 9] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 6,
        width: 14,
    },
    FieldLayout {
        name: "symbol",
        offset: 20,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 30,
        width: 2,
    },
    FieldLayout {
        name: "price_paise",
        offset: 32,
        width: 8,
    },
    FieldLayout {
        name: "quantity",
        offset: 40,
        width: 10,
    },
    FieldLayout {
        name: "buy_order_number",
        offset: 50,
        width: 16,
    },
    FieldLayout {
        name: "sell_order_number",
        offset: 66,
        width: 16,
    },
];

pub const CURRENT_FULL_ORDER_FIELDS: [FieldLayout; 17] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "order_number",
        offset: 6,
        width: 16,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 22,
        width: 14,
    },
    FieldLayout {
        name: "side",
        offset: 36,
        width: 1,
    },
    FieldLayout {
        name: "activity",
        offset: 37,
        width: 1,
    },
    FieldLayout {
        name: "symbol",
        offset: 38,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 48,
        width: 2,
    },
    FieldLayout {
        name: "disclosed_quantity",
        offset: 50,
        width: 10,
    },
    FieldLayout {
        name: "original_quantity",
        offset: 60,
        width: 10,
    },
    FieldLayout {
        name: "limit_price_paise",
        offset: 70,
        width: 8,
    },
    FieldLayout {
        name: "trigger_price_paise",
        offset: 78,
        width: 8,
    },
    FieldLayout {
        name: "market_order_flag",
        offset: 86,
        width: 1,
    },
    FieldLayout {
        name: "stop_loss_flag",
        offset: 87,
        width: 1,
    },
    FieldLayout {
        name: "ioc_flag",
        offset: 88,
        width: 1,
    },
    FieldLayout {
        name: "algo_indicator",
        offset: 89,
        width: 1,
    },
    FieldLayout {
        name: "client_identity",
        offset: 90,
        width: 1,
    },
];

pub const CURRENT_FULL_TRADE_FIELDS: [FieldLayout; 14] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "trade_number",
        offset: 6,
        width: 17,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 23,
        width: 14,
    },
    FieldLayout {
        name: "symbol",
        offset: 37,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 47,
        width: 2,
    },
    FieldLayout {
        name: "price_paise",
        offset: 49,
        width: 8,
    },
    FieldLayout {
        name: "quantity",
        offset: 57,
        width: 10,
    },
    FieldLayout {
        name: "buy_order_number",
        offset: 67,
        width: 16,
    },
    FieldLayout {
        name: "buy_algo_indicator",
        offset: 83,
        width: 1,
    },
    FieldLayout {
        name: "buy_client_identity",
        offset: 84,
        width: 1,
    },
    FieldLayout {
        name: "sell_order_number",
        offset: 85,
        width: 16,
    },
    FieldLayout {
        name: "sell_algo_indicator",
        offset: 101,
        width: 1,
    },
    FieldLayout {
        name: "sell_client_identity",
        offset: 102,
        width: 1,
    },
];

/// Full CM Order layout before the one-day 2025-03-28 exception and the
/// permanent 2025-06-16 volume-width expansion.
pub const LEGACY_FULL_ORDER_FIELDS: [FieldLayout; 17] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "order_number",
        offset: 6,
        width: 16,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 22,
        width: 14,
    },
    FieldLayout {
        name: "side",
        offset: 36,
        width: 1,
    },
    FieldLayout {
        name: "activity",
        offset: 37,
        width: 1,
    },
    FieldLayout {
        name: "symbol",
        offset: 38,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 48,
        width: 2,
    },
    FieldLayout {
        name: "disclosed_quantity",
        offset: 50,
        width: 8,
    },
    FieldLayout {
        name: "original_quantity",
        offset: 58,
        width: 8,
    },
    FieldLayout {
        name: "limit_price_paise",
        offset: 66,
        width: 8,
    },
    FieldLayout {
        name: "trigger_price_paise",
        offset: 74,
        width: 8,
    },
    FieldLayout {
        name: "market_order_flag",
        offset: 82,
        width: 1,
    },
    FieldLayout {
        name: "stop_loss_flag",
        offset: 83,
        width: 1,
    },
    FieldLayout {
        name: "ioc_flag",
        offset: 84,
        width: 1,
    },
    FieldLayout {
        name: "algo_indicator",
        offset: 85,
        width: 1,
    },
    FieldLayout {
        name: "client_identity",
        offset: 86,
        width: 1,
    },
];

/// Full CM Trade layout before the 17-digit trade-number change on 2024-07-01.
pub const LEGACY_FULL_TRADE_100_FIELDS: [FieldLayout; 14] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "trade_number",
        offset: 6,
        width: 16,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 22,
        width: 14,
    },
    FieldLayout {
        name: "symbol",
        offset: 36,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 46,
        width: 2,
    },
    FieldLayout {
        name: "price_paise",
        offset: 48,
        width: 8,
    },
    FieldLayout {
        name: "quantity",
        offset: 56,
        width: 8,
    },
    FieldLayout {
        name: "buy_order_number",
        offset: 64,
        width: 16,
    },
    FieldLayout {
        name: "buy_algo_indicator",
        offset: 80,
        width: 1,
    },
    FieldLayout {
        name: "buy_client_identity",
        offset: 81,
        width: 1,
    },
    FieldLayout {
        name: "sell_order_number",
        offset: 82,
        width: 16,
    },
    FieldLayout {
        name: "sell_algo_indicator",
        offset: 98,
        width: 1,
    },
    FieldLayout {
        name: "sell_client_identity",
        offset: 99,
        width: 1,
    },
];

/// Full CM Trade layout from 2024-07-01 except the 2025-03-28 exception,
/// through the permanent quantity-width expansion on 2025-06-16.
pub const LEGACY_FULL_TRADE_101_FIELDS: [FieldLayout; 14] = [
    FieldLayout {
        name: "record_indicator",
        offset: 0,
        width: 2,
    },
    FieldLayout {
        name: "segment",
        offset: 2,
        width: 4,
    },
    FieldLayout {
        name: "trade_number",
        offset: 6,
        width: 17,
    },
    FieldLayout {
        name: "exchange_jiffy",
        offset: 23,
        width: 14,
    },
    FieldLayout {
        name: "symbol",
        offset: 37,
        width: 10,
    },
    FieldLayout {
        name: "series",
        offset: 47,
        width: 2,
    },
    FieldLayout {
        name: "price_paise",
        offset: 49,
        width: 8,
    },
    FieldLayout {
        name: "quantity",
        offset: 57,
        width: 8,
    },
    FieldLayout {
        name: "buy_order_number",
        offset: 65,
        width: 16,
    },
    FieldLayout {
        name: "buy_algo_indicator",
        offset: 81,
        width: 1,
    },
    FieldLayout {
        name: "buy_client_identity",
        offset: 82,
        width: 1,
    },
    FieldLayout {
        name: "sell_order_number",
        offset: 83,
        width: 16,
    },
    FieldLayout {
        name: "sell_algo_indicator",
        offset: 99,
        width: 1,
    },
    FieldLayout {
        name: "sell_client_identity",
        offset: 100,
        width: 1,
    },
];

#[cfg(test)]
mod tests {
    use super::{
        CURRENT_FULL_ORDER_FIELDS, CURRENT_FULL_ORDER_RECORD_LEN, CURRENT_FULL_TRADE_FIELDS,
        CURRENT_FULL_TRADE_RECORD_LEN, FieldLayout, FullOrderLayoutVersion, FullTradeLayoutVersion,
        LEGACY_FULL_ORDER_FIELDS, LEGACY_FULL_ORDER_RECORD_LEN, LEGACY_FULL_TRADE_100_FIELDS,
        LEGACY_FULL_TRADE_100_RECORD_LEN, LEGACY_FULL_TRADE_101_FIELDS,
        LEGACY_FULL_TRADE_101_RECORD_LEN, SourceDate, SourceDateError, TRIM_ORDER_FIELDS,
        TRIM_ORDER_RECORD_LEN, TRIM_TRADE_FIELDS, TRIM_TRADE_RECORD_LEN, TrimCmLayoutVersion,
        full_cm_layout_for_date, trim_cm_layout_for_date,
    };

    fn assert_contiguous(fields: &[FieldLayout], expected_len: usize) {
        let mut next_offset = 0;
        for field in fields {
            assert_eq!(
                field.offset, next_offset,
                "gap or overlap at {}",
                field.name
            );
            next_offset = field.end();
        }
        assert_eq!(next_offset, expected_len);
    }

    #[test]
    fn trim_order_layout_is_contiguous_and_68_bytes() {
        assert_contiguous(&TRIM_ORDER_FIELDS, TRIM_ORDER_RECORD_LEN);
    }

    #[test]
    fn trim_trade_layout_is_contiguous_and_82_bytes() {
        assert_contiguous(&TRIM_TRADE_FIELDS, TRIM_TRADE_RECORD_LEN);
    }

    #[test]
    fn current_full_order_layout_matches_the_documented_91_byte_map() {
        assert_contiguous(&CURRENT_FULL_ORDER_FIELDS, CURRENT_FULL_ORDER_RECORD_LEN);
        assert_eq!(
            CURRENT_FULL_ORDER_FIELDS.map(|field| (field.name, field.offset, field.width)),
            [
                ("record_indicator", 0, 2),
                ("segment", 2, 4),
                ("order_number", 6, 16),
                ("exchange_jiffy", 22, 14),
                ("side", 36, 1),
                ("activity", 37, 1),
                ("symbol", 38, 10),
                ("series", 48, 2),
                ("disclosed_quantity", 50, 10),
                ("original_quantity", 60, 10),
                ("limit_price_paise", 70, 8),
                ("trigger_price_paise", 78, 8),
                ("market_order_flag", 86, 1),
                ("stop_loss_flag", 87, 1),
                ("ioc_flag", 88, 1),
                ("algo_indicator", 89, 1),
                ("client_identity", 90, 1),
            ]
        );
    }

    #[test]
    fn current_full_trade_layout_matches_the_documented_103_byte_map() {
        assert_contiguous(&CURRENT_FULL_TRADE_FIELDS, CURRENT_FULL_TRADE_RECORD_LEN);
        assert_eq!(
            CURRENT_FULL_TRADE_FIELDS.map(|field| (field.name, field.offset, field.width)),
            [
                ("record_indicator", 0, 2),
                ("segment", 2, 4),
                ("trade_number", 6, 17),
                ("exchange_jiffy", 23, 14),
                ("symbol", 37, 10),
                ("series", 47, 2),
                ("price_paise", 49, 8),
                ("quantity", 57, 10),
                ("buy_order_number", 67, 16),
                ("buy_algo_indicator", 83, 1),
                ("buy_client_identity", 84, 1),
                ("sell_order_number", 85, 16),
                ("sell_algo_indicator", 101, 1),
                ("sell_client_identity", 102, 1),
            ]
        );
    }

    #[test]
    fn trim_and_current_full_orders_share_the_exact_core_prefix() {
        assert_eq!(&TRIM_ORDER_FIELDS[..8], &CURRENT_FULL_ORDER_FIELDS[..8]);
    }

    #[test]
    fn legacy_full_order_layout_matches_the_documented_87_byte_map() {
        assert_contiguous(&LEGACY_FULL_ORDER_FIELDS, LEGACY_FULL_ORDER_RECORD_LEN);
        assert_eq!(
            LEGACY_FULL_ORDER_FIELDS.map(|field| (field.name, field.offset, field.width)),
            [
                ("record_indicator", 0, 2),
                ("segment", 2, 4),
                ("order_number", 6, 16),
                ("exchange_jiffy", 22, 14),
                ("side", 36, 1),
                ("activity", 37, 1),
                ("symbol", 38, 10),
                ("series", 48, 2),
                ("disclosed_quantity", 50, 8),
                ("original_quantity", 58, 8),
                ("limit_price_paise", 66, 8),
                ("trigger_price_paise", 74, 8),
                ("market_order_flag", 82, 1),
                ("stop_loss_flag", 83, 1),
                ("ioc_flag", 84, 1),
                ("algo_indicator", 85, 1),
                ("client_identity", 86, 1),
            ]
        );
        assert_eq!(
            &LEGACY_FULL_ORDER_FIELDS[..8],
            &CURRENT_FULL_ORDER_FIELDS[..8]
        );
    }

    #[test]
    fn legacy_full_trade_layout_matches_the_documented_100_byte_map() {
        assert_contiguous(
            &LEGACY_FULL_TRADE_100_FIELDS,
            LEGACY_FULL_TRADE_100_RECORD_LEN,
        );
        assert_eq!(
            LEGACY_FULL_TRADE_100_FIELDS.map(|field| (field.name, field.offset, field.width)),
            [
                ("record_indicator", 0, 2),
                ("segment", 2, 4),
                ("trade_number", 6, 16),
                ("exchange_jiffy", 22, 14),
                ("symbol", 36, 10),
                ("series", 46, 2),
                ("price_paise", 48, 8),
                ("quantity", 56, 8),
                ("buy_order_number", 64, 16),
                ("buy_algo_indicator", 80, 1),
                ("buy_client_identity", 81, 1),
                ("sell_order_number", 82, 16),
                ("sell_algo_indicator", 98, 1),
                ("sell_client_identity", 99, 1),
            ]
        );
    }

    #[test]
    fn legacy_full_trade_layout_matches_the_documented_101_byte_map() {
        assert_contiguous(
            &LEGACY_FULL_TRADE_101_FIELDS,
            LEGACY_FULL_TRADE_101_RECORD_LEN,
        );
        assert_eq!(
            LEGACY_FULL_TRADE_101_FIELDS.map(|field| (field.name, field.offset, field.width)),
            [
                ("record_indicator", 0, 2),
                ("segment", 2, 4),
                ("trade_number", 6, 17),
                ("exchange_jiffy", 23, 14),
                ("symbol", 37, 10),
                ("series", 47, 2),
                ("price_paise", 49, 8),
                ("quantity", 57, 8),
                ("buy_order_number", 65, 16),
                ("buy_algo_indicator", 81, 1),
                ("buy_client_identity", 82, 1),
                ("sell_order_number", 83, 16),
                ("sell_algo_indicator", 99, 1),
                ("sell_client_identity", 100, 1),
            ]
        );
    }

    #[test]
    fn source_date_rejects_invalid_gregorian_dates() {
        assert_eq!(SourceDate::new(0, 1, 1), Err(SourceDateError::YearZero));
        assert_eq!(
            SourceDate::new(2025, 13, 1),
            Err(SourceDateError::InvalidMonth { month: 13 })
        );
        assert_eq!(
            SourceDate::new(2025, 2, 29),
            Err(SourceDateError::InvalidDay {
                year: 2025,
                month: 2,
                day: 29,
                maximum: 28,
            })
        );
        assert!(SourceDate::new(2024, 2, 29).is_ok());
        assert!(SourceDate::new(2000, 2, 29).is_ok());
        assert!(SourceDate::new(2100, 2, 29).is_err());
    }

    #[test]
    fn full_layout_dispatches_across_every_documented_boundary() {
        let cases = [
            (
                (2024, 6, 30),
                FullOrderLayoutVersion::Bytes87,
                FullTradeLayoutVersion::Bytes100,
            ),
            (
                (2024, 7, 1),
                FullOrderLayoutVersion::Bytes87,
                FullTradeLayoutVersion::Bytes101,
            ),
            (
                (2025, 3, 27),
                FullOrderLayoutVersion::Bytes87,
                FullTradeLayoutVersion::Bytes101,
            ),
            (
                (2025, 3, 28),
                FullOrderLayoutVersion::Bytes91,
                FullTradeLayoutVersion::Bytes103,
            ),
            (
                (2025, 3, 29),
                FullOrderLayoutVersion::Bytes87,
                FullTradeLayoutVersion::Bytes101,
            ),
            (
                (2025, 6, 15),
                FullOrderLayoutVersion::Bytes87,
                FullTradeLayoutVersion::Bytes101,
            ),
            (
                (2025, 6, 16),
                FullOrderLayoutVersion::Bytes91,
                FullTradeLayoutVersion::Bytes103,
            ),
        ];

        for ((year, month, day), order, trade) in cases {
            let selected = full_cm_layout_for_date(
                SourceDate::new(year, month, day).expect("valid test date"),
            );
            assert_eq!(selected.order, order);
            assert_eq!(selected.trade, trade);
        }
    }

    #[test]
    fn selected_full_versions_expose_the_matching_fields_and_lengths() {
        for order in [
            FullOrderLayoutVersion::Bytes87,
            FullOrderLayoutVersion::Bytes91,
        ] {
            assert_eq!(
                order.fields().last().expect("last field").end(),
                order.record_len()
            );
        }
        for trade in [
            FullTradeLayoutVersion::Bytes100,
            FullTradeLayoutVersion::Bytes101,
            FullTradeLayoutVersion::Bytes103,
        ] {
            assert_eq!(
                trade.fields().last().expect("last field").end(),
                trade.record_len()
            );
        }
    }

    #[test]
    fn trim_dispatch_starts_on_the_documented_introduction_date() {
        assert_eq!(
            trim_cm_layout_for_date(SourceDate::new(2026, 5, 17).expect("valid date")),
            None
        );
        let selected = trim_cm_layout_for_date(SourceDate::new(2026, 5, 18).expect("valid date"))
            .expect("Trim is documented from this date");
        assert_eq!(selected, TrimCmLayoutVersion::Bytes68And82);
        assert_eq!(selected.order_record_len(), TRIM_ORDER_RECORD_LEN);
        assert_eq!(selected.trade_record_len(), TRIM_TRADE_RECORD_LEN);
        assert_eq!(selected.order_fields(), TRIM_ORDER_FIELDS);
        assert_eq!(selected.trade_fields(), TRIM_TRADE_FIELDS);
    }
}
