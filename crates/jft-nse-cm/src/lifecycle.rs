//! Explicit order lifecycle transitions and anomaly classification.

use std::collections::BTreeMap;

use crate::events::{MarketSegment, OrderActivity, OrderEvent, Session, Side};
use crate::jiffy::ExchangeJiffy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderKey {
    pub session: Session,
    pub segment: MarketSegment,
    pub symbol_raw: [u8; 10],
    pub series_raw: [u8; 2],
    pub order_number: u64,
}

impl From<&OrderEvent> for OrderKey {
    fn from(event: &OrderEvent) -> Self {
        Self {
            session: event.session,
            segment: event.segment,
            symbol_raw: event.symbol_raw,
            series_raw: event.series_raw,
            order_number: event.order_number,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveOrder {
    pub side: Side,
    pub quantity: u64,
    pub price_paise: u64,
    pub entry_jiffy: ExchangeJiffy,
    pub last_event_jiffy: ExchangeJiffy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTransition {
    Entered {
        current: ActiveOrder,
    },
    Modified {
        previous: ActiveOrder,
        current: ActiveOrder,
    },
    Cancelled {
        previous: ActiveOrder,
        cancel_jiffy: ExchangeJiffy,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleAnomaly {
    DuplicateEntry {
        key: OrderKey,
        active: ActiveOrder,
        incoming_jiffy: ExchangeJiffy,
    },
    UnknownModify {
        key: OrderKey,
        incoming_jiffy: ExchangeJiffy,
    },
    UnknownCancel {
        key: OrderKey,
        incoming_jiffy: ExchangeJiffy,
    },
    RegressingJiffy {
        key: OrderKey,
        activity: OrderActivity,
        previous_jiffy: ExchangeJiffy,
        incoming_jiffy: ExchangeJiffy,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrderLifecycle {
    active: BTreeMap<OrderKey, ActiveOrder>,
}

impl OrderLifecycle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.active.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    #[must_use]
    pub fn get(&self, key: &OrderKey) -> Option<&ActiveOrder> {
        self.active.get(key)
    }

    pub fn active_orders(&self) -> impl Iterator<Item = (&OrderKey, &ActiveOrder)> {
        self.active.iter()
    }

    pub fn apply(&mut self, event: &OrderEvent) -> Result<LifecycleTransition, LifecycleAnomaly> {
        let key = OrderKey::from(event);
        match event.activity {
            OrderActivity::Entry => self.enter(key, event),
            OrderActivity::Modify => self.modify(key, event),
            OrderActivity::Cancel => self.cancel(key, event),
        }
    }

    fn enter(
        &mut self,
        key: OrderKey,
        event: &OrderEvent,
    ) -> Result<LifecycleTransition, LifecycleAnomaly> {
        if let Some(active) = self.active.get(&key).copied() {
            return Err(LifecycleAnomaly::DuplicateEntry {
                key,
                active,
                incoming_jiffy: event.exchange_jiffy,
            });
        }

        let current = ActiveOrder {
            side: event.side,
            quantity: event.quantity,
            price_paise: event.price_paise,
            entry_jiffy: event.exchange_jiffy,
            last_event_jiffy: event.exchange_jiffy,
        };
        self.active.insert(key, current);
        Ok(LifecycleTransition::Entered { current })
    }

    fn modify(
        &mut self,
        key: OrderKey,
        event: &OrderEvent,
    ) -> Result<LifecycleTransition, LifecycleAnomaly> {
        let previous = self
            .active
            .get(&key)
            .copied()
            .ok_or(LifecycleAnomaly::UnknownModify {
                key,
                incoming_jiffy: event.exchange_jiffy,
            })?;
        reject_regressing_jiffy(key, previous, event)?;

        let current = ActiveOrder {
            side: event.side,
            quantity: event.quantity,
            price_paise: event.price_paise,
            entry_jiffy: previous.entry_jiffy,
            last_event_jiffy: event.exchange_jiffy,
        };
        self.active.insert(key, current);
        Ok(LifecycleTransition::Modified { previous, current })
    }

    fn cancel(
        &mut self,
        key: OrderKey,
        event: &OrderEvent,
    ) -> Result<LifecycleTransition, LifecycleAnomaly> {
        let previous = self
            .active
            .get(&key)
            .copied()
            .ok_or(LifecycleAnomaly::UnknownCancel {
                key,
                incoming_jiffy: event.exchange_jiffy,
            })?;
        reject_regressing_jiffy(key, previous, event)?;

        self.active.remove(&key);
        Ok(LifecycleTransition::Cancelled {
            previous,
            cancel_jiffy: event.exchange_jiffy,
        })
    }
}

fn reject_regressing_jiffy(
    key: OrderKey,
    previous: ActiveOrder,
    event: &OrderEvent,
) -> Result<(), LifecycleAnomaly> {
    if event.exchange_jiffy < previous.last_event_jiffy {
        return Err(LifecycleAnomaly::RegressingJiffy {
            key,
            activity: event.activity,
            previous_jiffy: previous.last_event_jiffy,
            incoming_jiffy: event.exchange_jiffy,
        });
    }
    Ok(())
}
