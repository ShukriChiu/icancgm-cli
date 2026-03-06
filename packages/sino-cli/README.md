# @sinocare/icancgm-cli

`@sinocare/icancgm-cli` installs the `sino` command and downloads the matching Rust binary from GitHub Releases during `postinstall`.

## Install

```bash
npm install -g @sinocare/icancgm-cli
```

The binary is downloaded from:

```text
https://github.com/ShukriChiu/icancgm-cli/releases
```

## Verify

```bash
sino health --json
```

## Notes

- Production environment only
- Current version requires `user_id` for data queries
- The npm package is a thin wrapper; the actual query engine is the Rust binary

## Useful environment variables

- `SINO_BINARY_PATH`
- `SINO_SKIP_DOWNLOAD=1`
- `SINO_RELEASE_REPO`
- `SINO_RELEASE_VERSION`
- `SINO_RELEASE_BASE_URL`
