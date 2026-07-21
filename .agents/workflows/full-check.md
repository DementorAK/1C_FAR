---
description: Full check after code edits
---

If changes are made to the code base, it is essential to run tests, check the formatting, and check the code quality for all supported platforms. 

> [!IMPORTANT]
> All errors, warnings, formatting discrepancies, and test failures found during these checks must be completely resolved and fixed.

# Formatting

cargo fmt --all

# Windows (far3)

cargo check --target x86_64-pc-windows-msvc --all-targets
cargo clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings
cargo test

# Linux (far2l)

cargo check --no-default-features --features far2l --target x86_64-unknown-linux-gnu --all-targets
cargo clippy --no-default-features --features far2l --target x86_64-unknown-linux-gnu --all-targets -- -D warnings
cargo test --no-default-features --features far2l

# Linux (far2m)

cargo check --no-default-features --features far2m --target x86_64-unknown-linux-gnu --all-targets
cargo clippy --no-default-features --features far2m --target x86_64-unknown-linux-gnu --all-targets -- -D warnings
cargo test --no-default-features --features far2m
