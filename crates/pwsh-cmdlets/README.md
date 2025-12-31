# PowerShell Cmdlets Module

This module provides the built-in cmdlets for the PowerShell interpreter.

## Overview

The cmdlets module implements the five core cmdlets needed for the Week 6 MVP:

1. **Write-Output** - Output values to the pipeline
2. **Get-Process** - List system processes
3. **Where-Object** - Filter pipeline objects
4. **Select-Object** - Select object properties
5. **ForEach-Object** - Process each pipeline object

## Architecture

### Cmdlet Trait

All cmdlets implement the `Cmdlet` trait from `pwsh-runtime`:

```rust
pub trait Cmdlet: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError>;
}
```

### CmdletContext

Cmdlets receive a `CmdletContext` containing:
- Pipeline input (`Vec<Value>`)
- Named parameters (`HashMap<String, Value>`)
- Positional arguments (`Vec<Value>`)

### Registration

All cmdlets are registered using the `register_all` function:

```rust
pub fn register_all(registry: &mut pwsh_runtime::CmdletRegistry) {
    registry.register(Box::new(WriteOutputCmdlet));
    registry.register(Box::new(WhereObjectCmdlet));
    registry.register(Box::new(SelectObjectCmdlet));
    registry.register(Box::new(ForEachObjectCmdlet));
    registry.register(Box::new(GetProcessCmdlet));
}
```

## Cmdlets

### Write-Output

Outputs values to the pipeline.

**Syntax:**
```powershell
Write-Output <value>
<value> | Write-Output
```

**Examples:**
```powershell
Write-Output "Hello World"
Write-Output 42
$x | Write-Output
```

**Implementation:** `src/write_output.rs`

### Get-Process

Returns information about system processes.

**Syntax:**
```powershell
Get-Process
Get-Process -Name <name>
```

**Examples:**
```powershell
Get-Process
Get-Process -Name "chrome"
```

**Returns:** Objects with properties:
- `Name` - Process name
- `Id` - Process ID
- `CPU` - CPU time
- `WorkingSet` - Memory usage

**Note:** Currently returns mock data for demonstration. Real OS integration will be added in Phase 4.

**Implementation:** `src/get_process.rs`

### Where-Object

Filters pipeline objects based on conditions.

**Syntax:**
```powershell
<objects> | Where-Object -Property <name>
```

**Examples:**
```powershell
# Filter objects where Active property is truthy
$objects | Where-Object -Property "Active"
```

**Note:** Basic implementation filters by property truthiness. Script block support will be added in Week 9.

**Implementation:** `src/where_object.rs`

### Select-Object

Selects specific properties from objects or limits results.

**Syntax:**
```powershell
<objects> | Select-Object -Property <name>
<objects> | Select-Object -First <count>
```

**Examples:**
```powershell
Get-Process | Select-Object -Property "Name"
Get-Process | Select-Object -First 3
```

**Implementation:** `src/select_object.rs`

### ForEach-Object

Processes each object in the pipeline.

**Syntax:**
```powershell
<objects> | ForEach-Object -MemberName <name>
```

**Examples:**
```powershell
# Get the Name property from each object
$objects | ForEach-Object -MemberName "Name"
```

**Note:** Basic implementation accesses object members. Script block support will be added in Week 9.

**Implementation:** `src/foreach_object.rs`

## Testing

### Unit Tests

Each cmdlet has comprehensive unit tests in its source file.

Run cmdlet unit tests:
```bash
cargo test --package pwsh-cmdlets --lib
```

### Integration Tests

Integration tests in `tests/integration_tests.rs` verify end-to-end scenarios.

Run integration tests:
```bash
cargo test --package pwsh-cmdlets --test integration_tests
```

### All Tests

```bash
cargo test --package pwsh-cmdlets
```

## Adding New Cmdlets

To add a new cmdlet:

1. Create a new file in `src/` (e.g., `my_cmdlet.rs`)
2. Implement the `Cmdlet` trait:
   ```rust
   use pwsh_runtime::{Cmdlet, CmdletContext, RuntimeError, Value};

   pub struct MyCmdlet;

   impl Cmdlet for MyCmdlet {
       fn name(&self) -> &str {
           "My-Cmdlet"
       }

       fn execute(&self, context: CmdletContext) -> Result<Vec<Value>, RuntimeError> {
           // Implementation here
           Ok(context.pipeline_input)
       }
   }
   ```
3. Add unit tests in the same file
4. Export the cmdlet in `src/lib.rs`:
   ```rust
   mod my_cmdlet;
   pub use my_cmdlet::MyCmdlet;
   ```
5. Register it in the `register_all` function:
   ```rust
   pub fn register_all(registry: &mut pwsh_runtime::CmdletRegistry) {
       // ... existing registrations
       registry.register(Box::new(MyCmdlet));
   }
   ```

## Future Enhancements

### Week 9: Script Block Support
- Where-Object with `{ $_ -gt 5 }` syntax
- ForEach-Object with `{ $_ * 2 }` syntax
- `$_` automatic variable

### Phase 3: Advanced Parameters
- Multiple property selection in Select-Object
- Complex filtering in Where-Object
- -Last parameter for Select-Object

### Phase 4: Real System Integration
- Get-Process reads actual process data from OS
- Additional process cmdlets (Stop-Process, etc.)
- File system cmdlets (Get-ChildItem, etc.)

## Dependencies

- `pwsh-runtime` - For `Cmdlet` trait, `Value`, and error types
- `pwsh-lexer` (dev) - For testing
- `pwsh-parser` (dev) - For testing

## Documentation

Each cmdlet is documented with:
- Module-level documentation
- Function documentation
- Example usage
- Unit tests serving as examples

## Status

✅ Week 6 MVP Complete
- All 5 core cmdlets implemented
- All tests passing (13 unit + 9 integration)
- Ready for Phase 2 enhancements
