# Rust Linting

This directory provides Clippy and rustfmt settings, plus optional custom Dylint fallback rules.

## Install

Install Rust components:

```bash
rustup component add clippy rustfmt
```

Install Dylint tools (optional, for custom lint in `.dylint`):

```bash
cargo install cargo-dylint dylint-link
rustup toolchain install nightly-2025-09-18
rustup component add --toolchain nightly-2025-09-18 rustc-dev llvm-tools-preview
```

## Use the configs

Copy these files into your Rust project root:

- `rust/Cargo.toml` lint section into your `Cargo.toml`
- `rust/clippy.toml` -> `clippy.toml`
- `rust/rustfmt.toml` -> `rustfmt.toml`
- `rust/.cargo/config.toml` -> `.cargo/config.toml` (adds `cargo dylint-all` alias)

For the custom fallback lint, also see `rust/.dylint/README.md`.

## Run

```bash
cargo clippy --all-targets --all-features
cargo fmt --all -- --check
cargo dylint-all
```
