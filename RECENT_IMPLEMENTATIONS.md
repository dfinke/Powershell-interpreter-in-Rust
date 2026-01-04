# Recent Implementations

This file tracks the most recent features implemented in this repo (what changed, where, and how it was verified).

## 2026-01-04

### Week 16 — Get-Content — Chunk 3 (Filtering Parameters)

**Implemented (PowerShell-aligned)**: `Get-Content -TotalCount`, `-Tail`.

**Notes**:
- `-TotalCount` and `-Tail` are mutually exclusive (error if both are supplied).
- Parameters must be non-negative integers.

**Files changed**:
- `crates/pwsh-cmdlets/src/get_content.rs` — added parameter parsing + filtered line reading + unit tests.

**Verification**:
- `cargo test` (workspace)
- `cargo clippy --all-targets --all-features -- -D warnings` (workspace)

### CI / Tooling — fix clippy warning in `Get-Content`

**Fixed**: `clippy::explicit-counter-loop` in the `-TotalCount` code path by switching to `enumerate()`.

**Why**: GitHub Actions runs `cargo clippy --all-targets --all-features -- -D warnings`, so this warning broke the build.

### Select-Object — add `-Skip` (PowerShell-aligned)

**Implemented**: `Select-Object -Skip N` and combinations like `-Skip 2 -First 10`.

**Why**: Keeps `Get-Content` aligned with native PowerShell while still supporting common pipeline shaping patterns.

**Files changed**:
- `crates/pwsh-cmdlets/src/select_object.rs` — added `-Skip` + validated integer parsing.
- `crates/pwsh-cmdlets/tests/integration_tests.rs` — added pipeline coverage (`Get-Content | Select-Object -Skip -First`).

### Week 16 — Set-Content — Chunk 4 (Write file contents)

**Implemented**: `Set-Content -Path <path> -Value <value>` with pipeline support.

**Behavior**:
- Overwrites existing files (and creates new files).
- Accepts content via `-Value`, pipeline input, or a second positional argument.
- Arrays write one element per line.

**Files changed**:
- `crates/pwsh-cmdlets/src/set_content.rs` — new cmdlet + unit tests.
- `crates/pwsh-cmdlets/src/lib.rs` — register `Set-Content`.
- `crates/pwsh-cmdlets/tests/integration_tests.rs` — end-to-end tests.

**Verification**:
- `cargo test` (workspace)
- `cargo clippy --all-targets --all-features -- -D warnings` (workspace)
