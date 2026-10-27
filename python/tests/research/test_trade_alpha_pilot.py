import unittest

from jft.research.trade_alpha_pilot import TradePrint, analyze_trades


def trade(index: int, price: int) -> TradePrint:
    return TradePrint(
        trade_index=index,
        source_record_index=index * 3,
        raw_jiffy=index,
        price_paise=price,
        quantity=1,
    )


class TradeAlphaPilotTest(unittest.TestCase):
    def test_continuation_signal_detects_constructed_trend(self) -> None:
        rows = tuple(trade(index, 10_000 + index) for index in range(200))

        report = analyze_trades(rows, horizons=(2,))
        result = report.horizons[0]

        self.assertEqual(report.trade_count, 200)
        self.assertEqual(result.test.hit_rate, 1.0)
        self.assertEqual(result.test.majority_hit_rate, 1.0)
        self.assertGreater(result.test.mean_gross_signal_return_bps, 0.0)

    def test_requires_enough_trades(self) -> None:
        rows = tuple(trade(index, 10_000 + index) for index in range(19))

        with self.assertRaisesRegex(ValueError, "at least 20 trades"):
            analyze_trades(rows, horizons=(1,))

    def test_rejects_non_positive_horizon(self) -> None:
        rows = tuple(trade(index, 10_000 + index) for index in range(20))

        with self.assertRaisesRegex(ValueError, "positive trade counts"):
            analyze_trades(rows, horizons=(0,))


if __name__ == "__main__":
    unittest.main()
