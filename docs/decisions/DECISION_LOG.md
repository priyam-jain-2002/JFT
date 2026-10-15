# Decision Log

This is the authoritative record of consequential product, technical, tooling, and process decisions. Entries are append-only except for status and supersession metadata.

## Status meanings

- **Proposed**: under consideration; does not govern current work.
- **Active**: approved and currently governs the project.
- **Superseded**: historical; replaced by the linked decision.
- **Rejected**: considered but deliberately not adopted.

## Index

| ID | Date | Decision | Status | Supersedes | Superseded by |
|---|---|---|---|---|---|
| DEC-2026-001 | 2026-10-14 | Adopt repository governance records | Active | — | — |
| DEC-2026-002 | 2026-10-14 | Keep synced sources read-only | Active | — | — |
| DEC-2026-003 | 2026-10-15 | Adopt an evidence-gated NSE CM research path | Active | — | — |
| DEC-2026-004 | 2026-10-15 | Qualify CM Trim before purchasing or depending on Full | Active | — | — |
| DEC-2026-005 | 2026-10-15 | Separate prediction, execution, and hard risk | Active | — | — |
| DEC-2026-006 | 2026-10-15 | Hold live and profit-seeking use behind written data-rights clearance | Superseded | — | DEC-2026-007 |
| DEC-2026-007 | 2026-10-15 | Treat legal compliance as an external operational dependency | Active | DEC-2026-006 | — |
| DEC-2026-008 | 2026-10-15 | Keep deterministic market-data replay in Rust and research in Python | Active | — | — |

---

<a id="dec-2026-001"></a>

## DEC-2026-001 — Adopt repository governance records

- **Date:** 2026-10-14
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Process
- **Supersedes:** —
- **Superseded by:** —
- **Review trigger:** The records create material overhead without improving continuity, auditability, or delivery quality.

### Context

The project needs durable context across human and agent work. Important rationale, progress, and repository changes should not live only in conversations.

### Decision

Use four linked governance artifacts:

1. this full decision log as the source of truth for consequential decisions;
2. a compact decision log as the fast current-state index;
3. an append-only change ledger for meaningful repository changes; and
4. a phase checklist for progress, exit criteria, risks, and remaining work.

Update the applicable records in the same change as the work they describe.

### Alternatives considered

- **Use issues or chat history only:** rejected because this repository does not yet have a guaranteed issue workflow, and chat history is not durable project state.
- **Use only one chronological log:** rejected because decisions, changes, and execution status have different audiences and lifecycles.

### Consequences

- Project state and rationale remain visible to future contributors.
- Contributors carry a small documentation cost for meaningful changes.
- Duplicate records can drift, so the full and compact decision logs must be updated together.

### Validation

The repository contains each artifact, cross-links them from `README.md`, and defines update rules in `AGENTS.md`.

---

<a id="dec-2026-002"></a>

## DEC-2026-002 — Keep synced sources read-only

- **Date:** 2026-10-14
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Repository boundaries
- **Supersedes:** —
- **Superseded by:** —
- **Review trigger:** The source synchronization mechanism or project boundary changes.

### Context

The `sources/` directory is a local mirror of ChatGPT project material and may be replaced when a task is created from the project.

### Decision

Treat all content under `sources/` as immutable reference material. Store project-generated work elsewhere.

### Alternatives considered

- **Edit files in place:** rejected because edits can be overwritten and would blur the boundary between synced inputs and project outputs.

### Consequences

- Reference integrity is preserved.
- Any transformation or derived work must be written to a separate location.

### Validation

The boundary is stated in `AGENTS.md` and the repository map labels `sources/` as read-only.

---

<a id="dec-2026-003"></a>

## DEC-2026-003 - Adopt an evidence-gated NSE CM research path

- **Date:** 2026-10-15
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Product and research
- **Supersedes:** —
- **Superseded by:** —
- **Review trigger:** A gate fails or evidence supports a different market/horizon.

### Context

The concept ranges from short-horizon prediction to a trading farm and optional product. Building downstream infrastructure before proving data, reconstruction, signal, costs, and live behavior would create expense without evidence.

### Decision

Start with NSE CM and advance only through the chain in [`../PHASE_CHECKLIST.md`](../PHASE_CHECKLIST.md): valid data, deterministic reconstruction, OOS signal, net alpha, live shadow, small-capital evidence, measured capacity, trading farm, then a separately approved optional product.

### Alternatives considered

- **Start with many markets/instruments:** rejected because it multiplies data and microstructure uncertainty.
- **Build brokerage/product infrastructure first:** rejected because unproven alpha would not justify it.
- **Target sub-millisecond HFT immediately:** rejected because conventional cloud/broker latency is structurally mismatched.

