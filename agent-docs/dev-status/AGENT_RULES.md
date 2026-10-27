# Agent Rules - Dev Documentation Updates

## When to Update Dev Status

The agent MUST update `agent-docs/dev-status/STATUS.md` when:

1. **Any code file is modified** — add entry to "File Changes" table
2. **A feature is completed** — move item to "Completed" section
3. **A new feature is started** — move item to "In Progress" section
4. **Build status changes** — update "Build Status" line
5. **API signatures change** — also update relevant `agent-docs/*.md` files

## Checklist Before Declaring Done

The agent must complete ALL of the following before telling the user a task is done:

- [ ] `cargo check` runs without errors
- [ ] `cargo check` runs without new warnings
- [ ] `agent-docs/dev-status/STATUS.md` is updated
- [ ] Agent documentation (`agent-docs/`) is updated if APIs changed

## Commit Convention

If the user requests a commit, use conventional commits:
- `feat:` for new features
- `docs:` for documentation changes
- `fix:` for bug fixes
- `refactor:` for code refactoring

Example: `feat: add hierarchical menu system with recursive rendering`
