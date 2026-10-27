"""Chronological trade-price momentum diagnostic for the legacy CM sample."""

from __future__ import annotations

import argparse
import csv
import json
import math
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, List, Sequence, Tuple


VERSION = "0.1.0"
DEFAULT_HORIZONS = (10, 50, 200)


@dataclass(frozen=True)
class TradePrint:
    trade_index: int
    source_record_index: int
    raw_jiffy: int
    price_paise: int
    quantity: int


@dataclass(frozen=True)
class WindowMetrics:
    horizon_trades: int
    samples: int
    nonzero_labels: int
    hit_rate: float
    training_majority_direction: int
    majority_hit_rate: float
    information_coefficient: float
    mean_gross_signal_return_bps: float
    mean_abs_forward_return_bps: float


@dataclass(frozen=True)
class HorizonResult:
    horizon_trades: int
    validation: WindowMetrics
    test: WindowMetrics


@dataclass(frozen=True)
class TradePilotReport:
    methodology: str
    trade_count: int
    train_range: Tuple[int, int]
    validation_range: Tuple[int, int]
    test_range: Tuple[int, int]
    horizons: Tuple[HorizonResult, ...]
    limitations: Tuple[str, ...]


def read_trades(path: Path) -> Tuple[TradePrint, ...]:
    required = {
        "trade_index",
        "source_record_index",
        "raw_jiffy",
        "price_paise",
        "quantity",
    }
    try:
        with path.open("r", encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source)
            if set(reader.fieldnames or ()) != required:
                raise ValueError("trade CSV header does not match the canonical pilot contract")
            rows = tuple(_parse_row(row, line) for line, row in enumerate(reader, 2))
    except OSError as err:
        raise ValueError(f"cannot read trade CSV {path}: {err}") from err
    if not rows:
        raise ValueError("trade CSV contains no data rows")
    for expected, trade in enumerate(rows):
        if trade.trade_index != expected:
            raise ValueError(f"trade_index must be contiguous: expected {expected}, got {trade.trade_index}")
        if expected > 0 and trade.source_record_index <= rows[expected - 1].source_record_index:
            raise ValueError("source_record_index must be strictly increasing")
    return rows


def _parse_row(row: dict, line: int) -> TradePrint:
    try:
        trade = TradePrint(
            trade_index=int(row["trade_index"]),
            source_record_index=int(row["source_record_index"]),
            raw_jiffy=int(row["raw_jiffy"]),
            price_paise=int(row["price_paise"]),
            quantity=int(row["quantity"]),
        )
    except (KeyError, TypeError, ValueError) as err:
        raise ValueError(f"line {line} contains an invalid trade: {err}") from err
    if trade.price_paise <= 0 or trade.quantity <= 0:
        raise ValueError(f"line {line} contains a non-positive price or quantity")
    return trade


def analyze_trades(
    trades: Sequence[TradePrint], horizons: Sequence[int] = DEFAULT_HORIZONS
) -> TradePilotReport:
    if len(trades) < 20:
        raise ValueError("at least 20 trades are required")
    if not horizons or any(horizon <= 0 for horizon in horizons):
        raise ValueError("horizons must contain positive trade counts")
    train_end = len(trades) * 60 // 100
    validation_end = len(trades) * 80 // 100
    ranges = ((0, train_end), (train_end, validation_end), (validation_end, len(trades)))
    results: List[HorizonResult] = []
    for horizon in horizons:
        majority = _training_majority_direction(trades, ranges[0], horizon)
        results.append(
            HorizonResult(
                horizon_trades=horizon,
                validation=_evaluate_window(trades, ranges[1], horizon, majority),
                test=_evaluate_window(trades, ranges[2], horizon, majority),
            )
        )
    return TradePilotReport(
        methodology=(
            "Past trade-price return predicts same-direction future trade-price return. "
            "Chronological 60/20/20 windows; non-overlapping observations; fixed sign; "
            "no parameter tuning, fees, spread, latency, slippage, or fill model."
        ),
        trade_count=len(trades),
        train_range=ranges[0],
        validation_range=ranges[1],
        test_range=ranges[2],
        horizons=tuple(results),
        limitations=(
            "One anonymized public legacy sample day is not independent out-of-sample evidence.",
            "Trade-event horizons are not clock-time horizons and activity varies intraday.",
            "Gross signal return is mark-to-trade-price diagnostics, not executable P&L.",
            "The pilot ignores spread, fees, latency, slippage, impact, and fill probability.",
        ),
    )


