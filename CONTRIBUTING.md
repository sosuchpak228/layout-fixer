# Contributing

Please open an issue with the editor, Windows version, a minimal reproduction, and whether the target runs elevated. Do not attach private selected text or clipboard data.

Before a PR, run `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked` on Windows. Keep unsupported editors non-destructive: never paste stale clipboard content or guess that a selection is editable. Add narrow regression tests for layout changes.
