//! Narrow research-only exporter for the supplied public legacy CM sample.
//!
//! This is intentionally not a production Full parser. It accepts only the
//! documented 87-byte pre-2025 Full Order layout and emits top-of-book
//! snapshots for one symbol so downstream Python can run a bounded alpha
//! diagnostic without parsing raw exchange records.

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use jft_nse_cm::book::{BookKey, OrderBookProjection, TopOfBook};
use jft_nse_cm::events::{MarketSegment, OrderActivity, OrderEvent, Session, Side};
use jft_nse_cm::framing::FixedWidthLfRecords;
use jft_nse_cm::jiffy::ExchangeJiffy;
use jft_nse_cm::layouts::{FieldLayout, LEGACY_FULL_ORDER_FIELDS, LEGACY_FULL_ORDER_RECORD_LEN};
use jft_nse_cm::lifecycle::OrderLifecycle;

const VERSION: &str = "0.1.0";
const TRAIN_PERCENT: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    orders: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Instrument {
    symbol_raw: [u8; 10],
    series_raw: [u8; 2],
}

#[derive(Debug, Default)]
struct ExportCounts {
    parsed_records: usize,
    selected_events: usize,
    applied_events: usize,
    lifecycle_anomalies: usize,
    normal: usize,
    locked: usize,
    crossed: usize,
    one_sided_or_empty: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("legacy alpha snapshot export failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args(env::args().skip(1))?;
    let source = fs::read(&config.orders)
        .map_err(|error| format!("cannot read {}: {error}", config.orders.display()))?;
    let record_count = validate_framing(&source)?;
    let training_cutoff = record_count * TRAIN_PERCENT / 100;
    let selected = select_training_instrument(&source, training_cutoff)?;
    let output = File::create(&config.output)
        .map_err(|error| format!("cannot create {}: {error}", config.output.display()))?;
    let mut writer = BufWriter::new(output);
    writeln!(
        writer,
        "snapshot_index,source_record_index,raw_jiffy,bid_price_paise,bid_quantity,ask_price_paise,ask_quantity,mid_price_x2,spread_paise,top_state"
    )
    .map_err(|error| format!("cannot write CSV header: {error}"))?;

    let counts = export_selected_snapshots(&source, selected, &mut writer)?;
    writer
        .flush()
        .map_err(|error| format!("cannot flush {}: {error}", config.output.display()))?;

    println!("source_records={record_count}");
    println!("training_cutoff_record={training_cutoff}");
    println!(
        "selected_symbol_raw={}",
        String::from_utf8_lossy(&selected.symbol_raw)
    );
    println!(
        "selected_series_raw={}",
        String::from_utf8_lossy(&selected.series_raw)
    );
    println!("parsed_records={}", counts.parsed_records);
    println!("selected_events={}", counts.selected_events);
    println!("applied_events={}", counts.applied_events);
    println!("lifecycle_anomalies={}", counts.lifecycle_anomalies);
    println!("normal_snapshots={}", counts.normal);
    println!("locked_snapshots={}", counts.locked);
    println!("crossed_observations={}", counts.crossed);
    println!(
        "one_sided_or_empty_observations={}",
        counts.one_sided_or_empty
    );
    println!("output={}", config.output.display());
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let mut orders = None;
    let mut output = None;
    let mut iterator = args.into_iter();

    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "--orders" => {
                orders = Some(PathBuf::from(
                    iterator
                        .next()
                        .ok_or_else(|| "--orders requires a path".to_owned())?,
                ));
            }
            "--output" => {
                output = Some(PathBuf::from(
                    iterator
                        .next()
                        .ok_or_else(|| "--output requires a path".to_owned())?,
                ));
            }
            "--help" | "-h" => {
                println!(
                    "Usage: legacy_alpha_snapshot_export --orders PATH --output PATH\n\
                     Export research-only top-of-book snapshots from the supplied 87-byte legacy CM Order sample."
                );
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("legacy_alpha_snapshot_export {VERSION}");
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }

    Ok(Config {
        orders: orders.ok_or_else(|| "missing required --orders PATH".to_owned())?,
        output: output.ok_or_else(|| "missing required --output PATH".to_owned())?,
    })
}

fn validate_framing(source: &[u8]) -> Result<usize, String> {
    let mut records = 0;
    for framed in FixedWidthLfRecords::<LEGACY_FULL_ORDER_RECORD_LEN>::new(source) {
        framed.map_err(|error| {
            format!(
                "invalid legacy Order framing at record {} byte {}: {:?}",
                error.record_index, error.byte_offset, error.reason
            )
        })?;
        records += 1;
    }
    if records == 0 {
        return Err("legacy Order source contains no records".to_owned());
    }
    Ok(records)
}

fn select_training_instrument(source: &[u8], training_cutoff: usize) -> Result<Instrument, String> {
    let mut counts = BTreeMap::<Instrument, usize>::new();
    for framed in FixedWidthLfRecords::<LEGACY_FULL_ORDER_RECORD_LEN>::new(source) {
        let framed =
            framed.map_err(|error| format!("framing changed between passes: {error:?}"))?;
        if framed.record_index >= training_cutoff {
            break;
        }
        let event = parse_legacy_full_order(framed.raw, framed.record_index)?;
        let instrument = Instrument {
            symbol_raw: event.symbol_raw,
            series_raw: event.series_raw,
        };
        *counts.entry(instrument).or_default() += 1;
    }

    counts
        .into_iter()
        .max_by(|(left_key, left_count), (right_key, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_key.cmp(left_key))
        })
        .map(|(instrument, _)| instrument)
        .ok_or_else(|| "training window contains no parsed instruments".to_owned())
}

