# PowerShell Interpreter

A modern PowerShell interpreter implementation written in Rust, featuring the signature object-based pipeline architecture that makes PowerShell unique.

## 🚀 Project Status

**Current Phase**: Week 14 Complete - Object Pipeline Milestone Reached! 🎉🎉🎉  
**Latest Achievement**: Complete end-to-end object pipeline with Get-Process, filtering, projection, and transformation  
**Target 1.0**: Week 36 (Production-ready interpreter)

### What Works Now (Week 14 - Phase 3 Complete!)
- ✅ Complete lexer and parser
- ✅ Runtime evaluation engine
- ✅ **Object-based pipeline execution (MILESTONE!)**
- ✅ 5 core cmdlets (Write-Output, Get-Process, Where-Object, Select-Object, ForEach-Object)
- ✅ Phase 4 started: File system cmdlets (Get-ChildItem, Get-Content, Set-Content, Test-Path, New-Item, Remove-Item)
- ✅ Week 17 complete: Object manipulation cmdlets **Sort-Object** and **Group-Object**
- ✅ **Get-Process with process objects**
- ✅ **Where-Object with script block filtering**
- ✅ **Select-Object with property projection**
- ✅ **ForEach-Object with transformation**
- ✅ **Complete pipeline: Get-Process | Where-Object | Select-Object**
- ✅ Script block support in cmdlets
- ✅ Array literals (@(items))
- ✅ Variables and expressions
- ✅ String interpolation
- ✅ Control flow (if/else)
- ✅ User-defined functions
- ✅ Parameter binding with defaults
- ✅ Return statements
- ✅ Scope qualifiers ($global:, $local:, $script:)
- ✅ Advanced scope management
- ✅ Closures (basic)
- ✅ Script blocks as first-class values
- ✅ Pipeline integration with $_
- ✅ Script block execution
- ✅ Hashtable creation (@{key=value})
- ✅ Property access ($obj.Property)
- ✅ Interactive REPL

**Try it now:**
```bash
cargo run -p pwsh-cli
```

### Week 17 examples

- `examples/week17_sort_object.ps1`
- `examples/week17_group_object.ps1`

## 📋 Overview

This project aims to create a from-scratch PowerShell interpreter that:
- ✅ Implements PowerShell's **object pipeline** (not text-based like traditional shells)
- ✅ Supports core PowerShell language features
- ✅ Provides a modern, fast, and memory-safe implementation using Rust
- ✅ Maintains compatibility with PowerShell syntax and semantics
- ✅ Offers excellent error messages and developer experience

## 🎯 Why This Project?

### The Object Pipeline Advantage

Unlike traditional shells (bash, zsh) that pass text between commands, PowerShell passes **structured objects**:

```powershell
# Traditional shells (text-based):
ps aux | grep chrome | awk '{print $2}'

# PowerShell (object-based):
Get-Process | Where-Object {$_.Name -eq "chrome"} | Select-Object Id
```

This makes data manipulation more intuitive, powerful, and less error-prone.

### Why Rust?

- **Memory Safety**: Compile-time guarantees prevent entire classes of bugs
- **Performance**: Near-C performance for fast command execution
- **Modern Tooling**: Excellent package manager (Cargo) and testing framework
- **Growing Ecosystem**: Rich libraries for parsing, CLI, and systems programming
- **Community**: Strong trend toward systems tools in Rust (ripgrep, bat, fd, etc.)

## 📚 Documentation

### Planning & Strategy
- **[ROADMAP.md](ROADMAP.md)** - Week-by-week implementation plan with milestones
- **[NEXT_STEPS.md](NEXT_STEPS.md)** - Detailed breakdown of Weeks 15-26 into small, testable chunks
- **[DEFERRED_FEATURES.md](DEFERRED_FEATURES.md)** - Tracking of deferred features with rationale and timeline

