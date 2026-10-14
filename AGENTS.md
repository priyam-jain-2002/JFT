# JFT Agent Operating Guide

This file is the default operating contract for every human or AI agent working in this repository. Apply it from the repository root downward unless a more specific `AGENTS.md` overrides it.

## Repository boundaries

- Treat every file under `sources/` as read-only reference material.
- Do not edit, rename, move, or delete synced project files in `sources/`.
- Keep generated files, implementation work, and project documentation outside `sources/`.
- Preserve unrelated human changes. Never discard or rewrite work merely to simplify a task.
- Never commit secrets, credentials, personal data, or production data.

## How to work

1. Read this file, `README.md`, `docs/PHASE_CHECKLIST.md`, and both decision logs before changing the project.
2. Establish the current phase, desired outcome, acceptance criteria, constraints, and open questions.
3. Inspect the repository before proposing or implementing changes. Prefer evidence over assumptions.
4. Use the smallest relevant installed skill for the task. Read its complete `SKILL.md` before acting and follow its validation workflow. Typical routing:
   - research or product discovery: research primary sources, capture evidence and uncertainty, and update the phase checklist;
   - product definition: define the user, problem, outcome, scope, non-goals, risks, and measurable success criteria;
   - implementation: use the language/framework skill plus applicable testing, security, accessibility, and performance skills;
   - documents, slides, spreadsheets, PDFs, images, and websites: use the matching artifact skill and perform its required visual or functional checks;
   - review: use the relevant code, security, UX, product, or artifact-review skill without silently expanding scope.
5. If no suitable skill exists, use established project conventions and document important assumptions.
6. Make changes in small, reviewable units. Avoid speculative abstractions and unrelated cleanup.
7. Validate in proportion to risk. Run the narrowest useful checks first, then broader checks before handoff.
8. Update governance records in the same change when applicable:
   - architectural, product, process, or tooling choice: `docs/decisions/DECISION_LOG.md` and `docs/decisions/COMPACT_DECISION_LOG.md`;
   - meaningful repository change: `docs/CHANGE_LEDGER.md`;
   - phase progress or scope: `docs/PHASE_CHECKLIST.md`.
9. Report the outcome, validation performed, residual risk, and next unfinished item.

## Research practice

- Start with a specific question or decision the research must support.
- Prefer primary and authoritative sources; record links, access dates, and relevant limitations.
- Separate facts, interpretation, assumptions, and recommendations.
- Triangulate consequential claims. Flag weak, stale, conflicting, or missing evidence.
- Do not present an inference as a verified fact.
- Store research outputs under `docs/research/`; never alter source material to fit a conclusion.

## Product practice

- Tie work to a named user problem and expected outcome.
- State scope and non-goals. Avoid feature work without an explicit value hypothesis.
- Define acceptance criteria and a measurable success signal before implementation when practical.
- Consider usability, accessibility, privacy, security, reliability, cost, operations, and rollback.
- Prefer a small testable slice over a broad unvalidated build.
- Capture consequential trade-offs in the decision logs.

## Development practice

- Follow existing architecture and conventions unless an active decision intentionally changes them.
- Keep interfaces explicit and changes reversible where practical.
- Handle failures deliberately; do not hide errors or fabricate successful results.
- Add or update tests for changed behavior. Include negative paths and boundary cases when relevant.
- Avoid new dependencies unless their value, maintenance cost, license, and security posture are understood.
- Keep documentation aligned with observable behavior.

## Validation and completion

A task is complete only when the requested outcome exists and the evidence supports it.

- Run relevant formatting, static analysis, unit, integration, security, build, and end-to-end checks.
- For UI work, inspect responsive behavior, keyboard use, accessibility, loading, empty, error, and success states.
- For artifacts, render or open the final output and inspect it visually.
- If a check cannot run, say why, record the risk, and do not claim it passed.
- Never weaken tests or validation merely to obtain a green result.

## Red-flag protocol

If a human recommendation could materially harm users, security, privacy, data integrity, legal compliance, reliability, accessibility, finances, reputation, or project maintainability, stop before implementing the harmful part and raise a visible warning:

> ⚠️ RED FLAG — [risk category]: [specific recommendation] could cause [concrete impact]. Evidence/assumption: [basis]. Safer option: [alternative]. Decision needed from: [owner].

Then:

1. Clarify the intent and provide evidence, severity, likelihood, blast radius, and reversibility.
2. Offer at least one safer path that still serves the underlying goal.
3. Refuse actions that are illegal, unsafe, deceptive, destructive without authorization, or outside policy.
4. For a consciously accepted material risk, create or update a decision entry with the owner, rationale, mitigations, review date, and rollback plan.
5. Never bury a red flag in routine status text or mark the related checklist item complete while the risk is unresolved.

## Decision-record rules

- Use stable IDs in the form `DEC-YYYY-NNN`.
- Every decision has one status: `Proposed`, `Active`, `Superseded`, or `Rejected`.
- Only `Active` decisions govern current work.
- Never rewrite history to replace a decision. Add the new decision, mark the old one `Superseded`, and link both entries.
- Update the full and compact logs together. The compact log is an index, not a separate source of truth.
- Record a decision only when it has meaningful alternatives or future consequences; routine edits belong only in the change ledger.

## Change-ledger rules

- Append meaningful changes; do not rewrite historical entries.
- State what changed, why, validation performed, related decision or phase, and any follow-up.
- Group trivial edits made for one outcome into a single ledger entry.

## Phase-checklist rules

- Use `[x]` for verified completion, `[ ]` for remaining work, and `[-]` for intentionally skipped work with a reason.
- Each phase must have an outcome, entry criteria, exit criteria, tasks, validation, and open risks.
- Do not mark a phase complete while required exit criteria or unresolved red flags remain.
- Add newly discovered work to the relevant phase instead of relying on chat history.
