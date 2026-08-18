# Contributing to Headless Engine

We welcome community contributions, bug reports, and pull requests! This document outlines our development workflow and standards.

---

## 🛠️ Local Development Setup

### 1. Prerequisites
- [Rust](https://rustup.rs/) (Stable 1.75+)
- Cargo

### 2. Clone & Build
```bash
git clone https://github.com/your-username/headless-engine.git
cd headless-engine

# Run check
cargo check --all-targets

# Run tests
cargo test

# Run examples
cargo run --example multitab_concurrent
cargo run --example scrape_to_markdown
```

---

## 📋 Code Standards

Before submitting a Pull Request:
1. **Formatting:** Ensure code is formatted:
   ```bash
   cargo fmt --all
   ```
2. **Clippy Linter:** Check for warnings:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. **Tests:** All tests must pass:
   ```bash
   cargo test --all-targets --all-features
   ```

---

## 🚀 Submitting Pull Requests

1. Fork the repository and create a feature branch (`git checkout -b feature/awesome-feature`).
2. Commit your changes with clear, descriptive commit messages.
3. Push to your fork and submit a Pull Request to `main`.
4. Ensure all GitHub Actions CI checks pass.
