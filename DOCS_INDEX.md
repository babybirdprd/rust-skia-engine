# Director Engine Documentation Index

> 🤖 **For AI Agents:** This file is the canonical navigation index.  
> Load this first to understand the documentation structure.

## Quick Reference

| Audience | Entry Point |
|----------|-------------|
| **AI Agents** | [AGENTS.md](AGENTS.md) |
| **Users (Rhai scripting)** | [docs/user/scripting-guide.md](docs/user/scripting-guide.md) |
| **Contributors** | [docs/contributing/development.md](docs/contributing/development.md) |
| **Architecture** | [docs/architecture/overview.md](docs/architecture/overview.md) |

## Directory Structure

```
docs/
├── user/                  # End-user documentation (Rhai scripting)
│   ├── getting-started.md
│   └── scripting-guide.md # Complete Rhai API reference
├── architecture/          # System design and vision
│   ├── overview.md        # Engine internals, scene graph, rendering pipeline
│   └── roadmap.md         # Development trajectory and milestones
├── contributing/          # Contributor documentation
│   ├── development.md     # Build guide, testing, setup
│   └── documentation.md   # How to write and maintain docs
└── specs/                 # Design specifications (informal RFCs)
    ├── _TEMPLATE.md       # Template for new specs
    └── *.md               # Feature specs (SAM3, Templates, Rhai stdlib)

crates/
├── director-core/README.md    # Core engine: rendering, layout, animation, scripting
├── director-cli/README.md     # CLI video renderer
├── director-schema/README.md  # Schema type definitions
└── director-pipeline/README.md # Asset pipeline utilities
```

## Invariants

1. **All documentation lives in `docs/` or crate-level `README.md` files**
2. **No orphan markdown files in root** except: `README.md`, `AGENTS.md`, `DOCS_INDEX.md`, `CHANGELOG.md`
3. **All specs follow the template** in `docs/specs/_TEMPLATE.md`
4. **Rhai API changes** → update `docs/user/scripting-guide.md`
5. **Architecture changes** → update `docs/architecture/overview.md`
6. **New crate** → add `crates/<name>/README.md`

## Issue Tracking

Issues are tracked in **Grits** (not markdown files).

```bash
gr list              # View all open issues
gr advisory next     # Get AI-recommended next task
gr show <ID>        # View issue details
```

## Document Freshness

| Document | Last Major Update | Owner |
|----------|-------------------|-------|
| `AGENTS.md` | 2024-12 | babybirdprd |
| `docs/user/scripting-guide.md` | 2024-12 | babybirdprd |
| `docs/architecture/overview.md` | 2024-12 | babybirdprd |

---

*This index is the source of truth for documentation navigation.*