### Technical Documentation
- **[INVESTIGATION.md](INVESTIGATION.md)** - Comprehensive analysis of language choice, architecture, and implementation strategy
- **[TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md)** - Detailed technical design with code examples and architecture diagrams
- **[QUICKSTART.md](QUICKSTART.md)** - Get started building in 30 minutes

### Progress Tracking
- **[PHASE_3_COMPLETE.md](PHASE_3_COMPLETE.md)** - Object Pipeline milestone completion summary
- **[WEEK_*_SUMMARY.md](.)** - Weekly progress summaries (Weeks 5-14)

## 🏗️ Architecture

```
Source Code → Lexer → Tokens → Parser → AST → Evaluator → Result
                                                   ↓
                                           Runtime Environment
                                           (Scope, Cmdlets, Objects)
```

### Module Structure

```
powershell-interpreter/
├── crates/
│   ├── pwsh-lexer/      # Tokenization
│   ├── pwsh-parser/     # Parsing & AST
│   ├── pwsh-runtime/    # Execution environment
│   ├── pwsh-cmdlets/    # Built-in cmdlets
│   └── pwsh-cli/        # REPL & CLI
├── examples/            # Example scripts
└── docs/               # Documentation
```

## 🎯 Minimal Viable Product (MVP)

**Timeline**: 6 weeks  
**Goal**: Demonstrate object pipeline with essential cmdlets

### MVP Scope

**Language Features:**
- Variables (`$x = value`)
- Basic operators (`+`, `-`, `*`, `/`, `-eq`, `-ne`, `-gt`, `-lt`)
- String literals with interpolation (`"Hello $name"`)
- Pipeline operator (`|`)
- **Function definitions with parameters**
- **Return statements**
- **Default parameter values**
- Script blocks (`{ code }`)

**Essential Cmdlets:**
1. `Write-Output` - Display output
2. `Get-Process` - List processes
3. `Where-Object` - Filter objects
4. `Select-Object` - Select properties
5. `ForEach-Object` - Transform objects

**MVP Demo Script:**
```powershell
# Demonstrate script blocks in cmdlets (Week 11)
@(1,2,3,4,5) | Where-Object { $_ -gt 2 }   # 3, 4, 5
@(1,2,3,4,5) | ForEach-Object { $_ * 2 }   # 2, 4, 6, 8, 10
Get-Process | Where-Object { $_.CPU -gt 10 }

# Demonstrate object system (Week 10)
$person = @{Name="John"; Age=30; City="NYC"}
$person.Name   # John
$person.Age    # 30

# Demonstrate array literals (Week 11)
$arr = @(1, 2, 3, 4, 5)
$arr | Write-Output

# Demonstrate script blocks (Week 9)
$filter = { $_ -gt 5 }
1 | { $_ + 5 }        # Output: 6
10 | { $_ * 2 }       # Output: 20

# Demonstrate scope qualifiers (Week 8)
$global:x = 5
function Test {
    $local:y = 10
    $x + $y
}
Test  # Returns 15

# Demonstrate functions (Week 7)
function Add($a, $b) {
    return $a + $b
}

function Double($x) {
    $x * 2
}

$sum = Add 5 10
Write-Output "5 + 10 = $sum"

$doubled = Double 21
Write-Output "21 * 2 = $doubled"

# Demonstrate object pipeline (Week 6)
$procs = Get-Process
$filtered = $procs | Where-Object { $_.CPU -gt 5 }
$selected = $filtered | Select-Object Name, CPU
$selected | ForEach-Object { Write-Output "$($_.Name): $($_.CPU)" }
```

## 🗺️ Phased Implementation

### Phase 0: Foundation (Weeks 1-2)
- Project setup, basic lexer, simple REPL

### Phase 1: Core Language (Weeks 3-6)
- Parser, AST, evaluator, basic pipeline
- **MVP Delivery** ✅

### Phase 2: Functions & Scope (Weeks 7-9)
- Function definitions, parameter binding, closures, script blocks
- **Week 7 Complete** ✅
- **Week 8 Complete** ✅
- **Week 9 Complete** ✅

