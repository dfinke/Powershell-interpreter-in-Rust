# Deferred Features Tracker

**Last Updated**: January 3, 2026  
**Current Week**: 14 (Phase 3 Complete)

## Overview

This document tracks all features that have been deferred during the implementation of the PowerShell interpreter. Each deferred feature includes the rationale for deferral and the target implementation timeline.

## Deferred Features by Category

### 1. Object System Enhancements (from Week 10)

#### PSObject Base Class
- **Original Target**: Week 10
- **Current Status**: Deferred
- **Rationale**: Basic objects work well with HashMap-based Value::Object. Advanced PSObject features not needed for MVP pipeline.
- **New Target**: Week 21-22 (with Advanced Features)
- **Impact**: Low - current implementation sufficient for object pipeline
- **Implementation Effort**: Medium (40-60 hours)

**What's Missing:**
- Extended type system (ETS)
- Type adapters
- Property sets
- Member sets
- PSObject metadata

**When Needed:**
- Advanced type system operations
- Module system (Week 27-30)
- Better .NET interop (future)

---

#### Method Calls on Objects
- **Original Target**: Week 10
- **Current Status**: Deferred
- **Rationale**: Not required for current cmdlets. Property access is sufficient.
- **New Target**: Week 22-23 (with Type System)
- **Impact**: Medium - some cmdlets may benefit
- **Implementation Effort**: Medium (30-50 hours)

**What's Missing:**
- `$obj.Method()` syntax
- Method invocation
- Method parameter binding
- Return value handling

**When Needed:**
- String manipulation methods (`.ToUpper()`, `.Substring()`)
- Array methods (`.Add()`, `.Remove()`)
- Custom object methods
- Advanced cmdlet scenarios

---

### 2. Cmdlet Advanced Features

#### ForEach-Object -Begin/-Process/-End Blocks (from Week 13)
- **Original Target**: Week 13
- **Current Status**: Deferred
- **Rationale**: Core transformation works. Advanced blocks are optimization/special cases.
- **New Target**: Week 18 (with Object Manipulation cmdlets)
- **Impact**: Low - convenience feature
- **Implementation Effort**: Small (20-30 hours)

**What's Missing:**
- `-Begin { }` - Execute once before pipeline
- `-Process { }` - Execute for each item (default)
- `-End { }` - Execute once after pipeline

**When Needed:**
- Initialization/cleanup operations
- Aggregation scenarios
- Advanced pipeline patterns
- Performance optimization

**Example:**
```powershell
1..10 | ForEach-Object -Begin { $sum = 0 } `
                       -Process { $sum += $_ } `
                       -End { Write-Output "Total: $sum" }
```

---

#### ForEach-Object -ArgumentList Parameter
- **Original Target**: Week 13
- **Current Status**: Deferred
- **Rationale**: Not needed for basic transformation scenarios
- **New Target**: Week 19 (with Utility cmdlets)
- **Impact**: Low - convenience feature
- **Implementation Effort**: Small (10-15 hours)

---

#### ForEach-Object -Parallel Switch
- **Original Target**: Not planned for MVP
- **Current Status**: Deferred to post-1.0
- **Rationale**: Complex feature, requires thread safety
- **New Target**: Post-1.0 (Future Features)
- **Impact**: High for performance
- **Implementation Effort**: Large (100+ hours)

---

### 3. Select-Object Advanced Features (from Week 12)

#### Calculated Properties
- **Original Target**: Week 12
- **Current Status**: Deferred
- **Rationale**: Basic property selection works. Calculated properties are advanced.
- **New Target**: Week 17 (with Object Manipulation)
- **Impact**: Medium - useful but not essential
- **Implementation Effort**: Medium (30-40 hours)

**What's Missing:**
```powershell
Select-Object Name, @{Name='NameLength'; Expression={$_.Name.Length}}
```

---

#### -Unique Parameter
- **Original Target**: Week 12
- **Current Status**: Deferred
- **Rationale**: Not needed for basic projection
- **New Target**: Week 17 (with Sort-Object)
- **Impact**: Low - convenience feature
- **Implementation Effort**: Small (10-20 hours)

---

