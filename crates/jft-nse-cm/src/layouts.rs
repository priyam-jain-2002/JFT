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
        CURRENT_FULL_TRADE_RECORD_LEN, FieldLayout, LEGACY_FULL_ORDER_FIELDS,
        LEGACY_FULL_ORDER_RECORD_LEN, LEGACY_FULL_TRADE_100_FIELDS,
        LEGACY_FULL_TRADE_100_RECORD_LEN, LEGACY_FULL_TRADE_101_FIELDS,
        LEGACY_FULL_TRADE_101_RECORD_LEN, TRIM_ORDER_FIELDS, TRIM_ORDER_RECORD_LEN,
        TRIM_TRADE_FIELDS, TRIM_TRADE_RECORD_LEN,
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
}