### Phase 3: Object Pipeline (Weeks 10-14)
- Object system, core cmdlets, full pipeline
- **Week 10 Complete** ✅
- **Week 11 Complete** ✅
- **Week 12 Complete** ✅
- **Week 13 Complete** ✅
- **Week 14 Complete** ✅
- **🎉 OBJECT PIPELINE MILESTONE REACHED! 🎉**

### Phase 4: Built-in Cmdlets (Weeks 15-20) ⬅️ CURRENT PHASE
- **Week 15-16**: File system cmdlets (Get-ChildItem, Get-Content, Set-Content, etc.)
- **Week 17-18**: Object manipulation (Sort-Object, Group-Object, Measure-Object, Compare-Object)
- **Week 19-20**: Utility cmdlets (Write-Host, Read-Host, Format-Table, Get-Date, etc.)
- **Target**: 20+ new cmdlets, real OS integration
- **See [NEXT_STEPS.md](NEXT_STEPS.md) for detailed chunk-by-chunk breakdown**

### Phase 5: Advanced Features (Weeks 21-26) - Path to Beta
- **Week 21-22**: Loops (foreach, while, for) + Range operator + Method calls
- **Week 23-24**: Error handling (try/catch/finally, throw, $Error)
- **Week 25-26**: Collections & Types (array indexing, type casting)
- **Beta Milestone (Week 26)**: 30+ cmdlets, all core features, 100 examples

### Phase 6: Module System (Weeks 27-30)
- Module loading, Import-Module, manifests

### Phase 7: Polish & Optimization (Weeks 31-36)
- Performance tuning, documentation, 1.0 release

See [ROADMAP.md](ROADMAP.md) for detailed week-by-week plan.

## 🚀 Quick Start

### Prerequisites

- Rust 1.64 or later (workspace inheritance support required)
- Recommended: install toolchain components used by CI
    - `rustup component add clippy rustfmt`
- Git
- Code editor (VS Code + rust-analyzer recommended)

### Get Started in 30 Minutes

Follow the [QUICKSTART.md](QUICKSTART.md) guide to:
1. Set up the project structure
2. Create a basic lexer
3. Build a simple REPL
4. Run your first tokenization

```bash
# Clone repository
git clone <repo-url>
cd powershell-interpreter

# Follow QUICKSTART.md to build your first prototype
# In just 30 minutes you'll have a working tokenizer!
```

## 🧪 Development Workflow

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the REPL
cargo run -p pwsh-cli

# Check code quality
cargo clippy

# Format code
cargo fmt