#### -Skip Parameter
- **Original Target**: Week 12
- **Current Status**: Deferred
- **Rationale**: -First and -Last cover most use cases
- **New Target**: Week 18
- **Impact**: Low
- **Implementation Effort**: Small (5-10 hours)

---

### 4. System Integration (from Week 14)

#### Real OS Process Integration
- **Original Target**: Week 14
- **Current Status**: Deferred (using mock data)
- **Rationale**: Mock data proves pipeline architecture. Real OS integration is complex.
- **New Target**: Week 15-16 (File System cmdlets phase)
- **Impact**: High - required for production use
- **Implementation Effort**: Medium (40-50 hours)

**What's Missing:**
- Real process enumeration from OS
- Cross-platform process APIs (Windows/Linux/macOS)
- Additional process properties (Threads, Handles, Path, etc.)
- Process control (Start/Stop)

**Implementation Requirements:**
- Use `sysinfo` or `procfs` crate
- Handle platform differences
- Error handling for privileged processes
- Performance for large process lists

---

#### Additional Process Properties
- **Original Target**: Week 14
- **Current Status**: Deferred
- **Rationale**: 4 properties sufficient for MVP
- **New Target**: Week 15-16
- **Impact**: Medium
- **Implementation Effort**: Small-Medium (20-30 hours)

**Properties to Add:**
- Path
- CommandLine
- StartTime
- Threads
- Handles
- PrivateMemorySize
- VirtualMemorySize
- ProcessorAffinity

---

### 5. Language Features (Not Yet Implemented)

#### Range Operator (1..10)
- **Original Target**: Not specified
- **Current Status**: Using array literals
- **Rationale**: Array literals work, range operator is syntactic sugar
- **New Target**: Week 21 (Loops phase)
- **Impact**: Medium - very useful feature
- **Implementation Effort**: Small (15-20 hours)

**What's Missing:**
```powershell
1..10           # Range of numbers
'a'..'z'        # Character range
10..1           # Reverse range
```

---

#### Loops
- **Original Target**: Week 21-22
- **Current Status**: Not implemented
- **Rationale**: Pipeline covers iteration for MVP
- **New Target**: Week 21-22 (as planned)
- **Impact**: High - essential language feature
- **Implementation Effort**: Large (80 hours)

**Missing Loop Types:**
- `foreach ($item in $collection) { }`
- `while ($condition) { }`
- `do { } while ($condition)`
- `for ($i=0; $i -lt 10; $i++) { }`
- `break` and `continue` statements

---

#### Error Handling (try/catch/finally)
- **Original Target**: Week 23-24
- **Current Status**: Not implemented
- **Rationale**: Basic errors work, structured error handling is advanced
- **New Target**: Week 23-24 (as planned)
- **Impact**: High - essential for production
- **Implementation Effort**: Large (80 hours)

---

#### Advanced Array Features
- **Original Target**: Week 25-26
- **Current Status**: Basic arrays work
- **Rationale**: Array literals sufficient for MVP
- **New Target**: Week 25-26 (as planned)
- **Impact**: Medium
- **Implementation Effort**: Medium (40-50 hours)

**What's Missing:**
- Array indexing: `$arr[0]`, `$arr[-1]`
- Array slicing: `$arr[1..3]`
- Array operators: `+`, `+=`, `-contains`, etc.
- Multi-dimensional arrays

---

#### Type System
- **Original Target**: Week 25-26
- **Current Status**: Dynamic typing only
- **Rationale**: Dynamic typing sufficient for MVP
- **New Target**: Week 25-26 (as planned)
- **Impact**: High - important for robustness
- **Implementation Effort**: Large (60-80 hours)

**What's Missing:**
- Type casting: `[int]$value`, `[string]$value`
- Type constraints on parameters: `param([int]$count)`
- Type validation
- Type conversion rules
- Accelerators: `[PSCustomObject]`, `[ordered]`, etc.

---

## Implementation Priority

### High Priority (Weeks 15-20)
These are essential for Beta milestone and should be implemented next:

1. **Real OS Process Integration** (Week 15-16)
   - Critical for production use
   - Enables real system automation
   - Foundation for other system cmdlets

