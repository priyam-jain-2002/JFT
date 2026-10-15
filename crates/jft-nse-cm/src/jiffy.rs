//! Exact exchange-jiffy representation and conversion.

pub const JIFFIES_PER_SECOND: u64 = 65_536;

/// Exact NSE exchange time expressed as jiffies since 1980-01-01 00:00:00 GMT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExchangeJiffy(u64);

impl ExchangeJiffy {
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn whole_seconds_since_1980(self) -> u64 {
        self.0 / JIFFIES_PER_SECOND
    }

    #[must_use]
    pub const fn subsecond_jiffies(self) -> u16 {
        (self.0 % JIFFIES_PER_SECOND) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::ExchangeJiffy;

    #[test]
    fn preserves_nse_example_without_float_rounding() {
        let jiffy = ExchangeJiffy::new(91_200_611_817_221);

        assert_eq!(jiffy.raw(), 91_200_611_817_221);
        assert_eq!(jiffy.whole_seconds_since_1980(), 1_391_610_898);
        assert_eq!(jiffy.subsecond_jiffies(), 5_893);
    }
}
