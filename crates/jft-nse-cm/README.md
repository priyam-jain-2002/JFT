# jft-nse-cm

Authoritative Rust data plane for NSE Cash Market Order and Trade data.

This crate owns source integrity, layout dispatch, fixed-width framing/parsing, exact jiffy handling, canonical events, lifecycle reduction, order-book replay, reconciliation, quarantine, and deterministic output hashes.

Python must consume the versioned canonical artifacts emitted by this crate. It must not parse raw NSE Order/Trade bytes or maintain a second authoritative order book.

Implemented evidence-backed slices currently cover exact LF framing, lazy byte-addressable outcomes over caller-provided source bytes, current CM Trim Order/Trade layouts, strict field parsing, lossless quarantine, raw-jiffy decomposition, and synthetic fixtures. Disk-backed input, lifecycle reduction, book replay, cross-stream ordering, trigger verification, and canonical columnar output are not implemented yet.
