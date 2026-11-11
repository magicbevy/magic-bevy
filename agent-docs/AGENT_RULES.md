# Agent Rules - Dev Documentation Updates

## Checklist Before Declaring Done

The agent must complete ALL of the following before telling the user a task is done:

- [ ] `cargo check` runs without errors
- [ ] `cargo check` runs without new warnings

## Commit Convention

If the user requests a commit, use conventional commits:
- `feat:` for new features
- `docs:` for documentation changes
- `fix:` for bug fixes
- `refactor:` for code refactoring

Example: `feat: add hierarchical menu system with recursive rendering`
