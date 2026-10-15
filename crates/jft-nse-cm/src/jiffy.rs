//! Exact exchange-jiffy representation and lossless decomposition.

pub const JIFFIES_PER_SECOND: u64 = 65_536;

/// Exact raw NSE exchange-jiffy counter.
///
/// NSE v1.19 is internally inconsistent about the counter's epoch: its prose
/// names 1980-01-01 00:00:00 GMT, while its published Unix-offset constant is
/// 19,800 seconds smaller than the actual UTC offset. Until that is resolved,
/// this type deliberately makes no UTC or local-time claim.
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
    pub const fn whole_exchange_seconds(self) -> u64 {
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
        assert_eq!(jiffy.whole_exchange_seconds(), 1_391_610_898);
        assert_eq!(jiffy.subsecond_jiffies(), 5_893);
    }

    #[test]
    fn published_epoch_constant_differs_from_utc_by_ist_offset() {
        const DAYS_FROM_UNIX_EPOCH_TO_1980_UTC: u64 = 10 * 365 + 2;
        const SECONDS_PER_DAY: u64 = 86_400;
        const ACTUAL_1980_UTC_OFFSET_SECONDS: u64 =
            DAYS_FROM_UNIX_EPOCH_TO_1980_UTC * SECONDS_PER_DAY;
        const NSE_PUBLISHED_OFFSET_SECONDS: u64 = 315_513_000;
        const IST_OFFSET_SECONDS: u64 = 5 * 3_600 + 30 * 60;

        assert_eq!(ACTUAL_1980_UTC_OFFSET_SECONDS, 315_532_800);
        assert_eq!(
            ACTUAL_1980_UTC_OFFSET_SECONDS - NSE_PUBLISHED_OFFSET_SECONDS,
            IST_OFFSET_SECONDS
        );
    }
}
