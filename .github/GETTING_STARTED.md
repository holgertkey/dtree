# Getting Started with GitHub Actions

This guide helps you get started with the automated CI/CD pipeline for dtree-tui.

## What Was Set Up

✅ **CHANGELOG.md** - Updated with version 1.1.0 details
✅ **CI Workflow** (`.github/workflows/ci.yml`) - Automated testing and checks
✅ **Release Workflow** (`.github/workflows/release.yml`) - Automated binary releases
✅ **Workflow Documentation** (`.github/WORKFLOWS.md`) - Detailed workflow documentation
✅ **Release Checklist** (`.github/RELEASE_CHECKLIST.md`) - Step-by-step release guide
✅ **README Badges** - Status badges for CI, releases, and downloads

## Quick Start

### 1. Commit and Push Changes

```bash
# Add all GitHub Actions files
git add .github/ CHANGELOG.md README.md

# Commit
git commit -m "Add GitHub Actions CI/CD pipeline

- Add automated testing workflow (ci.yml)
- Add automated release workflow (release.yml)
- Update CHANGELOG.md for v1.1.0
- Add workflow documentation and release checklist
- Add status badges to README"

# Push to main
git push origin main
```

### 2. Verify CI Workflow

After pushing, the CI workflow will automatically run:

1. Go to: https://github.com/holgertkey/dtree/actions
2. Click on the latest "CI" workflow run
3. Verify all jobs pass:
   - ✅ Test Suite (Linux, macOS, Windows)
   - ✅ Rustfmt
   - ✅ Clippy
   - ✅ Build Check (all platforms)
   - ✅ Security Audit

### 3. Create Your First Automated Release (Optional)

When you're ready to create a new release:

1. **Follow the Release Checklist**:
   ```bash
   # See detailed steps in:
   cat .github/RELEASE_CHECKLIST.md
   ```

2. **Quick Release Steps**:
   ```bash
   # 1. Update version in Cargo.toml
   # 2. Update CHANGELOG.md
   # 3. Commit changes
   git add Cargo.toml CHANGELOG.md
   git commit -m "Bump version to X.Y.Z"
   git push origin main

   # 4. Create and push tag
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```

3. **Monitor Release**:
   - Go to: https://github.com/holgertkey/dtree/actions
   - Watch the "Release" workflow build binaries
   - Check releases: https://github.com/holgertkey/dtree/releases

## What Happens Now?

### On Every Push to Main/Develop

The CI workflow automatically:
- ✅ Runs all tests on Linux, macOS, and Windows
- ✅ Checks code formatting (rustfmt)
- ✅ Runs linter (clippy)
- ✅ Builds for all supported platforms
- ✅ Runs security audit
- ✅ Generates code coverage report

### On Every New Tag (vX.Y.Z)

The Release workflow automatically:
- ✅ Creates a GitHub Release
- ✅ Builds binaries for 6 platforms:
  - Linux x86_64 (glibc)
  - Linux x86_64 (musl - static)
  - Linux ARM64
  - macOS x86_64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64
- ✅ Uploads binaries to the release
- ✅ Updates release notes with installation instructions

## Status Badges

Your README now includes these badges:

- **CI Badge**: Shows if tests are passing
- **Release Badge**: Shows if release workflow succeeded
- **GitHub Release Badge**: Shows latest version
- **Crates.io Badge**: Shows published version
- **Downloads Badge**: Shows download count

## Troubleshooting

### CI Workflow Fails

1. Check the workflow logs in Actions tab
2. Run the same commands locally:
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
3. Fix issues and push again

### Release Workflow Fails

1. Check if tag already exists:
   ```bash
   git tag -l | grep vX.Y.Z
   ```
2. Delete and recreate if needed:
   ```bash
   git tag -d vX.Y.Z
   git push origin :refs/tags/vX.Y.Z
   # Fix issues, then recreate tag
   ```

### Need Help?

- 📖 Read detailed docs: `.github/WORKFLOWS.md`
- 📋 Follow release checklist: `.github/RELEASE_CHECKLIST.md`
- 🐛 Open an issue: https://github.com/holgertkey/dtree/issues

## Next Steps

1. ✅ Commit and push the GitHub Actions files
2. ✅ Verify CI workflow passes
3. ✅ When ready, create a new release following the checklist

## Learn More

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow Configuration](.github/WORKFLOWS.md)
- [Release Checklist](.github/RELEASE_CHECKLIST.md)
- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)

Happy releasing! 🚀
