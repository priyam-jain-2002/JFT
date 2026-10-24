# Python research plane

Python begins at the versioned canonical-data boundary emitted by the Rust data plane.

Implemented ownership currently includes a typed, immutable manifest-v1 reader that validates the explicit JSON Schema and cross-field integrity invariants without reading raw NSE records. Future ownership includes Arrow/Parquet canonical readers, diagnostics, features, labels, datasets, baselines, model training, evaluation, and visualization. Raw NSE Order/Trade parsing and authoritative order-book reconstruction are intentionally excluded.
