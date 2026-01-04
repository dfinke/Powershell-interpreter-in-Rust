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

### Select-Object — add `-Skip` (PowerShell-aligned)

**Implemented**: `Select-Object -Skip N` and combinations like `-Skip 2 -First 10`.

**Why**: Keeps `Get-Content` aligned with native PowerShell while still supporting common pipeline shaping patterns.

**Files changed**:
- `crates/pwsh-cmdlets/src/select_object.rs` — added `-Skip` + validated integer parsing.
- `crates/pwsh-cmdlets/tests/integration_tests.rs` — added pipeline coverage (`Get-Content | Select-Object -Skip -First`).
