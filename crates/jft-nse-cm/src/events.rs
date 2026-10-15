//! Versioned canonical order and trade events.

use crate::jiffy::ExchangeJiffy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    PreOpen,
    RegularMarket,
    ClosingAuction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketSegment {
    Cash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderActivity {
    Entry,
    Cancel,
    Modify,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderEvent {
    pub session: Session,
    pub segment: MarketSegment,
    pub order_number: u64,
    pub exchange_jiffy: ExchangeJiffy,
    pub side: Side,
    pub activity: OrderActivity,
    pub symbol_raw: [u8; 10],
    pub symbol: String,
    pub series_raw: [u8; 2],
    pub series: String,
    pub quantity: u64,
    pub price_paise: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeEvent {
    pub session: Session,
    pub segment: MarketSegment,
    pub exchange_jiffy: ExchangeJiffy,
    pub symbol_raw: [u8; 10],
    pub symbol: String,
    pub series_raw: [u8; 2],
    pub series: String,
    pub price_paise: u64,
    pub quantity: u64,
    pub buy_order_number: u64,
    pub sell_order_number: u64,
}

/// A parsed event paired with the exact fixed-width bytes that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRecord<T, const N: usize> {
    pub event: T,
    pub raw: [u8; N],
}
