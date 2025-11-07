# Contributing to asyncapi-rust

Thank you for your interest in contributing! This document provides guidelines for contributing to asyncapi-rust.

## Code of Conduct

This project adheres to the Rust Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to mark@lilback.com.

## How to Contribute

### Reporting Bugs

- Check existing issues to avoid duplicates
- Use the bug report template
- Include minimal reproducible example
- Specify Rust version and OS

### Suggesting Features

- Check existing issues and discussions
- Explain the use case clearly
- Consider implementation complexity
- Be open to feedback

### Pull Requests

1. **Fork and clone** the repository
2. **Create a branch** from `main`: `git checkout -b feature/my-feature`
3. **Make your changes** with clear commit messages
4. **Add tests** for new functionality
5. **Run tests**: `cargo test --all-features`
6. **Run clippy**: `cargo clippy -- -D warnings`
7. **Format code**: `cargo fmt`
8. **Update documentation** if needed
9. **Submit PR** with description of changes

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/asyncapi-rust.git
cd asyncapi-rust

# Build the project (also installs pre-commit hooks via cargo-husky)
cargo build

# Run tests
cargo test --all-features

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```

### Pre-commit Hooks

This project uses [cargo-husky](https://github.com/rhysd/cargo-husky) to automatically run quality checks before each commit.

**What gets checked automatically:**
- ✅ **rustfmt** - Code must be properly formatted
- ✅ **clippy** - No clippy warnings allowed

The hooks are installed automatically when you run `cargo build` or `cargo test` for the first time.

**To bypass hooks (not recommended):**
```bash
git commit --no-verify
```

**Note:** CI will still enforce these checks, so bypassing hooks may cause CI failures.

**Troubleshooting:**
If hooks aren't running:
```bash
# Reinstall hooks
rm -rf .git/hooks && cargo build
```

### Testing

- Unit tests in each crate: `cargo test -p asyncapi-rust-codegen`
- Integration tests: `cargo test --test integration_*`
- All tests must pass before merge

### Documentation

- Add doc comments for public APIs
- Use examples in doc comments
- Update README if adding features
- Follow Rust documentation guidelines

### Code Style

- Follow Rust API Guidelines
- Use `cargo fmt` (enforced in CI)
- Pass `cargo clippy` with no warnings
- Keep commits focused and atomic

## Release Process (Maintainers)

Releases are managed via the `scripts/release.sh` script with automatic quality checks.

**To create a release:**

```bash
./scripts/release.sh
```

**The script automatically runs:**
1. ✅ Code formatting check (`cargo fmt --all -- --check`)
2. ✅ Clippy with warnings denied (`cargo clippy --all-targets --all-features -- -D warnings`)
3. ✅ Full test suite (`cargo test --all-features`)
4. ✅ Documentation build (`cargo doc --all-features --no-deps`)
5. Generates changelog using git-cliff
6. Updates version numbers in all `Cargo.toml` files
7. Creates git commit and tag

**After the script completes:**
1. Review changes: `git show HEAD`
2. Push to GitHub: `git push origin main && git push origin vX.Y.Z`
3. Publish to crates.io (in dependency order):
   ```bash
   cd asyncapi-rust-models && cargo publish
   sleep 30  # Wait for crates.io indexing
   cd ../asyncapi-rust-codegen && cargo publish
   sleep 30
   cd ../asyncapi-rust && cargo publish
   ```
4. Create GitHub release using the generated changelog

**Note:** The release script will abort if any quality check fails. All issues must be fixed before creating a release.

## License

By contributing, you agree that your contributions will be dual-licensed under MIT OR Apache-2.0, matching the project license.

## Questions?

Feel free to open an issue for questions or reach out to mark@lilback.com.