def _training_majority_direction(
    trades: Sequence[TradePrint], window: Tuple[int, int], horizon: int
) -> int:
    observations = tuple(_observations(trades, window, horizon))
    if not observations:
        raise ValueError(f"horizon {horizon} has no training observations")
    up = sum(future > 0 for _, future in observations)
    down = sum(future < 0 for _, future in observations)
    return 1 if up >= down else -1


def _evaluate_window(
    trades: Sequence[TradePrint],
    window: Tuple[int, int],
    horizon: int,
    majority: int,
) -> WindowMetrics:
    observations = tuple(_observations(trades, window, horizon))
    if not observations:
        raise ValueError(f"horizon {horizon} has no observations in window {window}")
    past = [item[0] for item in observations]
    future = [item[1] for item in observations]
    nonzero = [index for index, value in enumerate(future) if value != 0 and past[index] != 0]
    hits = sum(past[index] * future[index] > 0 for index in nonzero)
    majority_hits = sum(majority * future[index] > 0 for index in nonzero)
    gross = [_sign(signal) * value for signal, value in zip(past, future)]
    return WindowMetrics(
        horizon_trades=horizon,
        samples=len(observations),
        nonzero_labels=len(nonzero),
        hit_rate=_ratio(hits, len(nonzero)),
        training_majority_direction=majority,
        majority_hit_rate=_ratio(majority_hits, len(nonzero)),
        information_coefficient=_pearson(past, future),
        mean_gross_signal_return_bps=sum(gross) / len(gross),
        mean_abs_forward_return_bps=sum(abs(value) for value in future) / len(future),
    )


def _observations(
    trades: Sequence[TradePrint], window: Tuple[int, int], horizon: int
) -> Iterable[Tuple[float, float]]:
    start, end = window
    first = start + horizon
    for index in range(first, end - horizon, horizon):
        before = trades[index - horizon].price_paise
        current = trades[index].price_paise
        future = trades[index + horizon].price_paise
        yield (
            (current - before) * 10_000.0 / before,
            (future - current) * 10_000.0 / current,
        )


def _sign(value: float) -> int:
    if value > 0:
        return 1
    if value < 0:
        return -1
    return 0


def _ratio(numerator: int, denominator: int) -> float:
    return numerator / denominator if denominator else 0.0


def _pearson(left: Sequence[float], right: Sequence[float]) -> float:
    if len(left) != len(right) or not left:
        raise ValueError("Pearson inputs must have the same non-zero length")
    left_mean = sum(left) / len(left)
    right_mean = sum(right) / len(right)
    covariance = sum((x - left_mean) * (y - right_mean) for x, y in zip(left, right))
    left_variance = sum((x - left_mean) ** 2 for x in left)
    right_variance = sum((y - right_mean) ** 2 for y in right)
    denominator = math.sqrt(left_variance * right_variance)
    return covariance / denominator if denominator else 0.0


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m jft.research.trade_alpha_pilot",
        description="Run the bounded legacy CM trade-price momentum pilot.",
    )
    parser.add_argument("--version", action="version", version=VERSION)
    parser.add_argument("--trades", required=True, type=Path, help="Rust-exported trade CSV")
    parser.add_argument("--output", required=True, type=Path, help="JSON report destination")
    return parser


def main(argv: Sequence[str] = ()) -> int:
    arguments = _parser().parse_args(list(argv) if argv else None)
    try:
        report = analyze_trades(read_trades(arguments.trades))
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(
            json.dumps(asdict(report), indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
    except (OSError, ValueError) as err:
        print(f"trade alpha pilot failed: {err}", file=sys.stderr)
        return 1
    print(f"trades={report.trade_count}")
    for result in report.horizons:
        print(
            "horizon={h} test_samples={n} hit_rate={hit:.4f} "
            "baseline={baseline:.4f} ic={ic:.6f} gross_bps={gross:.6f}".format(
                h=result.horizon_trades,
                n=result.test.samples,
                hit=result.test.hit_rate,
                baseline=result.test.majority_hit_rate,
                ic=result.test.information_coefficient,
                gross=result.test.mean_gross_signal_return_bps,
            )
        )
    print(f"output={arguments.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
