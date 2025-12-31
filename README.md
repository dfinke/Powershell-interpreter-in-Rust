# PowerShell Interpreter

A modern PowerShell interpreter implementation written in Rust, featuring the signature object-based pipeline architecture that makes PowerShell unique.

## 🚀 Project Status

**Current Phase**: Investigation & Planning  
**Target MVP**: Week 6 (Object pipeline with 5 cmdlets)  
**Target 1.0**: Week 36 (Production-ready interpreter)

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

- **[INVESTIGATION.md](INVESTIGATION.md)** - Comprehensive analysis of language choice, architecture, and implementation strategy
- **[TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md)** - Detailed technical design with code examples and architecture diagrams
- **[ROADMAP.md](ROADMAP.md)** - Week-by-week implementation plan with milestones
- **[QUICKSTART.md](QUICKSTART.md)** - Get started building in 30 minutes

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
- Function definitions (basic)
- Script blocks (`{ code }`)

**Essential Cmdlets:**
1. `Write-Output` - Display output
2. `Get-Process` - List processes
3. `Where-Object` - Filter objects
4. `Select-Object` - Select properties
5. `ForEach-Object` - Transform objects

**MVP Demo Script:**
```powershell
# Demonstrate object pipeline
$procs = Get-Process
$filtered = $procs | Where-Object { $_.CPU -gt 5 }
$selected = $filtered | Select-Object Name, CPU
$selected | ForEach-Object { Write-Output "$($_.Name): $($_.CPU)" }

# Demonstrate functions
function Get-TopProcesses($count) {
    Get-Process | Where-Object { $_.CPU -gt 0 } | Select-Object -First $count
}

Get-TopProcesses 5
```

## 🗺️ Phased Implementation

### Phase 0: Foundation (Weeks 1-2)
- Project setup, basic lexer, simple REPL

### Phase 1: Core Language (Weeks 3-6)
- Parser, AST, evaluator, basic pipeline
- **MVP Delivery** ✅

### Phase 2: Functions & Scope (Weeks 7-9)
- Function definitions, parameter binding, closures

### Phase 3: Object Pipeline (Weeks 10-14)
- Object system, core cmdlets, full pipeline

### Phase 4: Built-in Cmdlets (Weeks 15-20)
- File system, object manipulation, utility cmdlets

### Phase 5: Advanced Features (Weeks 21-26)
- Loops, error handling, arrays, hashtables

### Phase 6: Module System (Weeks 27-30)
- Module loading, Import-Module, manifests

### Phase 7: Polish & Optimization (Weeks 31-36)
- Performance tuning, documentation, 1.0 release

See [ROADMAP.md](ROADMAP.md) for detailed week-by-week plan.

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable)
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

### MVP Success (Week 6)
- [ ] 5 cmdlets working
- [ ] Object pipeline functional
- [ ] 10 example scripts execute
- [ ] 80%+ test coverage

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