fn export_selected_snapshots(
    source: &[u8],
    selected: Instrument,
    writer: &mut impl Write,
) -> Result<ExportCounts, String> {
    let mut lifecycle = OrderLifecycle::new();
    let mut counts = ExportCounts::default();
    let mut snapshot_index = 0;

    for framed in FixedWidthLfRecords::<LEGACY_FULL_ORDER_RECORD_LEN>::new(source) {
        let framed =
            framed.map_err(|error| format!("framing changed between passes: {error:?}"))?;
        let event = parse_legacy_full_order(framed.raw, framed.record_index)?;
        counts.parsed_records += 1;
        if event.symbol_raw != selected.symbol_raw || event.series_raw != selected.series_raw {
            continue;
        }
        counts.selected_events += 1;

        if lifecycle.apply(&event).is_err() {
            counts.lifecycle_anomalies += 1;
            continue;
        }
        counts.applied_events += 1;

        let projection = OrderBookProjection::from_lifecycle(&lifecycle)
            .map_err(|error| format!("book projection overflow: {error:?}"))?;
        let book_key = BookKey {
            session: event.session,
            segment: event.segment,
            symbol_raw: event.symbol_raw,
            series_raw: event.series_raw,
        };
        let Some(book) = projection.get(&book_key) else {
            counts.one_sided_or_empty += 1;
            continue;
        };

        let row = match book.top() {
            TopOfBook::Normal {
                bid_price_paise,
                bid,
                ask_price_paise,
                ask,
                spread_paise,
            } => {
                counts.normal += 1;
                Some((
                    bid_price_paise,
                    bid.total_quantity,
                    ask_price_paise,
                    ask.total_quantity,
                    spread_paise,
                    "normal",
                ))
            }
            TopOfBook::Locked {
                price_paise,
                bid,
                ask,
            } => {
                counts.locked += 1;
                Some((
                    price_paise,
                    bid.total_quantity,
                    price_paise,
                    ask.total_quantity,
                    0,
                    "locked",
                ))
            }
            TopOfBook::Crossed { .. } => {
                counts.crossed += 1;
                None
            }
            TopOfBook::Empty | TopOfBook::BidOnly { .. } | TopOfBook::AskOnly { .. } => {
                counts.one_sided_or_empty += 1;
                None
            }
        };

        if let Some((bid_price, bid_quantity, ask_price, ask_quantity, spread, state)) = row {
            let mid_price_x2 = bid_price
                .checked_add(ask_price)
                .ok_or_else(|| "mid-price addition overflow".to_owned())?;
            writeln!(
                writer,
                "{snapshot_index},{},{},{bid_price},{bid_quantity},{ask_price},{ask_quantity},{mid_price_x2},{spread},{state}",
                framed.record_index,
                event.exchange_jiffy.raw()
            )
            .map_err(|error| format!("cannot write snapshot: {error}"))?;
            snapshot_index += 1;
        }
    }

    Ok(counts)
}

