<!--
Sync Impact Report
Version change: template (unversioned) -> 1.0.0
Modified principles:
- Template Principle 1 -> I. Correctness Before Convenience
- Template Principle 2 -> II. Mandatory Verification First
- Template Principle 3 -> III. Explicit Failure and Security Handling
- Template Principle 4 -> IV. Minimal Complexity and Maximum Reuse
- Template Principle 5 -> V. Deterministic Delivery and Traceability
Added sections:
- Engineering Quality Gates
- Development Workflow & Review
Removed sections:
- None
Templates requiring updates:
- ✅ .specify/templates/plan-template.md
- ✅ .specify/templates/spec-template.md
- ✅ .specify/templates/tasks-template.md
- ✅ .specify/templates/commands/*.md (no files found; no updates required)
- ✅ README.md (reviewed; no principle references to update)
Follow-up TODOs:
- None
-->
# mcm-finder Constitution

## Core Principles

### I. Correctness Before Convenience
- Every change MUST preserve current behavior unless a requirement explicitly authorizes a change.
- Every behavior change MUST document expected outputs, edge cases, and failure modes before coding.
- Ambiguous requirements MUST be clarified or recorded as explicit assumptions in planning artifacts.
Rationale: Correctness is the primary quality metric. Fast delivery without behavioral accuracy creates
hidden cost and user-visible defects.

### II. Mandatory Verification First
- For every new feature and bug fix, tests that can fail for the target behavior MUST be defined before
implementation.
- Verification MUST include the smallest relevant unit/integration/regression scope and MUST be re-run
after implementation.
- Unverified code MUST NOT be merged, even for small edits.
Rationale: Verification-first execution is the most reliable control against regressions and accidental
scope drift.

### III. Explicit Failure and Security Handling
- All external inputs MUST be validated at system boundaries with deterministic failure behavior.
- Broad catch-all handling and silent fallbacks are prohibited; errors MUST be surfaced or propagated.
- Secrets, credentials, and tokens MUST NOT be committed, hardcoded, or written to logs.
- Security-sensitive changes MUST describe abuse cases and mitigations in the plan or PR notes.
Rationale: Reliability and security degrade quickly when failure paths are implicit or hidden.

### IV. Minimal Complexity and Maximum Reuse
- Existing abstractions, utilities, and conventions MUST be reused before introducing new patterns.
- New abstractions MUST include a concrete justification tied to measurable maintainability or safety gains.
- Public interfaces MUST use explicit types/contracts and include usage-focused documentation.
Rationale: Unnecessary complexity reduces review quality, slows delivery, and increases defect risk.

### V. Deterministic Delivery and Traceability
- Each change MUST map files and behavior to explicit requirements and validation evidence.
- Diffs MUST remain minimal and reversible; rollback expectations MUST be documented for risky changes.
- PR and commit descriptions MUST include validation commands, risk notes, and affected user behavior.
Rationale: Traceable delivery enables consistent reviews, safer releases, and faster incident resolution.

## Engineering Quality Gates

- **Plan Gate**: Every plan MUST include a Constitution Check result for all five core principles.
- **Spec Gate**: Every spec MUST include functional requirements, edge cases, and non-functional
constraints for correctness, security, and observability.
- **Task Gate**: Task breakdowns MUST include verification, security/error-handling, and observability
work where applicable.
- **Validation Gate**: Before merge, contributors MUST execute all applicable repository checks; if a
check does not exist, manual verification evidence MUST be documented.
- **Documentation Gate**: Behavior or operational changes MUST update user/developer documentation in
the same change set.

## Development Workflow & Review

1. **Frame**: Capture scope, assumptions, and risk boundaries before implementation.
2. **Plan**: Define behavior deltas, failure handling, tests, and rollback expectations.
3. **Implement**: Apply the smallest safe change set that satisfies the scoped requirement.
4. **Verify**: Execute planned checks and record concrete results.
5. **Review**: Reviewers MUST block merges that violate any core principle unless a documented exception
is approved through governance.
6. **Integrate**: Merge only when traceability, verification evidence, and documentation updates are
complete.

## Governance

- This constitution is the highest authority for engineering process and quality in this repository.
Conflicts with other guidance are resolved in favor of this document.
- Amendments MUST be proposed in a dedicated change that includes rationale, impact analysis, and
template sync updates.
- Amendment approval requires maintainer review and explicit confirmation that dependent templates and
guidance remain consistent.
- Versioning policy follows semantic versioning for governance:
  - **MAJOR**: Principle removal or incompatible governance change.
  - **MINOR**: New principle/section or materially stronger requirement.
  - **PATCH**: Clarifications that do not change normative meaning.
- Compliance review is mandatory for plans, specs, tasks, and pull requests. Non-compliant changes MUST
be corrected before merge or covered by a time-bound governance exception.

**Version**: 1.0.0 | **Ratified**: 2026-02-13 | **Last Amended**: 2026-02-13
