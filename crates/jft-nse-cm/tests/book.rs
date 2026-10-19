use std::collections::BTreeMap;

use jft_nse_cm::book::{
    BookKey, BookProjectionError, OrderBookProjection, PriceBook, PriceLevel, TopOfBook,
};
use jft_nse_cm::events::{OrderActivity, OrderEvent, Session, Side};
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::TRIM_ORDER_RECORD_LEN;
use jft_nse_cm::lifecycle::OrderLifecycle;
use jft_nse_cm::parser::parse_trim_order;

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");

fn base_order() -> OrderEvent {
    assert_eq!(ENTRY_FIXTURE.len(), TRIM_ORDER_RECORD_LEN + 1);
    parse_trim_order(&ENTRY_FIXTURE[..TRIM_ORDER_RECORD_LEN])
        .expect("synthetic entry fixture")
        .event
}

fn insert(lifecycle: &mut OrderLifecycle, event: &OrderEvent) {
    lifecycle.apply(event).expect("valid lifecycle entry");
}

fn key(event: &OrderEvent) -> BookKey {
    BookKey {
        session: event.session,
        segment: event.segment,
        symbol_raw: event.symbol_raw,
        series_raw: event.series_raw,
    }
}

fn reference_top(bids: &BTreeMap<u64, PriceLevel>, asks: &BTreeMap<u64, PriceLevel>) -> TopOfBook {
    let best_bid = bids
        .iter()
        .next_back()
        .map(|(price, level)| (*price, *level));
    let best_ask = asks.iter().next().map(|(price, level)| (*price, *level));

    match (best_bid, best_ask) {
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

#[test]
fn empty_lifecycle_projects_to_an_empty_book_set() {
    let projection = OrderBookProjection::from_lifecycle(&OrderLifecycle::new()).expect("empty");

    assert!(projection.is_empty());
    assert_eq!(projection.len(), 0);
}

#[test]
fn aggregates_orders_at_the_same_price_and_preserves_price_order() {
    let first = base_order();
    let mut second = first.clone();
    second.order_number = 2;
    second.quantity = 40;
    let mut lower = first.clone();
    lower.order_number = 3;
    lower.price_paise = 249_900;
    lower.quantity = 10;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &lower);
    insert(&mut lifecycle, &second);
    insert(&mut lifecycle, &first);

    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");
    let book = projection.get(&key(&first)).expect("instrument book");

    assert_eq!(
        book.bids().collect::<Vec<_>>(),
        vec![
            (
                249_900,
                PriceLevel {
                    order_count: 1,
                    total_quantity: 10,
                }
            ),
            (
                250_000,
                PriceLevel {
                    order_count: 2,
                    total_quantity: 140,
                }
            ),
        ]
    );
    assert_eq!(
        book.best_bid(),
        Some((
            250_000,
            PriceLevel {
                order_count: 2,
                total_quantity: 140
            }
        ))
    );
}

#[test]
fn every_small_four_order_book_matches_reference_levels_and_top() {
    const ORDER_COUNT: u32 = 4;
    const ORDER_STATES: u32 = 5;
    let mut observed_top_classes = [false; 6];

    for encoded_configuration in 0..ORDER_STATES.pow(ORDER_COUNT) {
        let mut encoded = encoded_configuration;
        let mut events = Vec::new();
        let mut expected_bids = BTreeMap::<u64, PriceLevel>::new();
        let mut expected_asks = BTreeMap::<u64, PriceLevel>::new();

        for order_index in 0..ORDER_COUNT {
            let state = encoded % ORDER_STATES;
            encoded /= ORDER_STATES;
            if state == 0 {
                continue;
            }

            let mut event = base_order();
            event.order_number = u64::from(order_index) + 1;
            event.quantity = u64::from(order_index) + 1;
            event.side = if state <= 2 { Side::Buy } else { Side::Sell };
            event.price_paise = if state % 2 == 1 { 250_000 } else { 250_100 };

            let levels = match event.side {
                Side::Buy => &mut expected_bids,
                Side::Sell => &mut expected_asks,
            };
            let level = levels.entry(event.price_paise).or_default();
            level.order_count += 1;
            level.total_quantity += event.quantity;
            events.push(event);
        }

        let mut forward = OrderLifecycle::new();
        for event in &events {
            insert(&mut forward, event);
        }
        let mut reverse = OrderLifecycle::new();
        for event in events.iter().rev() {
            insert(&mut reverse, event);
        }

        let projection = OrderBookProjection::from_lifecycle(&forward).expect("small projection");
        let reversed =
            OrderBookProjection::from_lifecycle(&reverse).expect("reversed small projection");
        assert_eq!(
            projection, reversed,
            "configuration {encoded_configuration}"
        );

        if events.is_empty() {
            assert!(projection.is_empty());
            observed_top_classes[0] = true;
            continue;
        }

        assert_eq!(projection.len(), 1);
        let book = projection.get(&key(&events[0])).expect("single test book");
        assert_eq!(
            book.bids().collect::<Vec<_>>(),
            expected_bids
                .iter()
                .map(|(price, level)| (*price, *level))
                .collect::<Vec<_>>(),
            "bid levels for configuration {encoded_configuration}"
        );
        assert_eq!(
            book.asks().collect::<Vec<_>>(),
            expected_asks
                .iter()
                .map(|(price, level)| (*price, *level))
                .collect::<Vec<_>>(),
            "ask levels for configuration {encoded_configuration}"
        );
        let actual_top = book.top();
        assert_eq!(
            actual_top,
            reference_top(&expected_bids, &expected_asks),
            "top of book for configuration {encoded_configuration}"
        );
        observed_top_classes[match actual_top {
            TopOfBook::Empty => 0,
            TopOfBook::BidOnly { .. } => 1,
            TopOfBook::AskOnly { .. } => 2,
            TopOfBook::Normal { .. } => 3,
            TopOfBook::Locked { .. } => 4,
            TopOfBook::Crossed { .. } => 5,
        }] = true;
    }

    assert!(observed_top_classes.into_iter().all(|observed| observed));
}

#[test]
fn separates_bid_and_ask_levels_and_exposes_exact_best_prices() {
    let bid = base_order();
    let mut ask = bid.clone();
    ask.order_number = 2;
    ask.side = Side::Sell;
    ask.price_paise = 250_200;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &bid);
    insert(&mut lifecycle, &ask);

    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");
    let book = projection.get(&key(&bid)).expect("instrument book");

    assert_eq!(book.best_bid().expect("bid").0, 250_000);
    assert_eq!(book.best_ask().expect("ask").0, 250_200);
}

