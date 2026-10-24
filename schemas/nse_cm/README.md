# NSE CM canonical schemas

This directory holds reviewed, language-neutral manifest and future canonical-event contracts. Active DEC-2026-009 authorizes manifest integrity v1, versioning, and payload-based record identity. Canonical Order/Trade event fields, units, nullability, and wire-to-canonical mappings remain deliberately unfrozen pending representative current-data validation.

[`manifest.schema.json`](manifest.schema.json) is the active manifest-v1 public contract. Rust emits a deterministic compact JSON representation with a terminal LF and rejects unknown fields, schema IDs, major versions, noncanonical digest text, and out-of-range Rust integers. The synthetic golden manifest lives under `tests/fixtures/nse_cm/synthetic_nonlicensed/`; the Python reader independently validates this schema, reproduces the exact golden bytes, and applies the cross-field invariants that JSON Schema does not express.

The absence of a canonical-event schema here does not mean the exchange wire format is unimplemented. Rust already owns and tests date-dispatched fixed-width descriptors for current Trim 68/82-byte records and documented Full 87/91-byte Orders plus 100/101/103-byte Trades. Those descriptors define source byte boundaries and era selection; they must not be mistaken for the future Rust/Python canonical contract.

The manifest schema is explicit and reviewed rather than generated implicitly from Rust types. Do not add a canonical event schema until a separate owner decision records the representative-data evidence.
