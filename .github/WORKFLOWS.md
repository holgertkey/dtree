# GitHub Actions Workflows

This document describes the GitHub Actions workflows configured for dtree-tui.

## Workflows Overview

### 1. CI Workflow (`ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Jobs:**

#### Test Suite
- Runs tests on Linux, macOS, and Windows
- Tests with stable and beta Rust toolchains
- Includes both debug and release mode tests

#### Code Quality Checks
- **Rustfmt**: Ensures code formatting consistency
- **Clippy**: Catches common mistakes and enforces best practices
- **Security Audit**: Checks dependencies for known vulnerabilities

#### Build Verification
- Builds for all supported platforms:
  - Linux (x86_64 glibc and musl)
  - macOS (x86_64 Intel and ARM64 Apple Silicon)
  - Windows (x86_64)
- Reports binary sizes

#### Code Coverage
- Generates test coverage reports
- Uploads to Codecov (optional)

### 2. Release Workflow (`release.yml`)

**Triggers:**
- Push of version tags (e.g., `v1.1.0`, `v2.0.0`)

**Jobs:**

#### Create Release
- Extracts version from git tag
- Reads changelog for the version from `CHANGELOG.md`
- Creates GitHub release with release notes

#### Build Release Binaries
- Cross-compiles for all supported platforms
- Strips symbols to reduce binary size
- Creates archives (tar.gz for Unix, zip for Windows)
- Uploads binaries as release assets

#### Publish to crates.io
- Automatically publishes to crates.io (if `CRATES_TOKEN` is set)
- Only runs after successful builds

#### Update Release Notes
- Enhances release notes with installation instructions
- Adds links to documentation
- Lists all available binaries

## Setup Instructions

### 1. Enable GitHub Actions

GitHub Actions is enabled by default for public repositories. For private repositories:
1. Go to repository Settings → Actions → General
2. Select "Allow all actions and reusable workflows"

### 2. Configure Secrets (Optional)

#### For crates.io Publishing

If you want to automatically publish to crates.io on release:

1. Get your crates.io API token:
   ```bash
   cargo login
   # Token is in ~/.cargo/credentials
   ```

2. Add the token to GitHub:
   - Go to repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `CRATES_TOKEN`
   - Value: Your crates.io API token

#### For Code Coverage (Optional)

If you want to track code coverage with Codecov:

1. Sign up at [codecov.io](https://codecov.io)
2. Add your repository
3. Get the upload token
4. Add to GitHub Secrets:
   - Name: `CODECOV_TOKEN`
   - Value: Your Codecov token

**Note:** The workflow will work without these secrets, but will skip the optional steps.

### 3. Protected Branches (Recommended)

Protect your main branch to require CI checks:

1. Go to Settings → Branches → Branch protection rules
2. Add rule for `main` branch
3. Enable:
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - Select checks: `Test Suite`, `Clippy`, `Rustfmt`, `Build Check`

## Creating a Release

### Manual Release Process

1. **Update version in `Cargo.toml`**:
   ```toml
   [package]
   version = "1.2.0"
   ```

2. **Update `CHANGELOG.md`**:
   - Move items from `[Unreleased]` to new version section
   - Add release date
   - Create new empty `[Unreleased]` section

3. **Commit changes**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Bump version to 1.2.0"
   ```

4. **Create and push tag**:
   ```bash
   git tag -a v1.2.0 -m "Release v1.2.0"
   git push origin main
   git push origin v1.2.0
   ```

5. **Wait for automation**:
   - CI will run tests and checks
   - Release workflow will build binaries
   - GitHub release will be created automatically
   - Binaries will be uploaded as assets
   - (Optional) Package will be published to crates.io

### Automated Release Process (Future)

You can automate the entire release process using tools like:
- [release-plz](https://github.com/MarcoIeni/release-plz)
- [cargo-release](https://github.com/crate-ci/cargo-release)

## Monitoring Workflows

### View Workflow Runs

1. Go to repository → Actions tab
2. Click on a workflow to see runs
3. Click on a run to see logs

### Debugging Failed Workflows

If a workflow fails:

1. Click on the failed run
2. Click on the failed job
3. Expand the failed step to see error logs
4. Fix the issue locally and push again

### Common Issues

#### Build Failures

- **Missing dependencies**: Check if all required system packages are installed
- **Platform-specific issues**: Test locally with the same target triple
- **Test failures**: Run tests locally with `cargo test --all-features`

#### Release Failures

- **Tag already exists**: Delete and recreate the tag
  ```bash
  git tag -d v1.2.0
  git push origin :refs/tags/v1.2.0
  # Fix issues, then recreate tag
  ```

- **Binary upload fails**: Check GitHub API rate limits or token permissions

- **crates.io publish fails**:
  - Ensure `CRATES_TOKEN` is set correctly
  - Check that version doesn't already exist on crates.io
  - Verify `Cargo.toml` metadata is complete

## Workflow Badges

Add these badges to your `README.md`:

```markdown
[![CI](https://github.com/holgertkey/dtree/actions/workflows/ci.yml/badge.svg)](https://github.com/holgertkey/dtree/actions/workflows/ci.yml)
[![Release](https://github.com/holgertkey/dtree/actions/workflows/release.yml/badge.svg)](https://github.com/holgertkey/dtree/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/holgertkey/dtree/branch/main/graph/badge.svg)](https://codecov.io/gh/holgertkey/dtree)
```

## Customization

### Changing Supported Platforms

Edit the `matrix.include` section in `release.yml`:

```yaml
- os: ubuntu-latest
  target: x86_64-unknown-linux-gnu
  asset_name: dtree-linux-x86_64
  use_cross: false
```

### Adjusting Test Matrix

Edit the `matrix` section in `ci.yml`:

```yaml
matrix:
  os: [ubuntu-latest, windows-latest, macos-latest]
  rust: [stable, beta]
```

### Disabling Optional Jobs

Comment out or remove jobs you don't need:

```yaml
# coverage:
#   name: Code Coverage
#   ...
```

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Cross-compilation with cross](https://github.com/cross-rs/cross)
- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)

## Support

If you encounter issues with the workflows:

1. Check the [GitHub Actions status page](https://www.githubstatus.com/)
2. Review the workflow logs in the Actions tab
3. Search for similar issues in the repository issues
4. Open a new issue with workflow logs attached
