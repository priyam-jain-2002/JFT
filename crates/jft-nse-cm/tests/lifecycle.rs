use jft_nse_cm::events::{OrderActivity, OrderEvent, Session};
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::TRIM_ORDER_RECORD_LEN;
use jft_nse_cm::lifecycle::{
    ActiveOrder, LifecycleAnomaly, LifecycleTransition, OrderKey, OrderLifecycle,
};
use jft_nse_cm::parser::parse_trim_order;

const ENTRY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_entry.dat");
const MODIFY_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_modify.dat");
const CANCEL_FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/nse_cm/synthetic_nonlicensed/trim_order_cancel.dat");

fn parse_fixture(fixture: &[u8]) -> OrderEvent {
    assert_eq!(fixture.len(), TRIM_ORDER_RECORD_LEN + 1);
    assert_eq!(fixture[TRIM_ORDER_RECORD_LEN], b'\n');
    parse_trim_order(&fixture[..TRIM_ORDER_RECORD_LEN])
        .expect("synthetic lifecycle fixture must parse")
        .event
}

fn with_activity_and_jiffy(
    mut event: OrderEvent,
    activity: OrderActivity,
    jiffy: u64,
) -> OrderEvent {
    event.activity = activity;
    event.exchange_jiffy = ExchangeJiffy::new(jiffy);
    event
}

#[test]
fn parsed_entry_modify_cancel_sequence_has_exact_state_transitions() {
    let entry = parse_fixture(ENTRY_FIXTURE);
    let modify = parse_fixture(MODIFY_FIXTURE);
    let cancel = parse_fixture(CANCEL_FIXTURE);
    let key = OrderKey::from(&entry);
    let mut lifecycle = OrderLifecycle::new();

    let entered = lifecycle.apply(&entry).expect("entry");
    let entry_state = ActiveOrder {
        side: entry.side,
        quantity: 100,
        price_paise: 250_000,
        entry_jiffy: ExchangeJiffy::new(91_200_611_817_221),
        last_event_jiffy: ExchangeJiffy::new(91_200_611_817_221),
    };
    assert_eq!(
        entered,
        LifecycleTransition::Entered {
            current: entry_state
        }
    );
    assert_eq!(lifecycle.get(&key), Some(&entry_state));

    let modified = lifecycle.apply(&modify).expect("modify");
    let modified_state = ActiveOrder {
        side: modify.side,
        quantity: 75,
        price_paise: 250_100,
        entry_jiffy: entry_state.entry_jiffy,
        last_event_jiffy: ExchangeJiffy::new(91_200_611_817_223),
    };
    assert_eq!(
        modified,
        LifecycleTransition::Modified {
            previous: entry_state,
            current: modified_state,
        }
    );
    assert_eq!(lifecycle.get(&key), Some(&modified_state));

    assert_eq!(
        lifecycle.apply(&cancel).expect("cancel"),
        LifecycleTransition::Cancelled {
            previous: modified_state,
            cancel_jiffy: ExchangeJiffy::new(91_200_611_817_224),
        }
    );
    assert!(lifecycle.is_empty());
}

#[test]
fn duplicate_entry_is_classified_and_does_not_mutate_state() {
    let entry = parse_fixture(ENTRY_FIXTURE);
    let duplicate =
        with_activity_and_jiffy(entry.clone(), OrderActivity::Entry, 91_200_611_817_222);
    let key = OrderKey::from(&entry);
    let mut lifecycle = OrderLifecycle::new();
    lifecycle.apply(&entry).expect("first entry");
    let before = lifecycle.clone();

    assert_eq!(
        lifecycle.apply(&duplicate),
        Err(LifecycleAnomaly::DuplicateEntry {
            key,
            active: *before.get(&key).expect("active order"),
            incoming_jiffy: duplicate.exchange_jiffy,
        })
    );
    assert_eq!(lifecycle, before);
}

#[test]
fn unknown_modify_and_cancel_are_distinct_and_do_not_create_state() {
    let modify = parse_fixture(MODIFY_FIXTURE);
    let cancel = parse_fixture(CANCEL_FIXTURE);
    let key = OrderKey::from(&modify);
    let mut lifecycle = OrderLifecycle::new();

    assert_eq!(
        lifecycle.apply(&modify),
        Err(LifecycleAnomaly::UnknownModify {
            key,
            incoming_jiffy: modify.exchange_jiffy,
        })
    );
    assert_eq!(
        lifecycle.apply(&cancel),
        Err(LifecycleAnomaly::UnknownCancel {
            key,
            incoming_jiffy: cancel.exchange_jiffy,
        })
    );
    assert!(lifecycle.is_empty());
}

#[test]
fn regressing_modify_is_rejected_without_changing_active_state() {
    let entry = parse_fixture(ENTRY_FIXTURE);
    let modify = with_activity_and_jiffy(entry.clone(), OrderActivity::Modify, 91_200_611_817_220);
    let key = OrderKey::from(&entry);
    let mut lifecycle = OrderLifecycle::new();
    lifecycle.apply(&entry).expect("entry");
    let before = lifecycle.clone();

    assert_eq!(
        lifecycle.apply(&modify),
        Err(LifecycleAnomaly::RegressingJiffy {
            key,
            activity: OrderActivity::Modify,
            previous_jiffy: entry.exchange_jiffy,
            incoming_jiffy: modify.exchange_jiffy,
        })
    );
    assert_eq!(lifecycle, before);
}

#[test]
fn regressing_cancel_is_rejected_without_removing_active_state() {
    let entry = parse_fixture(ENTRY_FIXTURE);
    let cancel = with_activity_and_jiffy(entry.clone(), OrderActivity::Cancel, 91_200_611_817_220);
    let key = OrderKey::from(&entry);
    let mut lifecycle = OrderLifecycle::new();
    lifecycle.apply(&entry).expect("entry");
    let before = lifecycle.clone();

    assert_eq!(
        lifecycle.apply(&cancel),
        Err(LifecycleAnomaly::RegressingJiffy {
            key,
            activity: OrderActivity::Cancel,
            previous_jiffy: entry.exchange_jiffy,
            incoming_jiffy: cancel.exchange_jiffy,
        })
    );
    assert_eq!(lifecycle, before);
}

#[test]
fn same_jiffy_modify_is_accepted_in_caller_supplied_order() {
    let entry = parse_fixture(ENTRY_FIXTURE);
    let mut modify = with_activity_and_jiffy(
        entry.clone(),
        OrderActivity::Modify,
        entry.exchange_jiffy.raw(),
    );
    modify.quantity = 99;
    let key = OrderKey::from(&entry);
    let mut lifecycle = OrderLifecycle::new();
    lifecycle.apply(&entry).expect("entry");

    lifecycle.apply(&modify).expect("same-jiffy modify");
    assert_eq!(lifecycle.get(&key).expect("active").quantity, 99);
}

#[test]
fn independent_order_keys_produce_input_order_independent_active_state() {
    let first = parse_fixture(ENTRY_FIXTURE);
    let mut second = first.clone();
    second.order_number = 2;
    second.session = Session::PreOpen;

    let mut forward = OrderLifecycle::new();
    forward.apply(&first).expect("first");
    forward.apply(&second).expect("second");

    let mut reverse = OrderLifecycle::new();
    reverse.apply(&second).expect("second");
    reverse.apply(&first).expect("first");

    assert_eq!(forward, reverse);
    assert_eq!(forward.len(), 2);
    assert_eq!(forward.active_orders().count(), 2);
}
