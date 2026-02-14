# MCM-Finder CLI Reference

Binary: `mcm`

## Help

```bash
cd backend
cargo run --bin mcm -- --help
cargo run --bin mcm -- search --help
```

## Search Command

```bash
cargo run --bin mcm -- search "<query>" [options]
```

Common options:
- `--version <mc-version>` (ex: `1.20.1`)
- `--loader <loader>` (repeatable; `fabric|forge|quilt|neoforge`)
- `--category <category>` (repeatable)
- `--min-downloads <count>`
- `--updated-within <days>`
- `--open-source`
- `--provider <provider>` (repeatable; `modrinth|curseforge`)
- `--sort <mode>`
- `--force-refresh`

Sort modes:
- `relevance`
- `update-recency`
- `downloads`
- `created`

## Session Commands

```bash
# Save current shortlist/session data
cargo run --bin mcm -- save-session --help

# Load saved session by ID
cargo run --bin mcm -- load-session --help
```

## Example Workflows

```bash
# Fast relevance search
cargo run --bin mcm -- search "fps boost"

# Legacy modpack audit
cargo run --bin mcm -- search "map" \
  --version 1.7.10 \
  --loader forge \
  --sort update-recency

# Provider-restricted query
cargo run --bin mcm -- search "questing" --provider modrinth --provider curseforge
```

## Notes

- `CURSEFORGE_API_KEY` is required for CurseForge API access.
- Without the key, searches still run using available providers in degraded mode.
