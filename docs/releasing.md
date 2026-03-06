# Releasing And Installing

## Overview

The npm package downloads a platform-specific Rust binary from GitHub Releases during `postinstall`.

When you push a tag like `v0.1.0`, `.github/workflows/release-binaries.yml` automatically builds and uploads the 5 expected platform binaries to a GitHub Release.

The release flow is:

1. Set the new version in both Rust and npm metadata.
2. Run the release readiness check.
3. Create and push a Git tag that matches the npm package version, for example `v0.1.0`.
4. Let GitHub Actions create or update the GitHub Release and upload the binaries.
5. Let the npm publish workflow publish `@shukrichiu/icancgm-cli`.

The wrapper then downloads:

- `sino-darwin-arm64`
- `sino-darwin-x64`
- `sino-linux-arm64`
- `sino-linux-x64`
- `sino-win32-x64.exe`

## Expected GitHub Release Repo

By default the wrapper downloads from:

```text
https://github.com/shukrichiu/icancgm-cli/releases/download/<tag>/<asset>
```

You can override this during install with:

```bash
SINO_RELEASE_REPO=owner/repo npm install -g @shukrichiu/icancgm-cli
```

## Build Rust Release Binaries

Run these from the repository root.

### macOS Apple Silicon

```bash
cargo build --release -p sino-cli --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/sino ./sino-darwin-arm64
```

### macOS Intel

```bash
cargo build --release -p sino-cli --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/sino ./sino-darwin-x64
```

### Linux x64

```bash
cargo build --release -p sino-cli --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/sino ./sino-linux-x64
```

### Linux arm64

```bash
cargo build --release -p sino-cli --target aarch64-unknown-linux-gnu
cp target/aarch64-unknown-linux-gnu/release/sino ./sino-linux-arm64
```

GitHub Actions builds this target with `cargo-zigbuild` and a pinned Zig version (`0.13.0`) to avoid the flaky `latest` resolution path.

### Windows x64

```bash
cargo build --release -p sino-cli --target x86_64-pc-windows-msvc
cp target/x86_64-pc-windows-msvc/release/sino.exe ./sino-win32-x64.exe
```

## Create The GitHub Release

Make sure the package version in `packages/sino-cli/package.json` and the Rust workspace version are aligned.

Use the helper script to update both places together:

```bash
npm run version:set -- 0.1.0
```

Run the consistency check before tagging:

```bash
npm run release:check -- v0.1.0
```

Example for version `0.1.0`:

```bash
git tag v0.1.0
git push origin v0.1.0
```

After the tag is pushed, GitHub Actions automatically:

- builds 5 target binaries in parallel
- creates or updates the GitHub Release
- uploads the expected assets
- publishes the npm package after the GitHub Release is published

Use the manual `gh release create` flow only as a fallback if the workflow is unavailable.

## Publish The npm Package

`publish-npm.yml` publishes the npm package automatically after the release is published.

Before this works, configure the repository secret:

```text
NPM_TOKEN
```

The workflow will:

- check out the tagged commit
- run `npm run release:check`
- build the wrapper package
- run `npm publish --workspace @shukrichiu/icancgm-cli --access public --provenance`

## Manual npm Publish Fallback

If you need to publish manually, build the TypeScript wrapper first:

```bash
npm install
npm run build -w @shukrichiu/icancgm-cli
```

Then publish from the workspace package:

```bash
npm publish --workspace @shukrichiu/icancgm-cli --access public
```

If you want to test the package tarball locally first:

```bash
npm run release:pack
```

## Install The CLI

After publishing:

```bash
npm install -g @shukrichiu/icancgm-cli
```

The package will automatically:

1. Detect the current OS and CPU architecture.
2. Download the matching binary from GitHub Releases.
3. Save it under the installed package directory.
4. Expose the `sino` command on `PATH`.

## Verify Installation

```bash
sino health --json
```

Expected result: a JSON health response from `https://ican.sinocare.com/api/scrm-mcp`.

## Useful Overrides

### Use a different binary path

```bash
SINO_BINARY_PATH=/absolute/path/to/sino sino health --json
```

### Skip the release download

Useful for local development:

```bash
SINO_SKIP_DOWNLOAD=1 npm install
```

### Use a different release repo

```bash
SINO_RELEASE_REPO=my-org/icancgm-cli npm install -g @shukrichiu/icancgm-cli
```

### Use a different release tag

```bash
SINO_RELEASE_VERSION=v0.1.0-beta.1 npm install -g @shukrichiu/icancgm-cli
```

### Use a custom base release URL

This is useful if you later move binaries to an object store.

```bash
SINO_RELEASE_BASE_URL=https://downloads.example.com/icancgm-cli npm install -g @shukrichiu/icancgm-cli
```
