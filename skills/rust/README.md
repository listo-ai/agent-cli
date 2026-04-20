# Rust

## Do
- Use `cargo fmt`, `cargo clippy`, and the repo's make targets before merging.
- Prefer small crates, explicit types at boundaries, and `Result`-based error handling.
- Keep async entrypoints thin and move domain logic into testable functions.
- Reach for `thiserror` or `anyhow` instead of panics in production paths.

## Don't
- Don't add `unwrap()` or `expect()` in non-test code without a very strong reason.
- Don't hide IO or network work inside constructors.
- Don't bypass existing workspace patterns for tracing, config, or transport glue.

## Review Checklist
- Are errors contextual and actionable?
- Are new types serializable only where needed?
- Is the concurrency model obvious from the function boundaries?