# Generate documentation
cargo doc --open
```

## 🎓 Learning Resources

### Interpreter/Compiler Design
- **Crafting Interpreters** by Robert Nystrom (⭐ highly recommended)
- **Writing An Interpreter In Go** by Thorsten Ball
- **Engineering a Compiler** by Cooper & Torczon

### Rust for Language Implementation
- The Rust Book (official)
- **Programming Rust** by Blandy & Orendorff
- Study: RustPython, tree-sitter source code

### PowerShell References
- PowerShell Language Specification (official)
- PowerShell Core source code
- **PowerShell in Depth** by Don Jones

### Similar Projects for Inspiration
- **Nushell** - Rust-based shell with structured data
- **RustPython** - Python interpreter in Rust
- **Deno** - V8-based runtime (architecture reference)

## 🤝 Contributing

We welcome contributions! See our contribution guidelines:

### Getting Started
1. Read [INVESTIGATION.md](INVESTIGATION.md) to understand the project
2. Review [TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md) for architecture
3. Check [ROADMAP.md](ROADMAP.md) for current priorities
4. Follow [QUICKSTART.md](QUICKSTART.md) to set up development

### Development Principles
- **Test-Driven Development (TDD)**: Write tests first
- **Minimal Changes**: Make surgical, focused changes
- **Documentation**: Update docs with code changes
- **Code Quality**: Run clippy and fmt before committing

## 📊 Success Metrics

### MVP Success (Week 6) - ✅ ACHIEVED
- [x] 5 cmdlets working
- [x] Object pipeline functional
- [x] 10 example scripts execute
- [x] 80%+ test coverage

### Week 7 Success - ✅ ACHIEVED
- [x] Function definitions working
- [x] Parameter binding with defaults
- [x] Return statements
- [x] 156 total tests passing
- [x] 100% test coverage for functions

### Week 8 Success - ✅ ACHIEVED
- [x] Scope qualifiers ($global:, $local:, $script:) working
- [x] Advanced scope management
- [x] Closures (basic) functioning
- [x] 186 total tests passing
- [x] 22 new tests added
- [x] 100% test coverage for scope features

### Week 9 Success - ✅ ACHIEVED
- [x] Script blocks as first-class values
- [x] Script block creation and assignment
- [x] Pipeline integration with script blocks
- [x] `$_` automatic variable support
- [x] 89 total tests passing
- [x] 8 new tests added
- [x] 100% test coverage for script block features

### Week 10 Success - ✅ ACHIEVED
- [x] Hashtable creation syntax (@{key=value})
- [x] Property access working ($obj.Property)
- [x] Object system foundations
- [x] 94 total tests passing
- [x] 13 new tests added
- [x] 100% test coverage for object features

### Week 11 Success - ✅ ACHIEVED  
- [x] Array literal syntax (@(items))
- [x] Array evaluation and unrolling in pipelines
- [x] Where-Object with script block filtering
- [x] ForEach-Object with script block transformation
- [x] Script block support in cmdlets
- [x] Success criteria verified: `@(1,2,3,4,5) | Where-Object { $_ -gt 2 }` ✅

### Week 12 Success - ✅ ACHIEVED
- [x] Select-Object cmdlet with property projection
- [x] -First and -Last parameters working
- [x] Multiple property selection
- [x] Success criteria verified: `$objects | Select-Object Name, CPU` ✅

### Week 13 Success - ✅ ACHIEVED
- [x] ForEach-Object cmdlet with script block transformation
- [x] -MemberName parameter for property extraction
- [x] Pipeline integration with $_
- [x] Success criteria verified: `@(1..10) | ForEach-Object { $_ * 2 }` ✅

### Week 14 Success - ✅ ACHIEVED - 🎉 MILESTONE! 🎉
- [x] Get-Process cmdlet with process objects
- [x] -Name parameter filtering
- [x] Complete object pipeline working
- [x] 235 total tests passing
- [x] 3 new success criteria tests added
- [x] Success criteria verified: `Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU` ✅
- [x] **OBJECT PIPELINE MILESTONE REACHED!**

### Beta Success (Week 26)
- [ ] 30+ cmdlets
- [ ] All core language features
- [ ] 100 example scripts
- [ ] 85%+ test coverage

### 1.0 Release (Week 36)
- [ ] 50+ cmdlets
- [ ] Production quality
- [ ] Complete documentation
- [ ] 90%+ test coverage
- [ ] Performance benchmarks met

## 🔮 Future Vision

### Post-1.0 Features
- Bytecode compilation for performance
- JIT optimization for hot paths
- Parallel pipeline execution (`-Parallel`)
- Language Server Protocol (LSP) support
- WebAssembly compilation
- Package management system
- Remote execution capabilities

## 📄 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- PowerShell team at Microsoft for the original implementation
- Rust community for excellent tools and libraries
- Robert Nystrom for "Crafting Interpreters"
- All contributors and supporters

## 📞 Contact & Support

- **Issues**: Use GitHub Issues for bug reports and features
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: See docs/ directory

---

**Ready to build?** Start with [QUICKSTART.md](QUICKSTART.md) and you'll have a working lexer in 30 minutes! 🚀