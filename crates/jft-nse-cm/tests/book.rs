use jft_nse_cm::book::{BookKey, BookProjectionError, OrderBookProjection, PriceLevel};
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
