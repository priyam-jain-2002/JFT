# NSE CM canonical schemas

This directory is reserved for reviewed, language-neutral canonical-event and manifest contracts. No canonical schema has been frozen: field identity, versioning, units, enums, nullability, compatibility, lineage keys, and wire-to-canonical mappings remain gated by proposed DEC-2026-009.

The absence of files here does not mean the exchange wire format is unimplemented. Rust already owns and tests date-dispatched fixed-width descriptors for current Trim 68/82-byte records and documented Full 87/91-byte Orders plus 100/101/103-byte Trades. Those descriptors define source byte boundaries and era selection; they must not be mistaken for the future Rust/Python canonical contract.

Do not add generated or ad hoc schemas here before the owner activates or amends DEC-2026-009.
