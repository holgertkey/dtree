# Release Checklist

Use this checklist when preparing a new release of dtree-tui.

## Pre-Release Preparation

### 1. Version Update

- [ ] Update version in `Cargo.toml`:
  ```toml
  [package]
  version = "X.Y.Z"
  ```

- [ ] Update version in `Cargo.lock`:
  ```bash
  cargo update -p dtree-tui
  ```

### 2. Changelog Update

- [ ] Update `CHANGELOG.md`:
  - [ ] Move changes from `[Unreleased]` to new version section
  - [ ] Add release date: `## [X.Y.Z] - YYYY-MM-DD`
  - [ ] Create new empty `[Unreleased]` section at top
  - [ ] Add version link at bottom: `[X.Y.Z]: https://github.com/holgertkey/dtree/releases/tag/vX.Y.Z`

### 3. Documentation Update

- [ ] Review and update README.md if needed
- [ ] Update documentation in `docs/` directory if features changed
- [ ] Check that all new features are documented

### 4. Testing

- [ ] Run all tests locally:
  ```bash
  cargo test --all-features
  ```

- [ ] Run clippy:
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] Check formatting:
  ```bash
  cargo fmt --all -- --check
  ```

- [ ] Build release binary and test manually:
  ```bash
  cargo build --release
  ./target/release/dtree --version
  ./target/release/dtree  # Test interactively
  ```

- [ ] Test on all platforms (if possible):
  - [ ] Linux
  - [ ] macOS
  - [ ] Windows

### 5. Final Checks

- [ ] All CI checks passing on main branch
- [ ] No open critical bugs
- [ ] All planned features for this release completed
- [ ] Security audit passed:
  ```bash
  cargo audit
  ```

## Release Process

### 1. Commit Changes

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Bump version to X.Y.Z"
```

### 2. Push to Main

```bash
git push origin main
```

- [ ] Wait for CI to pass
- [ ] Review GitHub Actions results

### 3. Create and Push Tag

```bash
# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push tag to trigger release workflow
git push origin vX.Y.Z
```

### 4. Monitor Release Workflow

- [ ] Go to [Actions tab](https://github.com/holgertkey/dtree/actions)
- [ ] Watch the Release workflow:
  - [ ] Create Release job completes
  - [ ] Build Release Binaries jobs complete (all platforms)
  - [ ] Update Release Notes completes

### 5. Verify Release

- [ ] Check [Releases page](https://github.com/holgertkey/dtree/releases)
- [ ] Verify release notes are correct
- [ ] Verify all binaries are attached:
  - [ ] `dtree-linux-x86_64.tar.gz`
  - [ ] `dtree-linux-x86_64-musl.tar.gz`
  - [ ] `dtree-linux-aarch64.tar.gz`
  - [ ] `dtree-macos-x86_64.tar.gz`
  - [ ] `dtree-macos-aarch64.tar.gz`
  - [ ] `dtree-windows-x86_64.exe.zip`

- [ ] Test download and installation of at least one binary:
  ```bash
  # Example for Linux
  wget https://github.com/holgertkey/dtree/releases/download/vX.Y.Z/dtree-linux-x86_64.tar.gz
  tar xzf dtree-linux-x86_64.tar.gz
  ./dtree --version
  ```

## Post-Release

### 1. Announcements

- [ ] Create announcement on GitHub Discussions (if applicable)
- [ ] Update project description if major changes
- [ ] Tweet/post about release (if applicable)

### 2. Documentation

- [ ] Update any external documentation
- [ ] Update wiki (if exists)

### 3. Monitoring

- [ ] Monitor issue tracker for release-related bugs
- [ ] Watch download statistics
- [ ] Check for user feedback

## Hotfix Process (for urgent patches)

If you need to release a hotfix:

1. Create hotfix branch from tag:
   ```bash
   git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z
   ```

2. Make minimal fix
3. Update version to X.Y.Z+1 (patch increment)
4. Update CHANGELOG with hotfix details
5. Test thoroughly
6. Commit, tag, and push:
   ```bash
   git commit -m "Hotfix: description"
   git tag -a vX.Y.Z+1 -m "Hotfix release vX.Y.Z+1"
   git push origin hotfix/vX.Y.Z+1
   git push origin vX.Y.Z+1
   ```
7. Merge hotfix branch back to main:
   ```bash
   git checkout main
   git merge hotfix/vX.Y.Z+1
   git push origin main
   ```

## Rollback Process (if release fails)

If the release has critical issues:

1. Delete the release from GitHub:
   - Go to Releases page
   - Click on the problematic release
   - Click "Delete this release"

2. Delete the tag:
   ```bash
   git tag -d vX.Y.Z
   git push origin :refs/tags/vX.Y.Z
   ```

3. Fix issues, increment version, and release again

## Version Numbering Guide

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes
  - API/CLI changes that break existing workflows
  - Remove features
  - Major behavior changes

- **MINOR** (x.Y.0): New features (backwards compatible)
  - New commands or options
  - New functionality
  - Performance improvements

- **PATCH** (x.y.Z): Bug fixes (backwards compatible)
  - Bug fixes
  - Documentation updates
  - Security patches

## Notes

- Always create **annotated tags** (`git tag -a`) not lightweight tags
- Keep CHANGELOG.md up to date throughout development
- Test release process in a fork first if unsure
- Document any breaking changes clearly
- Consider creating a release candidate (rc) for major versions

## Resources

- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Actions Documentation](.github/WORKFLOWS.md)
