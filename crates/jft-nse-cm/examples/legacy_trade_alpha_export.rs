//! Research-only trade-print exporter for the supplied public legacy CM sample.

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use jft_nse_cm::events::Session;
use jft_nse_cm::framing::FixedWidthLfRecords;
use jft_nse_cm::layouts::{
    FieldLayout, LEGACY_FULL_TRADE_100_FIELDS, LEGACY_FULL_TRADE_100_RECORD_LEN,
};

const VERSION: &str = "0.1.0";
const TRAIN_PERCENT: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    trades: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Instrument {
    symbol_raw: [u8; 10],
    series_raw: [u8; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TradePrint {
    session: Session,
    raw_jiffy: u64,
    instrument: Instrument,
    price_paise: u64,
    quantity: u64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("legacy trade alpha export failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args(env::args().skip(1))?;
    let source = fs::read(&config.trades)
        .map_err(|error| format!("cannot read {}: {error}", config.trades.display()))?;
    let record_count = validate_framing(&source)?;
    let training_cutoff = record_count * TRAIN_PERCENT / 100;
    let selected = select_training_instrument(&source, training_cutoff)?;
    let output = File::create(&config.output)
        .map_err(|error| format!("cannot create {}: {error}", config.output.display()))?;
    let mut writer = BufWriter::new(output);
    writeln!(
        writer,
        "trade_index,source_record_index,raw_jiffy,price_paise,quantity"
    )
    .map_err(|error| format!("cannot write CSV header: {error}"))?;

    let mut selected_trades = 0;
    for framed in FixedWidthLfRecords::<LEGACY_FULL_TRADE_100_RECORD_LEN>::new(&source) {
        let framed =
            framed.map_err(|error| format!("framing changed between passes: {error:?}"))?;
        let trade = parse_legacy_trade(framed.raw, framed.record_index)?;
        if trade.instrument != selected {
            continue;
        }
        writeln!(
            writer,
            "{selected_trades},{},{},{},{}",
            framed.record_index, trade.raw_jiffy, trade.price_paise, trade.quantity
        )
        .map_err(|error| format!("cannot write trade print: {error}"))?;
        selected_trades += 1;
    }
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
    println!("selected_trades={selected_trades}");
    println!("output={}", config.output.display());
    Ok(())
}

fn parse_args<I>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let mut trades = None;
    let mut output = None;
    let mut iterator = args.into_iter();
    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "--trades" => {
                trades = Some(PathBuf::from(
                    iterator
                        .next()
                        .ok_or_else(|| "--trades requires a path".to_owned())?,
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
                    "Usage: legacy_trade_alpha_export --trades PATH --output PATH\n\
                     Export the most active training-window instrument from the supplied 100-byte legacy CM Trade sample."
                );
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("legacy_trade_alpha_export {VERSION}");
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    Ok(Config {
        trades: trades.ok_or_else(|| "missing required --trades PATH".to_owned())?,
        output: output.ok_or_else(|| "missing required --output PATH".to_owned())?,
    })
}

fn validate_framing(source: &[u8]) -> Result<usize, String> {
    let mut records = 0;
    for framed in FixedWidthLfRecords::<LEGACY_FULL_TRADE_100_RECORD_LEN>::new(source) {
        framed.map_err(|error| {
            format!(
                "invalid legacy Trade framing at record {} byte {}: {:?}",
                error.record_index, error.byte_offset, error.reason
            )
        })?;
        records += 1;
    }
    if records == 0 {
        return Err("legacy Trade source contains no records".to_owned());
    }
    Ok(records)
}

fn select_training_instrument(source: &[u8], training_cutoff: usize) -> Result<Instrument, String> {
    let mut counts = BTreeMap::<Instrument, usize>::new();
    for framed in FixedWidthLfRecords::<LEGACY_FULL_TRADE_100_RECORD_LEN>::new(source) {
        let framed =
            framed.map_err(|error| format!("framing changed between passes: {error:?}"))?;
        if framed.record_index >= training_cutoff {
            break;
        }
        let trade = parse_legacy_trade(framed.raw, framed.record_index)?;
        *counts.entry(trade.instrument).or_default() += 1;
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

fn parse_legacy_trade(record: &[u8], record_index: usize) -> Result<TradePrint, String> {
    if record.len() != LEGACY_FULL_TRADE_100_RECORD_LEN {
        return Err(format!(
            "record {record_index} has width {}, expected {LEGACY_FULL_TRADE_100_RECORD_LEN}",
            record.len()
        ));
    }
    let session = match field(record, LEGACY_FULL_TRADE_100_FIELDS[0]) {
        b"PO" => Session::PreOpen,
        b"RM" => Session::RegularMarket,
        b"CA" => Session::ClosingAuction,
        value => {
            return Err(format!(
                "record {record_index} has unsupported session {value:?}"
            ));
        }
    };
    if field(record, LEGACY_FULL_TRADE_100_FIELDS[1]) != b"CASH" {
        return Err(format!("record {record_index} is not CASH"));
    }
    let symbol_raw = exact_field::<10>(record, LEGACY_FULL_TRADE_100_FIELDS[4]);
    let series_raw = exact_field::<2>(record, LEGACY_FULL_TRADE_100_FIELDS[5]);
    if !symbol_raw.is_ascii() || !series_raw.is_ascii() {
        return Err(format!(
            "record {record_index} has non-ASCII instrument bytes"
        ));
    }

    Ok(TradePrint {
        session,
        raw_jiffy: parse_unsigned(record, LEGACY_FULL_TRADE_100_FIELDS[3], record_index)?,
        instrument: Instrument {
            symbol_raw,
            series_raw,
        },
        price_paise: parse_unsigned(record, LEGACY_FULL_TRADE_100_FIELDS[6], record_index)?,
        quantity: parse_unsigned(record, LEGACY_FULL_TRADE_100_FIELDS[7], record_index)?,
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
    use super::parse_legacy_trade;
    use jft_nse_cm::events::Session;

    const SAMPLE: &[u8] =
        b"RMCASH202404100000000191566165716076b11NSETESTEQ0001000000000005100000000000008503100000000000008603";

    #[test]
    fn parses_the_public_sample_trade_projection_fields() {
        let trade = parse_legacy_trade(SAMPLE, 0).expect("legacy sample trade");

        assert_eq!(trade.session, Session::RegularMarket);
        assert_eq!(trade.raw_jiffy, 91_566_165_716_076);
        assert_eq!(trade.instrument.symbol_raw, *b"b11NSETEST");
        assert_eq!(trade.instrument.series_raw, *b"EQ");
        assert_eq!(trade.price_paise, 10_000);
        assert_eq!(trade.quantity, 5);
    }

    #[test]
    fn rejects_non_numeric_trade_evidence_instead_of_guessing() {
        let mut record = SAMPLE.to_vec();
        record[48] = b'-';

        let error = parse_legacy_trade(&record, 9).expect_err("invalid price");
        assert!(error.contains("record 9 field price_paise"));
    }
}
