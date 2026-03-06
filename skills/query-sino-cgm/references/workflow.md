# Query Workflow

## Environment

- Production only: `https://ican.sinocare.com/api/scrm-mcp`
- Current version uses `user_id` for every data command.
- Future token-based flows are intentionally out of scope for this version.

## Recommended Commands

### User profile

```bash
sino user info --user-id 5e453d07e534bf0008ce11f0 --json
```

### Single-day CGM

```bash
sino cgm day --user-id 5e453d07e534bf0008ce11f0 --date 2025-02-04 --json
```

### Date-range CGM

```bash
sino cgm range \
  --user-id 5e453d07e534bf0008ce11f0 \
  --start-date 2025-02-01 \
  --end-date 2025-02-04 \
  --json
```

### Daily aggregate

```bash
sino daily --user-id 5e453d07e534bf0008ce11f0 --date 2025-02-04 --json
```

### Event detail

```bash
sino event get \
  --user-id 5e453d07e534bf0008ce11f0 \
  --event-id record_123 \
  --json
```

## Parameter Constraints

- Dates must be `YYYY-MM-DD`.
- `cgm day` is for a single `date`.
- `cgm range` requires both `start-date` and `end-date`.
- `event get` requires both `user_id` and `event_id`.

## Response Handling

- `user info` returns `{ user_id, user_info }`.
- `cgm` commands return `{ user_id, data, count, ... }`.
- `daily` returns a single-day aggregate JSON object with multiple arrays.
- `event get` returns `{ event: ... }`.

## Common Failure Cases

- Invalid date format: retry with `YYYY-MM-DD`.
- Missing `user_id`: ask the user for the exact identifier.
- Backend query error: surface the error without fabricating a diagnosis.
- Empty arrays: say that no matching production data was returned for the requested time window.