### Consequences

- Each phase has explicit exit and kill criteria.
- One liquid instrument and 1s/5s/30s/1m/5m horizons are the initial test bed.
- Failure at an early gate prevents later spend.

### Validation

The governing plan defines phases 0-11, artifacts, metrics, blockers, and return/kill conditions.

---

<a id="dec-2026-004"></a>

## DEC-2026-004 - Qualify CM Trim before purchasing or depending on Full

- **Date:** 2026-10-15
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Data
- **Supersedes:** —
- **Superseded by:** —
- **Review trigger:** Phase 1 matched Trim/Full comparison or vendor schema change.

### Context

NSE v1.19 describes Trim as an EOD, fixed-width subset with core identifiers, jiffies, prices, quantities, activity, and trade order references. Full adds order attributes and participant flags but has 90-day Orders/30-day Trades cooling periods. The specification does not alone prove Trim is sufficient for exact replay.

### Decision

Use representative CM Trim as the first candidate. Upgrade to Full only if sample evidence shows a material benefit to reconstruction, feature value, simulation fidelity, or risk that justifies rights, delay, storage, and cost.

### Alternatives considered

- **Assume Trim is sufficient:** rejected because quantity/tie/lifecycle semantics require samples.
- **Buy/build on Full first:** rejected because auxiliary fields may not improve the initial thesis enough to justify added burden.

### Consequences

- Phase 0/1 requires a Trim qualification and, if allowed, matched Trim/Full comparison.
- Full-only features cannot enter Trim experiments.

### Validation

The decision is accepted only after deterministic replay and quantified Trim/Full divergence.

---

<a id="dec-2026-005"></a>

## DEC-2026-005 - Separate prediction, execution, and hard risk

- **Date:** 2026-10-15
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Architecture
- **Supersedes:** —
- **Superseded by:** —
- **Review trigger:** Evidence shows the boundaries prevent a required safety or performance property.

### Context

Correct forecasts can lose after costs or poor fills. Complex models can also hide leakage and bypass safety if they directly control orders.

### Decision

Use the model order `naive -> linear/logistic -> gradient boosting -> small MLP -> TCN/CNN/DeepLOB-like -> regime ensemble`. Models emit calibrated estimates. A separate deterministic policy, execution engine, broker interface, and fail-closed risk engine control orders. RL is not an initial price-discovery or execution dependency.

### Alternatives considered

- **One neural network emits orders:** rejected because it couples alpha, fills, and safety.
- **Start with the most complex model:** rejected because it lacks a trustworthy baseline and increases overfit risk.

### Consequences

- Neural complexity must prove stable OOS net uplift.
- The simulator, shadow, and live system share decision/order contracts.
- Hard risk can reject any strategy output.

### Validation

Phase gates require baseline comparisons, conservative cost simulation, contract tests, and risk failure injection.

---

<a id="dec-2026-006"></a>

## DEC-2026-006 - Hold live and profit-seeking use behind written data-rights clearance

- **Date:** 2026-10-15
- **Status:** Superseded
- **Owner:** Project owner
- **Scope:** Legal and data governance
- **Supersedes:** —
- **Superseded by:** DEC-2026-007
- **Review trigger:** Written NDAL clarification and counsel/owner approval are recorded.

### Context

The supplied NDAL Order and Trade agreement limits data to internal Annexure-II purposes, restricts data/output redistribution, says it must not be exploited for commercial purposes or financial gain, and later refers to facilitating lawful trading and investment decisions. The intended proprietary training and live trading use is therefore ambiguous.

### Decision

Treat offline research as bounded by the executed Annexure-II purpose and hold all live/profit-seeking use until written NDAL clarification and appropriate legal review are recorded. A verbal assurance or project-owner preference does not convert the agreement into confirmed permission.

### Alternatives considered

- **Assume internal proprietary trading is allowed:** rejected because the financial-gain clause is direct and consequential.
- **Stop all planning:** rejected because documentation and a bounded qualification plan can proceed without deploying capital.

### Consequences

- Phase 0 owns the legal evidence.
- Phase 7 requires live-data rights; Phase 9 additionally requires explicit profitable/live-use clearance.
- Optional platform/data-derived output requires a separate permission analysis.

### Validation

Clear the hold only with a dated evidence link/record, permitted-use scope, conditions, approver, and review trigger.

---

<a id="dec-2026-007"></a>

## DEC-2026-007 - Treat legal compliance as an external operational dependency

- **Date:** 2026-10-15
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Process and repository boundary
- **Supersedes:** DEC-2026-006
- **Superseded by:** —
- **Review trigger:** Work moves from controlled offline engineering to live-data operation, capital deployment, redistribution, or external product release.

### Context

