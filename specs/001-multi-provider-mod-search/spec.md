# Feature Specification: Professional Multi-Provider Mod Finder

**Feature Branch**: `001-multi-provider-mod-search`  
**Created**: 2026-02-13  
**Status**: Draft  
**Input**: User description: "mcm-finder (minecraft-mod-finder) as a professional, simple mod search tool with basic search, advanced filters, deep non-API discovery, composite information views, and multi-provider support (required: Modrinth/CurseForge, phased: GitHub/mc百科) across CLI + Web UI."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Unified Professional Search (Priority: P1)

As a modpack creator or server operator, I want to run one search from CLI or Web UI and get a
single consolidated result set from primary providers, so I can quickly shortlist viable mods.

**Why this priority**: Unified search is the core value proposition and must work before any advanced
analysis features.

**Independent Test**: Execute the same query and filter set in CLI and Web UI; verify both channels
return the same consolidated result content for Modrinth and CurseForge.

**Acceptance Scenarios**:

1. **Given** Modrinth and CurseForge are reachable, **When** a user searches for a keyword,
   **Then** the system returns one merged result list with provider labels and normalized metadata.
2. **Given** the same query and filters, **When** the user runs search in CLI and Web UI,
   **Then** both channels produce equivalent result sets and ordering.

---

### User Story 2 - Advanced Version-Aware Filtering (Priority: P2)

As an advanced user, I want to filter by compatibility and metadata and sort by updates relevant to
my target Minecraft version, so outdated-but-compatible mods are not incorrectly excluded.

**Why this priority**: Professional workflows depend on precise compatibility filtering more than raw
keyword matching.

**Independent Test**: Run a search constrained to a legacy Minecraft version (for example 1.7.10)
with advanced filters and confirm only compatible items are returned and ranked by version-relevant
update recency.

**Acceptance Scenarios**:

1. **Given** a target Minecraft version and loader filter, **When** search runs,
   **Then** only mods that satisfy all filters are returned.
2. **Given** sort mode is update recency with a target version, **When** results are ranked,
   **Then** ranking uses updates relevant to that target version rather than unrelated latest updates.

---

### User Story 3 - Deep Discovery and Composite Insights (Priority: P3)

As a professional evaluator, I want deep discovery for relationships not directly exposed in provider
search and a composite mod view, so I can make confident decisions from one investigation flow.

**Why this priority**: Deep relationship discovery and synthesis improves decision quality but builds
on the core search and filter workflow.

**Independent Test**: Execute a query that requires relationship discovery, wait for completion, and
verify results include discovered signals plus an integrated summary per mod.

**Acceptance Scenarios**:

1. **Given** a query requiring deep relationship discovery, **When** search starts,
   **Then** the system waits for discovery completion before showing final results and displays
   explicit progress while processing.
2. **Given** a selected mod result, **When** the user opens details,
   **Then** the system shows a composite profile with cross-provider evidence, compatibility context,
   and a concise decision-oriented summary.

### Edge Cases

- One required provider is temporarily unavailable during search.
- Providers return conflicting metadata for the same mod identity.
- A query returns very broad results that exceed practical review volume.
- Target Minecraft version is rare and produces sparse matches.
- Deep discovery finds weak or ambiguous relationship signals.
- Locally cached client results become stale after provider-side updates.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide equivalent core search workflows in both CLI and Web UI.
- **FR-002**: System MUST execute each search across Modrinth and CurseForge in the initial release
  and present one consolidated result set.
- **FR-003**: Each result MUST include source provider, mod name, summary description, compatibility
  versions, supported loaders, and project destination link.
- **FR-004**: Users MUST be able to apply advanced filters including Minecraft version, loader,
  category/tag, update recency window, and provider scope.
- **FR-005**: When Minecraft version filtering is active, system MUST exclude non-compatible mods and
  rank by updates relevant to the selected version.
- **FR-006**: System MUST perform deep discovery for relationship patterns not directly available in
  standard provider search responses.
- **FR-007**: System MUST not publish partial final results before deep discovery completes; progress
  state MUST remain visible while processing.
- **FR-008**: System MUST provide a composite mod detail view that combines available provider
  evidence and highlights metadata conflicts.
- **FR-009**: System MUST generate an integrated, user-readable summary of compatibility,
  maintenance signals, and potential adoption risks for each shortlisted mod.
