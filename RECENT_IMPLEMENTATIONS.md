# Recent Implementations

This file tracks the most recent features implemented in this repo (what changed, where, and how it was verified).

## 2026-01-04

### Week 17 — Sort-Object & Group-Object (Object Manipulation)

**Implemented (PowerShell-aligned)**: `Sort-Object` and `Group-Object` with pipeline support, property-based operation, and deterministic results.

**Sort-Object behavior**:
- Sorts pipeline input (or positional args when not in a pipeline)
- `-Property` supports a single property or an array of properties
- `-Descending` supported (expects explicit boolean in this interpreter)
- Comparison: nulls first, numeric compare when possible, else case-insensitive string compare

**Group-Object behavior**:
- Groups by value (default) or by `-Property` (single or array)
- Supports positional property list *when used in a pipeline* (e.g. `$items | Group-Object Name, Extension`)
- `-NoElement` omits the `Group` array
- `-AsHashTable` returns a single hashtable-like object mapping group name → group info

**Parser compatibility fix**:
- Named parameter values now support **bare words** (PowerShell style) when syntactically unambiguous.
	- Example: `Sort-Object -Property CPU` treats `CPU` as a string literal instead of attempting to call a function/cmdlet named `CPU`.

**Supporting improvements**:
- `Get-ChildItem` now includes an `Extension` property (e.g. `.rs`) to support grouping examples.
- REPL tab-completion list includes `Sort-Object` and `Group-Object` via `pwsh_cmdlets::cmdlet_names()`.
- Added Week 17 example scripts.

**Files changed**:
- `crates/pwsh-cmdlets/src/sort_object.rs` — new cmdlet + unit tests
- `crates/pwsh-cmdlets/src/group_object.rs` — new cmdlet + unit tests
- `crates/pwsh-cmdlets/src/lib.rs` — cmdlet registration + completion names
- `crates/pwsh-cmdlets/src/get_childitem.rs` — add `Extension` property (plus minor clippy cleanup)
- `crates/pwsh-cmdlets/tests/integration_tests.rs` — end-to-end Week 17 coverage
- `crates/pwsh-parser/src/parser.rs` — bare-word named parameter value parsing
- `examples/week17_sort_object.ps1`, `examples/week17_group_object.ps1` — new example scripts
- `README.md` — docs update for Week 17

**Verification**:
- `cargo test` (workspace)
- `cargo fmt --all -- --check` (workspace)
- `cargo clippy --all-targets --all-features -- -D warnings` (workspace)

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
