# Cache Testing Rule

## Rule: Always Run Miri Tests After Cache Changes

Whenever any file in the `src/cache` directory is modified, you **MUST** run miri tests to verify memory safety and detect potential data races.

## Command

```bash
make test-miri
```

Or directly:

```bash
cargo +nightly miri test
```

## Rationale

The cache module (`src/cache`) contains:
- Concurrent data structures
- Atomic operations
- Unsafe code with `MaybeUninit` and `assume_init_ref()`
- Shared mutable state across threads

Miri is a tool that can detect:
- Undefined behavior
- Data races
- Invalid memory accesses
- Violations of Rust's safety rules

## When This Rule Applies

- ✅ Any modification to files in `src/cache/`
- ✅ Changes to atomic operations
- ✅ Changes to unsafe code blocks
- ✅ Logic changes in `get_data()`, `update()`, or `index()` methods
- ✅ Structural changes to `Cache` or `Item` types

## Enforcement

This rule is enforced by:
1. AI assistant (automatic execution after cache changes)
2. Developer responsibility (manual verification)
3. CI/CD pipeline (recommended to add miri checks)

## Example Workflow

1. Make changes to `src/cache/mod.rs`
2. Run `make test-miri`
3. Verify all tests pass with miri
4. If miri reports issues, fix them before proceeding
5. Only then commit the changes

---

*Last updated: When miri testing was configured for the project*