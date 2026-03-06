# Architecture

## Layers

- `crates/sino-core`
  - Production HTTP client for `https://ican.sinocare.com/api/scrm-mcp`
  - Shared request building and backend error handling
- `crates/sino-cli`
  - Rust CLI entrypoint and command parsing
  - Prints JSON returned by the production API
- `packages/sino-cli`
  - Thin npm wrapper that locates and starts the Rust binary
- `skills/query-sino-cgm`
  - Agent instructions for calling the CLI with `user_id`

## Current Constraints

- Production environment only
- `user_id` is required for data lookups
- No test environment switching
- `personal_token` is only a reserved future extension point

## v1 Command Mapping

- `sino health`
- `sino user info --user-id <id>`
- `sino cgm day --user-id <id> --date <YYYY-MM-DD>`
- `sino cgm range --user-id <id> --start-date <YYYY-MM-DD> --end-date <YYYY-MM-DD>`
- `sino daily --user-id <id> --date <YYYY-MM-DD>`
- `sino event get --user-id <id> --event-id <id>`
