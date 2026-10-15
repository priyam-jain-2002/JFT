# jft-nse-cm

Authoritative Rust data plane for NSE Cash Market Order and Trade data.

This crate owns source integrity, layout dispatch, fixed-width parsing, exact jiffy handling, canonical events, lifecycle reduction, order-book replay, reconciliation, quarantine, and deterministic output hashes.

Python must consume the versioned canonical artifacts emitted by this crate. It must not parse raw NSE Order/Trade bytes or maintain a second authoritative order book.

The current milestone contains only the compile-checked module boundary. Record semantics and external dependencies will be introduced through separately reviewed steps.
