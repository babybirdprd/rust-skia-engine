# Documentation Guidelines

How to write and maintain documentation for Director Engine.

## Documentation Structure

See [DOCS_INDEX.md](../../DOCS_INDEX.md) for the canonical structure.

| Location | Purpose |
|----------|---------|
| `docs/user/` | End-user docs (Rhai scripting) |
| `docs/architecture/` | System design, internals |
| `docs/contributing/` | Contributor guides |
| `docs/specs/` | Design specifications |
| `crates/*/README.md` | Per-crate documentation |

---

## When to Update Docs

| Change | Update |
|--------|--------|
| New Rhai API | `docs/user/scripting-guide.md` |
| Architecture change | `docs/architecture/overview.md` |
| New crate | `crates/<name>/README.md` |
| New feature spec | `docs/specs/<name>.md` (use template) |

---

## Writing Style

1. **Be concise** — Keep paragraphs short
2. **Use tables** — For structured data
3. **Use code blocks** — For all code examples
4. **Use consistent formatting** — Headers, lists, emphasis

### AI Agent Considerations

- Keep file sizes manageable for context windows
- Use tables for quick scanning
- Maintain the `DOCS_INDEX.md` for navigation
- All public APIs should have examples

---

## Spec Template

New design specs should use [_TEMPLATE.md](./_TEMPLATE.md) in `docs/specs/`.

Specs have a lifecycle:
- **Draft** — Initial proposal
- **In Review** — Under discussion
- **Approved** — Ready for implementation
- **Implemented** — Merged to main
- **Superseded** — Replaced by newer spec

---

## Document Freshness

Update the freshness table in `DOCS_INDEX.md` when making major updates:

```markdown
| Document | Last Major Update | Owner |
|----------|-------------------|-------|
| `AGENTS.md` | 2024-12 | babybirdprd |
```

---

*Documentation is a feature. Treat it with the same care as code.*
