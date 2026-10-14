# Phase Checklist

This is the execution source of truth. Keep it current as work is discovered and completed.

Legend: `[x]` verified complete · `[ ]` remaining · `[-]` intentionally skipped with a reason

## Current snapshot

- **Current phase:** Phase 1 — Discovery and research
- **Last updated:** 2026-10-14
- **Next milestone:** Evidence-backed product brief approved
- **Open red flags:** None

## Phase 0 — Repository foundation

**Outcome:** A version-controlled repository with durable operating and governance practices.

**Entry criteria:** Workspace is available.

### Work

- [x] Preserve the `sources/` read-only boundary.
- [x] Add agent guidance for skills, research, product, development, validation, and risk escalation.
- [x] Add full and compact decision logs with explicit status and supersession fields.
- [x] Add an append-only change ledger.
- [x] Add this phase-wise checklist and reusable conventions.
- [x] Add repository overview and safe ignore rules.
- [x] Initialize Git, set the GitHub remote, and create the initial commit.
- [x] Push the initial repository state to GitHub.

### Validation

- [x] Required files exist and internal links resolve.
- [x] Active decisions appear consistently in both decision logs.
- [x] No file under `sources/` was modified.
- [x] The remote branch contains the initial commit.

**Exit criteria:** Governance artifacts are present and the initial branch is available on the configured GitHub remote.

**Open risks:** None.

## Phase 1 — Discovery and research

**Outcome:** An evidence-backed understanding of the user, problem, environment, constraints, and opportunity.

**Entry criteria:** Phase 0 exit criteria are met.

### Work

- [ ] Identify target users, stakeholders, and decision owner.
- [ ] Write the problem statement and current user journey.
- [ ] Inventory available evidence and source quality.
- [ ] Research user needs, alternatives, competitors, and relevant technical constraints.
- [ ] Separate verified facts, assumptions, unknowns, and hypotheses.
- [ ] Identify privacy, security, legal, accessibility, operational, and reputational risks.
- [ ] Define the desired user outcome and initial success signal.
- [ ] Store evidence and citations under `docs/research/`.
- [ ] Review findings with the project owner.

### Validation

- [ ] Consequential claims are traceable to primary or authoritative sources.
- [ ] Key assumptions have a validation plan.
- [ ] Material unknowns and red flags have an owner.

**Exit criteria:** The project owner approves an evidence-backed product brief with a clear problem, target user, outcome, constraints, risks, and unknowns.

**Open risks:** Product context has not yet been provided.

## Phase 2 — Product definition and planning

**Outcome:** A bounded, testable product slice with agreed success measures and delivery plan.

**Entry criteria:** Phase 1 exit criteria are met.

### Work

- [ ] Define the value hypothesis and product principles.
- [ ] Set scope, non-goals, dependencies, and constraints.
- [ ] Write prioritized user stories or jobs and acceptance criteria.
- [ ] Define measurable success, guardrail, and operational metrics.
- [ ] Choose the smallest testable release slice.
- [ ] Map major failure modes and mitigations.
- [ ] Record consequential product and technical decisions.
- [ ] Create an implementation and validation plan.

### Validation

- [ ] Every in-scope feature maps to a user need and success signal.
- [ ] Acceptance criteria are observable and testable.
- [ ] Dependencies, owners, red flags, and rollback expectations are explicit.

**Exit criteria:** The project owner approves scope, acceptance criteria, success measures, risks, and delivery plan.

**Open risks:** None recorded yet.

## Phase 3 — Design and technical foundation

**Outcome:** Validated experience and architecture ready for incremental implementation.

**Entry criteria:** Phase 2 exit criteria are met.

### Work

- [ ] Model critical user flows, states, data, and system boundaries.
- [ ] Prototype high-risk interactions or technical assumptions.
- [ ] Define accessibility, privacy, security, reliability, and performance requirements.
- [ ] Select architecture and dependencies with documented trade-offs.
- [ ] Define testing strategy, environments, observability, migration, and rollback.
- [ ] Review the design with relevant stakeholders.

### Validation

- [ ] Critical flows include loading, empty, error, success, and recovery states.
- [ ] High-risk assumptions have prototype or research evidence.
- [ ] Architecture decisions are recorded and threat/failure modes reviewed.

**Exit criteria:** Experience and technical approach meet acceptance criteria with no unresolved blocking red flags.

**Open risks:** None recorded yet.

## Phase 4 — Incremental implementation

**Outcome:** The agreed product slice works end to end and is maintainable.

**Entry criteria:** Phase 3 exit criteria are met.

### Work

- [ ] Implement in small vertical slices.
- [ ] Add automated tests for behavior, negative paths, and boundaries.
- [ ] Add required telemetry, auditability, and operational safeguards.
- [ ] Keep documentation, decisions, ledger, and checklist current.
- [ ] Review code, security, accessibility, privacy, and performance as applicable.
- [ ] Demonstrate completed slices against acceptance criteria.

### Validation

- [ ] Formatting, static analysis, tests, security checks, and build pass.
- [ ] Critical user journeys work in the target environment.
- [ ] No unresolved high-severity defect or red flag remains.

**Exit criteria:** The release candidate satisfies agreed acceptance criteria and is ready for independent validation.

**Open risks:** None recorded yet.

## Phase 5 — Release validation

**Outcome:** Evidence shows the product slice is safe, usable, operable, and ready for controlled release.

**Entry criteria:** Phase 4 exit criteria are met.

### Work

- [ ] Run end-to-end, regression, accessibility, security, privacy, performance, and recovery checks as applicable.
- [ ] Conduct user or stakeholder acceptance validation.
- [ ] Verify telemetry, alerts, support path, runbook, backup, migration, and rollback.
- [ ] Triage residual issues by impact, likelihood, and owner.
- [ ] Record the release decision and evidence.

### Validation

- [ ] Success and guardrail metrics can be measured.
- [ ] Rollback and recovery have been tested or explicitly risk-accepted.
- [ ] Release owner signs off with no unresolved blocking red flags.

**Exit criteria:** Release owner approves deployment with documented evidence, residual risks, and rollback plan.

**Open risks:** None recorded yet.

## Phase 6 — Launch, learn, and iterate

**Outcome:** The release is operated safely and evidence guides the next decision.

**Entry criteria:** Phase 5 exit criteria are met.

### Work

- [ ] Release gradually where practical and monitor guardrails.
- [ ] Track user outcome, reliability, support, cost, and risk metrics.
- [ ] Compare results with the value hypothesis and baseline.
- [ ] Capture incidents, feedback, unexpected effects, and learnings.
- [ ] Decide to iterate, scale, hold, roll back, or retire.
- [ ] Supersede decisions whose assumptions no longer hold.

### Validation

- [ ] Post-release metrics and qualitative evidence have been reviewed.
- [ ] Follow-up actions have owners and target dates.
- [ ] Decisions, ledger, and next phase reflect observed results.

**Exit criteria:** Evidence is reviewed, the next product decision is recorded, and follow-up work is owned.

**Open risks:** None recorded yet.

## Phase template

```markdown
## Phase N — Name

**Outcome:** Observable result.

**Entry criteria:** What must be true before work starts.

### Work
- [ ] Task.

### Validation
- [ ] Check and evidence.

**Exit criteria:** What must be true to complete the phase.

**Open risks:** Risks, owners, and mitigation; or `None`.
```
