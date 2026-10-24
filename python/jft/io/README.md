# Canonical I/O

`manifest.py` is the fail-closed Python consumer for manifest v1 emitted by Rust. It validates the Draft 2020-12 schema, rejects duplicate JSON keys and unknown versions/fields, constructs immutable typed values, verifies trigger/count/byte/jiffy invariants, and reproduces the Rust golden JSON bytes. It never reads raw NSE Order/Trade records.

Install the pinned Python 3.9-compatible dependency set and run conformance with:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r python/requirements.txt
PYTHONPYCACHEPREFIX=/tmp/jft-pycache PYTHONPATH=python .venv/bin/python -m unittest discover -s python/tests -t python -p 'test_*.py' -v
```

Arrow/Parquet canonical-data readers remain future work and must preserve this Rust/Python ownership boundary.