#[test]
fn locked_and_crossed_prices_are_preserved_not_filtered() {
    let bid = base_order();
    let mut ask = bid.clone();
    ask.order_number = 2;
    ask.side = Side::Sell;
    ask.price_paise = 249_900;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &bid);
    insert(&mut lifecycle, &ask);

    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");
    let book = projection.get(&key(&bid)).expect("instrument book");

    assert_eq!(book.best_bid().expect("bid").0, 250_000);
    assert_eq!(book.best_ask().expect("ask").0, 249_900);
}

#[test]
fn sessions_and_raw_instruments_are_separate_books() {
    let regular = base_order();
    let mut pre_open = regular.clone();
    pre_open.order_number = 2;
    pre_open.session = Session::PreOpen;
    let mut other_symbol = regular.clone();
    other_symbol.order_number = 3;
    other_symbol.symbol_raw = *b"       TCS";
    other_symbol.symbol = "TCS".to_owned();
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &regular);
    insert(&mut lifecycle, &pre_open);
    insert(&mut lifecycle, &other_symbol);

    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");

    assert_eq!(projection.len(), 3);
    assert!(projection.get(&key(&regular)).is_some());
    assert!(projection.get(&key(&pre_open)).is_some());
    assert!(projection.get(&key(&other_symbol)).is_some());
}

#[test]
fn projection_reflects_only_current_modified_and_cancelled_state() {
    let entry = base_order();
    let mut modify = entry.clone();
    modify.activity = OrderActivity::Modify;
    modify.exchange_jiffy = ExchangeJiffy::new(entry.exchange_jiffy.raw() + 1);
    modify.quantity = 75;
    modify.price_paise = 250_100;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &entry);
    lifecycle.apply(&modify).expect("modify");

    let modified = OrderBookProjection::from_lifecycle(&lifecycle).expect("modified projection");
    let book = modified.get(&key(&entry)).expect("instrument book");
    assert_eq!(book.best_bid().expect("bid").0, 250_100);
    assert_eq!(book.best_bid().expect("bid").1.total_quantity, 75);

    let mut cancel = modify;
    cancel.activity = OrderActivity::Cancel;
    cancel.exchange_jiffy = ExchangeJiffy::new(cancel.exchange_jiffy.raw() + 1);
    lifecycle.apply(&cancel).expect("cancel");
    assert!(
        OrderBookProjection::from_lifecycle(&lifecycle)
            .expect("cancelled projection")
            .is_empty()
    );
}

