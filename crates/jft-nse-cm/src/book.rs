//! Deterministic order- and price-level book state.

use std::collections::BTreeMap;

use crate::events::{MarketSegment, Session, Side};
use crate::lifecycle::{OrderKey, OrderLifecycle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BookKey {
    pub session: Session,
    pub segment: MarketSegment,
    pub symbol_raw: [u8; 10],
    pub series_raw: [u8; 2],
}

impl From<OrderKey> for BookKey {
    fn from(key: OrderKey) -> Self {
        Self {
            session: key.session,
            segment: key.segment,
            symbol_raw: key.symbol_raw,
            series_raw: key.series_raw,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PriceLevel {
    pub order_count: u64,
    pub total_quantity: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PriceBook {
    bids: BTreeMap<u64, PriceLevel>,
    asks: BTreeMap<u64, PriceLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopOfBook {
    Empty,
    BidOnly {
        price_paise: u64,
        level: PriceLevel,
    },
    AskOnly {
        price_paise: u64,
        level: PriceLevel,
    },
    Normal {
        bid_price_paise: u64,
        bid: PriceLevel,
        ask_price_paise: u64,
        ask: PriceLevel,
        spread_paise: u64,
    },
    Locked {
        price_paise: u64,
        bid: PriceLevel,
        ask: PriceLevel,
    },
    Crossed {
        bid_price_paise: u64,
        bid: PriceLevel,
        ask_price_paise: u64,
        ask: PriceLevel,
        cross_paise: u64,
    },
}

impl PriceBook {
    pub fn bids(&self) -> impl DoubleEndedIterator<Item = (u64, PriceLevel)> + '_ {
        self.bids.iter().map(|(price, level)| (*price, *level))
    }

    pub fn asks(&self) -> impl DoubleEndedIterator<Item = (u64, PriceLevel)> + '_ {
        self.asks.iter().map(|(price, level)| (*price, *level))
    }

    #[must_use]
    pub fn best_bid(&self) -> Option<(u64, PriceLevel)> {
        self.bids().next_back()
    }

    #[must_use]
    pub fn best_ask(&self) -> Option<(u64, PriceLevel)> {
        self.asks().next()
    }

    #[must_use]
    pub fn top(&self) -> TopOfBook {
        match (self.best_bid(), self.best_ask()) {
            (None, None) => TopOfBook::Empty,
            (Some((price_paise, level)), None) => TopOfBook::BidOnly { price_paise, level },
            (None, Some((price_paise, level))) => TopOfBook::AskOnly { price_paise, level },
            (Some((bid_price_paise, bid)), Some((ask_price_paise, ask))) => {
                match bid_price_paise.cmp(&ask_price_paise) {
                    std::cmp::Ordering::Less => TopOfBook::Normal {
                        bid_price_paise,
                        bid,
                        ask_price_paise,
                        ask,
                        spread_paise: ask_price_paise - bid_price_paise,
                    },
                    std::cmp::Ordering::Equal => TopOfBook::Locked {
                        price_paise: bid_price_paise,
                        bid,
                        ask,
                    },
                    std::cmp::Ordering::Greater => TopOfBook::Crossed {
                        bid_price_paise,
                        bid,
                        ask_price_paise,
                        ask,
                        cross_paise: bid_price_paise - ask_price_paise,
                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookProjectionError {
    OrderCountOverflow {
        book: BookKey,
        side: Side,
        price_paise: u64,
    },
    QuantityOverflow {
        book: BookKey,
        side: Side,
        price_paise: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrderBookProjection {
    books: BTreeMap<BookKey, PriceBook>,
}

impl OrderBookProjection {
    pub fn from_lifecycle(lifecycle: &OrderLifecycle) -> Result<Self, BookProjectionError> {
        let mut projection = Self::default();
        for (order_key, order) in lifecycle.active_orders() {
            let book_key = BookKey::from(*order_key);
            let book = projection.books.entry(book_key).or_default();
            let levels = match order.side {
                Side::Buy => &mut book.bids,
                Side::Sell => &mut book.asks,
            };
            let level = levels.entry(order.price_paise).or_default();
            level.order_count = level.order_count.checked_add(1).ok_or(
                BookProjectionError::OrderCountOverflow {
                    book: book_key,
                    side: order.side,
                    price_paise: order.price_paise,
                },
            )?;
            level.total_quantity = level.total_quantity.checked_add(order.quantity).ok_or(
                BookProjectionError::QuantityOverflow {
                    book: book_key,
                    side: order.side,
                    price_paise: order.price_paise,
                },
            )?;
        }
        Ok(projection)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.books.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    #[must_use]
    pub fn get(&self, key: &BookKey) -> Option<&PriceBook> {
        self.books.get(key)
    }

    pub fn books(&self) -> impl Iterator<Item = (&BookKey, &PriceBook)> {
        self.books.iter()
    }
}
