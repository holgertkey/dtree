# Release Process

This document describes the process for creating a new release of dtree.

## Prerequisites

- [ ] All changes committed to `medium` branch
- [ ] All tests passing: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt -- --check`
- [ ] Documentation up to date
- [ ] Manual testing completed (see Testing Checklist below)

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Incompatible API changes or major rewrites
- **MINOR** (x.Y.0): New features, backwards compatible
- **PATCH** (x.y.Z): Bug fixes, backwards compatible

## Release Steps

### 1. Prepare Release

```bash
# Ensure you're on the correct branch
git checkout medium
git pull origin medium

# Verify clean working tree
git status

# Run full test suite
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt -- --check
```

### 2. Update Version

Edit these files:

**Cargo.toml:**
```toml
version = "X.Y.Z"  # Update to new version
```

**CHANGELOG.md:**
```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New feature descriptions

### Changed
- Changed feature descriptions

### Fixed
- Bug fix descriptions

### Removed
- Removed feature descriptions
```

Add link at bottom:
```markdown
[X.Y.Z]: https://github.com/holgertkey/dtree/releases/tag/vX.Y.Z
```

### 3. Build Release Binary

```bash
# Clean previous builds
cargo clean

# Build optimized release
cargo build --release

# Verify binary
ls -lh target/release/dtree
du -sh target/release/dtree

# Test the binary
./target/release/dtree --version
./target/release/dtree --help
```

### 4. Testing Checklist

Run through all major features:

#### Navigation
- [ ] Basic navigation (j/k/l/h)
- [ ] Enter to change directory
- [ ] u/Backspace to parent
- [ ] Mouse click to select
- [ ] Double-click to navigate

#### File Viewer
- [ ] Toggle split view (s)
- [ ] Scroll file content (Ctrl+j/k)
- [ ] Page Up/Down navigation
- [ ] Fullscreen viewer (v)
- [ ] Line numbers toggle (l in fullscreen)
- [ ] Line wrapping toggle (w in fullscreen)
- [ ] HEAD/TAIL mode (Home/End)

#### Visual Selection
- [ ] Enter visual mode (V in fullscreen)
- [ ] Select lines (j/k)
- [ ] Copy selection (y)
- [ ] Exit visual mode (Esc)

#### Search
- [ ] Normal search (/)
- [ ] Fuzzy search (/query)
- [ ] Navigate results (↑/↓)
- [ ] Select result (Enter)
- [ ] Cancel search (Esc)

#### Bookmarks
- [ ] Create bookmark (m)
- [ ] Select bookmark (')
- [ ] Delete bookmark (d in selection)
- [ ] Direct jump: `dt bookmark_name`

#### External Programs
- [ ] Open in editor (e)
- [ ] Open in file manager (o)
- [ ] Open hex editor for binary files

#### Terminal Cleanup
- [ ] Exit with q - clean terminal
- [ ] Exit with Esc - clean terminal
- [ ] Resize terminal in split view + exit - no artifacts
- [ ] Multiple exits in succession - no artifacts

#### Edge Cases
- [ ] Large directories (1000+ files)
- [ ] Binary files detection
- [ ] Permission denied directories
- [ ] Symlinks (broken and valid)
- [ ] Unicode filenames
- [ ] Very long file paths

### 5. Commit and Tag

```bash
# Stage changes
git add Cargo.toml CHANGELOG.md

# Commit with descriptive message
git commit -m "Release version X.Y.Z

- Update version to X.Y.Z
- Update CHANGELOG with release notes
"

# Create annotated tag
git tag -a vX.Y.Z -m "Release version X.Y.Z

Key changes:
- Feature 1
- Feature 2
- Bug fix 1
"

# Verify tag
git tag -l -n9 vX.Y.Z
```

### 6. Push to Repository

```bash
# Push commits
git push origin medium

# Push tags
git push origin vX.Y.Z

# Or push all tags
git push origin --tags
```

### 7. Create GitHub Release

1. Go to: https://github.com/holgertkey/dtree/releases/new
2. Select tag: `vX.Y.Z`
3. Release title: `dtree vX.Y.Z`
4. Description: Copy from CHANGELOG.md for this version
5. Attach binary (optional):
   - Upload `target/release/dtree` (rename to `dtree-vX.Y.Z-linux-x86_64`)
6. Mark as pre-release if applicable
7. Publish release

### 8. Post-Release

```bash
# Update version to next development version
# Edit Cargo.toml:
version = "X.Y+1.0-dev"  # For next minor version
# or
version = "X+1.0.0-dev"  # For next major version

# Commit
git add Cargo.toml
git commit -m "Bump version to X.Y+1.0-dev"
git push origin medium
```

Update CHANGELOG.md:
```markdown
## [Unreleased]

### Added
### Changed
### Fixed
### Removed
```

## Hotfix Process

For urgent bug fixes to a released version:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z

# Make fixes
# ... edit files ...

# Test thoroughly
cargo test
cargo clippy

# Update version to X.Y.Z+1
# Update CHANGELOG

# Commit and tag
git commit -m "Hotfix: description"
git tag -a vX.Y.Z+1 -m "Hotfix release X.Y.Z+1"

# Merge back to main branch
git checkout medium
git merge hotfix/vX.Y.Z+1

# Push
git push origin medium
git push origin vX.Y.Z+1
```

## Publishing to crates.io (Optional)

If publishing to crates.io:

```bash
# Dry run first
cargo publish --dry-run

# Check package contents
cargo package --list

# Publish
cargo publish
```

**Note**: Once published to crates.io, versions cannot be deleted or modified.

## Rollback Process

If a critical issue is discovered after release:

1. **Yanking from crates.io** (if published):
   ```bash
   cargo yank --vers X.Y.Z
   ```

2. **GitHub Release**:
   - Edit release and mark as "Pre-release"
   - Add warning in description
   - Create new hotfix release

3. **Communication**:
   - Update GitHub issue tracker
   - Add notice to README if critical

## Release Checklist Summary

- [ ] Version updated in Cargo.toml
- [ ] CHANGELOG.md updated with release notes
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Manual testing completed
- [ ] Release binary built and tested
- [ ] Changes committed
- [ ] Git tag created
- [ ] Pushed to repository
- [ ] GitHub release created
- [ ] Next dev version set
- [ ] Release announced (if applicable)

## Automation (Future)

Consider automating with:
- GitHub Actions for CI/CD
- Automated changelog generation
- Binary artifact uploads
- Release announcement automation

## Questions?

If you have questions about the release process, open an issue or discussion on GitHub.

---

Last updated: 2025-01-24