- **FR-010**: Initial release MUST treat Modrinth and CurseForge as required providers while keeping
  provider expansion to GitHub and mc百科 compatible with the same user workflow.
- **FR-011**: Users MUST be able to preserve consolidated findings with query context for later
  comparison and team review.
- **FR-012**: System MUST support combined information review so users can compare multiple mod
  candidates within one search session.
- **FR-013**: System MUST use multi-layer caching (provider response cache, consolidated result cache,
  and client-local cache) to reduce repeated-query latency and external API pressure.
- **FR-014**: CLI and Web UI clients MUST persist recent search sessions locally with freshness
  metadata and provide explicit refresh controls.

### Non-Functional Requirements *(mandatory)*

- **NFR-001**: For searches with up to 200 combined candidates before filtering, at least 95% of
  searches MUST show final results within 30 seconds under normal provider availability.
- **NFR-002**: Repeating the same query with the same filters and availability conditions MUST produce
  consistent top-20 ordering in at least 99% of runs.
- **NFR-003**: If one or more providers are unavailable, users MUST receive an explicit degraded
  coverage notice within 10 seconds after failure detection.
- **NFR-004**: In acceptance evaluation, at least 90% of target users MUST rate result usefulness at
  4/5 or better for professional mod selection tasks.
- **NFR-005**: At least 85% of first-time target users MUST complete a standard search-and-compare
  task in under 5 minutes without facilitator assistance.
- **NFR-006**: At least 70% of repeated searches using identical query and filters within 24 hours
  MUST return final results in 3 seconds or less.
- **NFR-007**: Client-local cached sessions MUST expose freshness status immediately and MUST block
  automatic reuse of entries beyond configured freshness windows unless the user explicitly opts in.

### Key Entities *(include if feature involves data)*

- **Search Query Profile**: User search intent including keywords, filters, provider scope, and sort
  mode.
- **Provider Result Record**: Normalized representation of one provider's evidence for a mod candidate.
- **Consolidated Mod Profile**: Aggregated cross-provider view of one mod with compatibility and risk
  signals.
- **Discovery Evidence Item**: Relationship evidence found through deep discovery beyond basic search
  output.
- **Search Session Summary**: Preserved snapshot of query context, resulting shortlist, and comparison
  notes.

### Assumptions & Dependencies

- CLI and Web UI are both in scope for this feature and must expose equivalent core capabilities.
- Modrinth and CurseForge are mandatory providers for initial delivery.
- GitHub and mc百科 are planned as phased provider expansions after initial release.
- Search results are intentionally withheld until deep discovery completes.
- Publicly accessible provider data remains available for professional research use.
- Users and environments permit local persistence of search sessions on the client side.
- "Professional user" means users who regularly create modpacks, operate modded servers, or perform
  repeat compatibility/risk evaluations for others.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least 85% of target professional users can complete a valid mod shortlist for a
  specified Minecraft version in 3 minutes or less.
- **SC-002**: At least 95% of evaluated searches return at least one user-accepted candidate within
  the top 10 results.
- **SC-003**: Compared with manual single-provider research, median time to produce a final shortlist
  decreases by at least 40%.
- **SC-004**: At least 90% of acceptance participants report higher confidence in final mod choices
  after reviewing consolidated evidence and summaries.
- **SC-005**: At least 70% of repeated searches in the same day complete in 3 seconds or less through
  cache reuse while preserving equivalent shortlist quality.

## UX Refinement Considerations *(optional backlog for self-use quality)*

- **Saved Search Profiles**: Users can save and reuse named filter/query presets for repeated research
  workflows (for example, legacy-version audits vs. latest-release audits).
- **Ranking Transparency**: Each result exposes a short "why this rank" explanation so users can trust
  shortlist ordering decisions.
- **Side-by-Side Comparison Mode**: Users can compare two or more candidate mods in one view with clear
  differences in compatibility, update activity, and risk signals.
- **Coverage Confidence Indicator**: Every search shows whether coverage is full or degraded and how
  that impacts decision confidence.
- **Staleness Visibility**: Cached results clearly show freshness status and last validation time to
  avoid accidental use of outdated information.
- **Resume Workflow**: Users can reopen the latest investigation session and continue from where
  they stopped without rebuilding filters manually.
- **Export for Team Review**: Shortlists can be exported in a review-friendly format with evidence and
  decision notes attached.
- **Noise Reduction Controls**: Users can hide low-signal results and focus on high-confidence matches
  when time is limited.