#[test]
fn projection_is_identical_for_equivalent_active_state_in_any_entry_order() {
    let first = base_order();
    let mut second = first.clone();
    second.order_number = 2;
    second.side = Side::Sell;
    second.price_paise = 250_200;
    let mut third = first.clone();
    third.order_number = 3;
    third.session = Session::PreOpen;

    let mut forward = OrderLifecycle::new();
    insert(&mut forward, &first);
    insert(&mut forward, &second);
    insert(&mut forward, &third);
    let mut reverse = OrderLifecycle::new();
    insert(&mut reverse, &third);
    insert(&mut reverse, &second);
    insert(&mut reverse, &first);

    assert_eq!(
        OrderBookProjection::from_lifecycle(&forward),
        OrderBookProjection::from_lifecycle(&reverse)
    );
}

#[test]
fn quantity_overflow_is_an_explicit_error() {
    let mut maximum = base_order();
    maximum.quantity = u64::MAX;
    let mut one = maximum.clone();
    one.order_number = 2;
    one.quantity = 1;
    let expected_book = key(&maximum);
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &maximum);
    insert(&mut lifecycle, &one);

    assert_eq!(
        OrderBookProjection::from_lifecycle(&lifecycle),
        Err(BookProjectionError::QuantityOverflow {
            book: expected_book,
            side: Side::Buy,
            price_paise: maximum.price_paise,
        })
    );
}

#[test]
fn top_of_book_classifies_empty_and_one_sided_states() {
    assert_eq!(PriceBook::default().top(), TopOfBook::Empty);

    let bid = base_order();
    let level = PriceLevel {
        order_count: 1,
        total_quantity: bid.quantity,
    };
    let mut bids = OrderLifecycle::new();
    insert(&mut bids, &bid);
    let projection = OrderBookProjection::from_lifecycle(&bids).expect("bid projection");
    assert_eq!(
        projection.get(&key(&bid)).expect("book").top(),
        TopOfBook::BidOnly {
            price_paise: 250_000,
            level,
        }
    );

    let mut ask = bid.clone();
    ask.side = Side::Sell;
    let mut asks = OrderLifecycle::new();
    insert(&mut asks, &ask);
    let projection = OrderBookProjection::from_lifecycle(&asks).expect("ask projection");
    assert_eq!(
        projection.get(&key(&ask)).expect("book").top(),
        TopOfBook::AskOnly {
            price_paise: 250_000,
            level,
        }
    );
}

#[test]
fn top_of_book_reports_exact_normal_spread() {
    let bid = base_order();
    let mut ask = bid.clone();
    ask.order_number = 2;
    ask.side = Side::Sell;
    ask.price_paise = 250_250;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &bid);
    insert(&mut lifecycle, &ask);
    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");
    let level = PriceLevel {
        order_count: 1,
        total_quantity: bid.quantity,
    };

    assert_eq!(
        projection.get(&key(&bid)).expect("book").top(),
        TopOfBook::Normal {
            bid_price_paise: 250_000,
            bid: level,
            ask_price_paise: 250_250,
            ask: level,
            spread_paise: 250,
        }
    );
}

#[test]
fn top_of_book_classifies_locked_state_without_dropping_levels() {
    let bid = base_order();
    let mut ask = bid.clone();
    ask.order_number = 2;
    ask.side = Side::Sell;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &bid);
    insert(&mut lifecycle, &ask);
    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");
    let level = PriceLevel {
        order_count: 1,
        total_quantity: bid.quantity,
    };

    assert_eq!(
        projection.get(&key(&bid)).expect("book").top(),
        TopOfBook::Locked {
            price_paise: 250_000,
            bid: level,
            ask: level,
        }
    );
}

#[test]
fn top_of_book_reports_exact_cross_amount_without_repair() {
    let bid = base_order();
    let mut ask = bid.clone();
    ask.order_number = 2;
    ask.side = Side::Sell;
    ask.price_paise = 249_900;
    let mut lifecycle = OrderLifecycle::new();
    insert(&mut lifecycle, &bid);
    insert(&mut lifecycle, &ask);
    let projection = OrderBookProjection::from_lifecycle(&lifecycle).expect("projection");
    let level = PriceLevel {
        order_count: 1,
        total_quantity: bid.quantity,
    };

    assert_eq!(
        projection.get(&key(&bid)).expect("book").top(),
        TopOfBook::Crossed {
            bid_price_paise: 250_000,
            bid: level,
            ask_price_paise: 249_900,
            ask: level,
            cross_paise: 100,
        }
    );
}