fn parse_legacy_full_order(record: &[u8], record_index: usize) -> Result<OrderEvent, String> {
    if record.len() != LEGACY_FULL_ORDER_RECORD_LEN {
        return Err(format!(
            "record {record_index} has width {}, expected {LEGACY_FULL_ORDER_RECORD_LEN}",
            record.len()
        ));
    }

    let session = match field(record, LEGACY_FULL_ORDER_FIELDS[0]) {
        b"PO" => Session::PreOpen,
        b"RM" => Session::RegularMarket,
        b"CA" => Session::ClosingAuction,
        value => {
            return Err(format!(
                "record {record_index} has unsupported session {value:?}"
            ));
        }
    };
    if field(record, LEGACY_FULL_ORDER_FIELDS[1]) != b"CASH" {
        return Err(format!("record {record_index} is not CASH"));
    }
    let side = match field(record, LEGACY_FULL_ORDER_FIELDS[4]) {
        b"B" => Side::Buy,
        b"S" => Side::Sell,
        value => {
            return Err(format!(
                "record {record_index} has unsupported side {value:?}"
            ));
        }
    };
    let activity = match field(record, LEGACY_FULL_ORDER_FIELDS[5]) {
        b"1" => OrderActivity::Entry,
        b"3" => OrderActivity::Cancel,
        b"4" => OrderActivity::Modify,
        value => {
            return Err(format!(
                "record {record_index} has unsupported activity {value:?}"
            ));
        }
    };
    let symbol_raw = exact_field::<10>(record, LEGACY_FULL_ORDER_FIELDS[6]);
    let series_raw = exact_field::<2>(record, LEGACY_FULL_ORDER_FIELDS[7]);
    if !symbol_raw.is_ascii() || !series_raw.is_ascii() {
        return Err(format!(
            "record {record_index} has non-ASCII instrument bytes"
        ));
    }

    Ok(OrderEvent {
        session,
        segment: MarketSegment::Cash,
        order_number: parse_unsigned(record, LEGACY_FULL_ORDER_FIELDS[2], record_index)?,
        exchange_jiffy: ExchangeJiffy::new(parse_unsigned(
            record,
            LEGACY_FULL_ORDER_FIELDS[3],
            record_index,
        )?),
        side,
        activity,
        symbol_raw,
        symbol: String::from_utf8(symbol_raw.to_vec()).expect("ASCII checked"),
        series_raw,
        series: String::from_utf8(series_raw.to_vec()).expect("ASCII checked"),
        // For this bounded legacy pilot, the documented Original Quantity and
        // Limit Price fields are projected into the existing canonical event.
        quantity: parse_unsigned(record, LEGACY_FULL_ORDER_FIELDS[9], record_index)?,
        price_paise: parse_unsigned(record, LEGACY_FULL_ORDER_FIELDS[10], record_index)?,
    })
}

fn field(record: &[u8], layout: FieldLayout) -> &[u8] {
    &record[layout.offset..layout.end()]
}

fn exact_field<const N: usize>(record: &[u8], layout: FieldLayout) -> [u8; N] {
    field(record, layout)
        .try_into()
        .expect("documented field width must match destination")
}

fn parse_unsigned(record: &[u8], layout: FieldLayout, record_index: usize) -> Result<u64, String> {
    let raw = field(record, layout);
    if !raw.iter().all(u8::is_ascii_digit) {
        return Err(format!(
            "record {record_index} field {} is not unsigned ASCII",
            layout.name
        ));
    }
    raw.iter().try_fold(0_u64, |value, digit| {
        value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(digit - b'0')))
            .ok_or_else(|| format!("record {record_index} field {} overflows u64", layout.name))
    })
}

#[cfg(test)]
mod tests {
    use super::parse_legacy_full_order;
    use jft_nse_cm::events::{OrderActivity, Session, Side};

    const SAMPLE: &[u8] =
        b"RMCASH100000000000008591566165709472B1b11NSETESTEQ00000000000000080001000000000000NNN03";

    #[test]
    fn parses_the_public_sample_legacy_projection_fields() {
        let event = parse_legacy_full_order(SAMPLE, 0).expect("legacy sample record");

        assert_eq!(event.session, Session::RegularMarket);
        assert_eq!(event.order_number, 1_000_000_000_000_085);
        assert_eq!(event.exchange_jiffy.raw(), 91_566_165_709_472);
        assert_eq!(event.side, Side::Buy);
        assert_eq!(event.activity, OrderActivity::Entry);
        assert_eq!(event.symbol_raw, *b"b11NSETEST");
        assert_eq!(event.series_raw, *b"EQ");
        assert_eq!(event.quantity, 8);
        assert_eq!(event.price_paise, 10_000);
    }

    #[test]
    fn rejects_non_numeric_evidence_instead_of_guessing() {
        let mut record = SAMPLE.to_vec();
        record[22] = b'-';

        let error = parse_legacy_full_order(&record, 7).expect_err("invalid jiffy");
        assert!(error.contains("record 7 field exchange_jiffy"));
    }
}
