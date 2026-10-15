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

#[cfg(test)]
mod tests {
    use super::{
        FieldLayout, TRIM_ORDER_FIELDS, TRIM_ORDER_RECORD_LEN, TRIM_TRADE_FIELDS,
        TRIM_TRADE_RECORD_LEN,
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
}
