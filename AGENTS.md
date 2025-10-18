# Project Orientation
- Library targets `embedded-hal` 1.0 in `no_std` environments; keep APIs allocator-free and resilient to HAL timing quirks.
- Supported controllers/adapters: HD44780 variants (PCF8574T, Adafruit MCP23008, dual-controller), AiP31068, and ST7032i. Each has dedicated hardware and action layers—reuse those abstractions instead of bypassing them.
- Error paths must use `CharacterDisplayError`; extend the enum only when the behaviour is externally observable.

# Development Workflow
- Touch both the high-level facade in `src/lib.rs` and the appropriate driver module when introducing a new capability.
- Maintain compatibility with optional features (`defmt`, `ufmt`). Ensure new code compiles when these features are disabled.
- Respect existing timing expectations; use the provided delays rather than busy waits or sleeping loops.

# Testing Requirements
- **Unit tests are mandatory for every behavioural change.** Extend the existing `embedded-hal-mock` based tests to cover new I²C transactions, timing assumptions, and failure paths.
- When fixing a bug, first reproduce it with a failing unit test, then implement the fix. Do not remove tests without a replacement.
- Run `cargo test` in the root crate before handing off any change set. If tests cannot run in your environment, explain why and supply the commands you attempted.

# Coding Standards
- Rust 2021 edition, `#![no_std]` defaults; avoid introducing `std` dependencies outside guarded test modules.
- Keep files ASCII unless you have a strong reason otherwise. Use descriptive method names and prefer small, composable helper functions.
- Document non-obvious hardware interactions with short comments; do not duplicate datasheet prose.

# PR / Handoff Checklist
- [ ] All new logic has unit-test coverage.
- [ ] `cargo fmt`, `cargo clippy --no-deps`, and `cargo test` succeed (note any exceptions).
- [ ] Public API changes are documented in doc comments and, if user-visible, in `CHANGELOG.md`.
- [ ] Verify no accidental regressions in multi-controller addressing or buffer bounds.
