check:
    cargo fmt --all
    cargo build --workspace
    cargo clippy --workspace --all-targets
    cargo test --workspace

deny:
    cargo deny check
