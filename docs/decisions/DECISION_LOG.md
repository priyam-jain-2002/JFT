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
