---
name: query-sino-cgm
description: Query SINO CGM production data through the `sino` CLI by `user_id`. Use when asked to查某个用户的 CGM 数据、查询某日或某时间段血糖、查看用户档案、读取单日聚合健康数据、按 event_id 查事件详情，或需要通过 CLI 调用三诺正式环境 `scrm-mcp` 接口。 Triggers include “按 user_id 查 CGM”, “查询某用户血糖”, “query SINO CGM by user_id”, “get glucose data for a user”, “查看 daily data”, and “查 event 详情”.
---

# Query Sino Cgm

## Overview

Use the `sino` CLI to query the production SCRM MCP API at `https://ican.sinocare.com/api/scrm-mcp`.
Treat `user_id` as the required identifier in the current version. Prefer JSON output so downstream agents can reason over the results safely.

## Workflow

1. Confirm the `user_id`.
2. Choose the narrowest command that answers the request.
3. Prefer `--pretty` for human-facing summaries and `--json` for machine processing.
4. If the user asks for a date range, use `cgm range`.
5. If the user asks for a single day with all related signals, use `daily`.
6. If the user asks for a specific event, use `event get`.

## Command Map

- Health check:
  - `sino health --json`
- User profile:
  - `sino user info --user-id <USER_ID> --json`
- Single-day CGM:
  - `sino cgm day --user-id <USER_ID> --date <YYYY-MM-DD> --json`
- Date-range CGM:
  - `sino cgm range --user-id <USER_ID> --start-date <YYYY-MM-DD> --end-date <YYYY-MM-DD> --json`
- Single-day aggregated data:
  - `sino daily --user-id <USER_ID> --date <YYYY-MM-DD> --json`
- Event detail:
  - `sino event get --user-id <USER_ID> --event-id <EVENT_ID> --json`

## Parameter Rules

- `user_id` is required for every data lookup command.
- Dates must use `YYYY-MM-DD`.
- Use either a single `date` or a `start-date` and `end-date` pair.
- Do not invent user IDs. Ask for one if it is missing.

## Failure Handling

- If the CLI returns a 400-style parameter error, check date format first.
- If the CLI returns a 500-style query error, report the backend error text without guessing a fix.
- If the response is empty, state that no data was returned for the requested `user_id` and time window.
- If the user asks for current-user flows or token-based auth, explain that the current version still requires `user_id`.

## References

- For concrete command examples and response-shape notes, read `references/workflow.md`.
