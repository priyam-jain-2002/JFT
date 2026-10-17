# jft-nse-cm

Authoritative Rust data plane for NSE Cash Market Order and Trade data.

This crate owns source integrity, layout dispatch, fixed-width framing/parsing, exact jiffy handling, canonical events, lifecycle reduction, order-book replay, reconciliation, quarantine, and deterministic output hashes.

Python must consume the versioned canonical artifacts emitted by this crate. It must not parse raw NSE Order/Trade bytes or maintain a second authoritative order book.

Implemented evidence-backed slices currently cover strict CM Full/Trim filename decoding, homogeneous stream-set discovery without a fixed stream count, exact data/trigger stream-member pairing, exact LF framing with borrowed corrupt-tail evidence, lazy byte-addressable outcomes over caller-provided source bytes, current Trim/Full and historical Full 87/100/101 Order/Trade layout maps, validated date dispatch including the one-day Full exception and Trim introduction, strict Trim field parsing, lossless quarantine, raw-jiffy decomposition, and synthetic current-layout fixtures. Directory-level grouping, Full semantic parsing, disk-backed input, lifecycle reduction, book replay, cross-stream ordering, trigger verification, and canonical columnar output are not implemented yet.
