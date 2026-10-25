# Build & Verification Checklist

## Before Marking a Feature Complete

- [ ] `cargo check` passes with no errors
- [ ] `cargo check` passes with no new warnings
- [ ] Changes documented in `agent-docs/dev-status/STATUS.md`
- [ ] Agent docs updated in `agent-docs/` if APIs changed
- [ ] Commit made with conventional message (e.g., `feat:`, `docs:`, `fix:`)

## Full Build Verification (when ready to test)

- [ ] `cargo build` succeeds
- [ ] Application launches (if GUI changes)
- [ ] Menu hierarchy renders correctly in UI
- [ ] Hot-reload still works after menu changes

## Agent Update Rules

When the agent makes any change:
1. Update `agent-docs/dev-status/STATUS.md` immediately
2. If API signatures change, update `agent-docs/` documentation files
3. If menu structure changes, update `agent-docs/crate-guide.md` and `agent-docs/key-files.md`
4. Run `cargo check` before declaring done
5. Only then commit (if user requested)