The repository needs to progress from planning into deterministic parser and reconstruction engineering. Legal interpretation and formal compliance remain necessary for operational use, but the project owner has placed that work outside repository scope and does not want it to block local offline engineering in the owner-controlled environment.

### Decision

Begin Phase 1 fixture-first engineering while Phase 0 continues as a non-blocking parallel technical track for representative-data qualification, delivery controls, and budget. Keep legal/compliance visible as an external “needs to be done” dependency, without making it a repository deliverable or claiming it is cleared. Repository engineering, tests, synthetic fixtures, and public-sample diagnostics may proceed. Operational live-data use, capital deployment, redistribution, and external-product activation remain separate owner-controlled decisions that require the appropriate external review.

### Alternatives considered

- **Keep Phase 0 blocked until legal clearance:** rejected by the project owner because it prevents controlled offline engineering.
- **Mark legal compliance complete:** rejected because no evidence establishes clearance and the repository should not make that claim.
- **Remove every legal reference:** rejected because a visible external dependency reduces the risk of accidental operational activation.

### Consequences

- Phase 1 can start using synthetic fixtures and the qualified public sample.
- Legal work does not appear in repository acceptance criteria or engineering blockers.
- The repository must not present itself as providing legal advice or certifying permitted use.
- Phase 0 remains open for non-legal technical work: representative current data, stream/schema qualification, delivery controls, and budget.
- Representative current paired data remains required to finish technical qualification and pass Phase 1, even though it is not required to start implementation.

### Validation

The phase table, Phase 0 exit criteria, Phase 1 prerequisites, README, compact log, and immediate actions consistently authorize offline engineering, retain the non-legal technical work, and show legal compliance only as an external operational marker.

---

<a id="dec-2026-008"></a>

## DEC-2026-008 - Keep deterministic market-data replay in Rust and research in Python

- **Date:** 2026-10-15
- **Status:** Active
- **Owner:** Project owner
- **Scope:** Architecture
- **Supersedes:** —
- **Superseded by:** —
- **Review trigger:** Profiling or deployment evidence shows the file/process boundary cannot meet a declared correctness, throughput, latency, or operability requirement.

### Context

Raw fixed-width exchange parsing, exact time conversion, lifecycle mutation, and book reconstruction require strict byte accounting and deterministic performance. Research and modeling require rapid iteration and the mature Python data/ML ecosystem. Duplicating authoritative logic across both languages would create silent divergence.

### Decision

Rust owns raw input integrity, schema-era dispatch, parsing, jiffy conversion, canonical events, lifecycle/book replay, reconciliation, quarantine, deterministic hashes, and canonical Arrow/Parquet output. Python consumes only the versioned canonical boundary for diagnostics, features, labels, datasets, baselines, models, and evaluation. The first integration is a Rust CLI plus Parquet/Arrow, JSON manifests, schemas, and hashes; PyO3/FFI is deferred until measured need.

### Alternatives considered

- **Implement Phase 1 in Python:** rejected because it violates the established deterministic data-plane boundary.
- **Expose Rust immediately through PyO3:** rejected because it adds build, ABI, packaging, and failure-surface complexity before a measured need.
- **Maintain parsers/books in both languages:** rejected because two authoritative implementations would drift and undermine replay evidence.
- **Write research/modeling in Rust:** rejected because it slows iteration without strengthening the authoritative data contract.

### Consequences

- Raw NSE Order/Trade files never enter the Python research layer.
- The canonical schema, integer units, provenance, enum codes, and hashes become the cross-language contract.
- Rust golden/property/fuzz/replay tests establish source-to-canonical correctness; Python conformance tests verify manifests and canonical consumption.
- Any future Rust simulator core must reuse the canonical contracts and may not move research/model ownership out of Python.

### Validation

Phase 1 expected modules use Rust; Python begins under `python/jft/`; the governing plan includes schema/version checks and forbids a duplicate Python parser/book.

---

## Decision template

Copy this section to add a decision. Assign the next stable ID and update both indexes.

```markdown
## DEC-YYYY-NNN — Short decision title

- **Date:** YYYY-MM-DD
- **Status:** Proposed | Active | Superseded | Rejected
- **Owner:** Name or role
- **Scope:** Product | Architecture | Security | Data | Tooling | Process | Other
- **Supersedes:** DEC-YYYY-NNN or —
- **Superseded by:** DEC-YYYY-NNN or —
- **Review trigger:** Date or condition

### Context
[Problem, constraints, and evidence.]

### Decision
[The chosen direction.]

### Alternatives considered
- **Alternative:** Why it was not selected.

### Consequences
- [Positive, negative, risk, cost, and reversibility.]

### Validation
[How the decision will be tested or reviewed.]
```