2. **File System Cmdlets** (Week 15-16)
   - Get-ChildItem, Get-Content, Set-Content
   - Essential for practical scripting
   - High user value

3. **Object Manipulation Cmdlets** (Week 17-18)
   - Sort-Object, Group-Object, Measure-Object
   - Complete object pipeline capabilities
   - Include: Calculated Properties for Select-Object

4. **Utility Cmdlets** (Week 19-20)
   - Write-Host, Read-Host, Get-Date
   - Format-Table, Format-List
   - User interaction and output formatting

---

### Medium Priority (Weeks 21-26)
Important for Beta completion:

1. **Loops** (Week 21-22)
   - foreach, while, do-while, for
   - break/continue
   - Range operator (1..10)

2. **Error Handling** (Week 23-24)
   - try/catch/finally
   - throw
   - $Error variable
   - -ErrorAction parameter

3. **Advanced Object Features** (Week 22-23)
   - Method calls
   - PSObject base class
   - ForEach-Object -Begin/-Process/-End

4. **Collections & Types** (Week 25-26)
   - Array indexing and slicing
   - Type casting and constraints
   - Hashtable improvements

---

### Low Priority (Weeks 27-36)
Polish and advanced features:

1. **Module System** (Week 27-30)
2. **Performance Optimization** (Week 31-32)
3. **Developer Experience** (Week 33-34)
4. **Documentation & Release** (Week 35-36)

---

### Post-1.0 Features
Deferred beyond initial release:

1. **ForEach-Object -Parallel**
   - Requires thread safety
   - Complex implementation
   - High performance value

2. **Advanced Type System Features**
   - .NET type integration
   - Custom type accelerators
   - Type adapters

3. **Bytecode Compilation**
   - Performance optimization
   - JIT compilation
   - VM implementation

4. **Language Server Protocol (LSP)**
   - Editor integration
   - IntelliSense support
   - Debugging protocol

5. **Remote Execution**
   - PowerShell remoting
   - Session management
   - Serialization

---

## Beta Success Gap Analysis

**Target (Week 26):** 30+ cmdlets

**Current (Week 14):** 5 cmdlets
- Write-Output
- Get-Process
- Where-Object
- Select-Object
- ForEach-Object

**Gap:** 25 cmdlets needed

**Proposed Cmdlet Additions (Weeks 15-26):**

**File System (6 cmdlets):**
1. Get-ChildItem
2. Get-Content
3. Set-Content
4. Test-Path
5. New-Item
6. Remove-Item

**Object Manipulation (4 cmdlets):**
7. Sort-Object
8. Group-Object
9. Measure-Object
10. Compare-Object

**Utility (6 cmdlets):**
11. Write-Host
12. Read-Host
13. Get-Date
14. Format-Table
15. Format-List
16. Out-String

**String Manipulation (4 cmdlets):**
17. Select-String
18. Out-File
19. Get-Random
20. ConvertTo-Json
21. ConvertFrom-Json

**Additional (9 cmdlets to reach 30+):**
22. Set-Variable
23. Get-Variable
24. Clear-Host
25. Get-Member
26. New-Object
27. Add-Member
28. Remove-Variable
29. Join-Path
30. Split-Path

**Total:** 30 cmdlets (5 current + 25 new) = **30 cmdlets by Week 26** ✅

---

## Tracking Status

| Category | Total Features | Deferred | In Progress | Completed |
|----------|---------------|----------|-------------|-----------|
| Object System | 2 | 2 | 0 | 0 |
| Cmdlet Features | 6 | 6 | 0 | 0 |
| System Integration | 2 | 2 | 0 | 0 |
| Language Features | 5 | 5 | 0 | 0 |
| **Total** | **15** | **15** | **0** | **0** |

---

## Conclusion

We have identified 15 deferred features across 5 categories. The implementation is strategically focused on core pipeline functionality first, with advanced features deferred to later phases. This approach has allowed us to achieve the Object Pipeline milestone on schedule.

**Key Priorities:**
1. Complete Phase 4 (Built-in Cmdlets) - Weeks 15-20
2. Implement critical deferred features during Phases 5-6
3. Polish and optimize in Phase 7

**Status:** All deferrals are intentional and well-documented. No technical debt identified.
