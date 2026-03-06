# Shujian's Health Data Sources — Connection & Query Reference

## Table of Contents

1. [Data Source Overview](#data-source-overview)
2. [Credential Management](#credential-management)
3. [CGM: Sinocare (三诺)](#cgm-sinocare-三诺)
4. [CGM: Other Brands](#cgm-other-brands)
5. [Wearables: Smartwatch](#wearables-smartwatch)
6. [Wearables: Oura Ring](#wearables-oura-ring)
7. [Blood Work & Lab Results](#blood-work--lab-results)
8. [Data Interpretation Quick Reference](#data-interpretation-quick-reference)

---

## Data Source Overview

This skill connects to multiple health data sources. Some are queryable via CLI
tools, others rely on user-provided exports or screenshots. Always check what
data the user actually has before suggesting a specific source.

| Source | Status | Access method | Key data |
|--------|--------|---------------|----------|
| Sinocare CGM (三诺) | **Active** | `sino` CLI | Glucose readings, daily aggregates, events |
| Dexcom CGM | Planned | API / export | Glucose readings |
| Freestyle Libre | Planned | Export / LibreView | Glucose readings |
| Apple Watch | Planned | Apple Health export | HR, HRV, steps, workouts, sleep |
| Oura Ring | Planned | Oura export / API | Sleep stages, HRV, readiness, temperature |
| Whoop | Planned | Export | Recovery, strain, sleep |
| Blood work | **Active** | User pastes / uploads | Metabolic panel, hormones, nutrients |

When a data source is "Planned", ask the user to paste or upload their data
directly in the conversation. When it's "Active", use the documented access
method to fetch data programmatically.

---

## Credential Management

The CLI uses a unified credential store so users configure each data source
once and never have to pass credentials manually again.

### Setup credentials

```bash
# View all data sources and their credential status
sino auth list

# View step-by-step guide for getting a specific source's credentials
sino auth guide sinocare
sino auth guide oura
sino auth guide dexcom

# Add credentials (one-time per source)
sino auth add sinocare --user-id <USER_ID>
sino auth add oura --token <PERSONAL_ACCESS_TOKEN>
sino auth add dexcom --username <EMAIL> --password <PASSWORD>

# Update credentials (same command, upsert semantics)
sino auth add sinocare --user-id <NEW_USER_ID>

# Remove credentials
sino auth remove sinocare
```

Credentials are stored locally at `~/.config/sino/credentials.json` (or
`~/Library/Application Support/sino/credentials.json` on macOS) with
`0600` file permissions.

### Credential resolution order

When a command needs a user_id or token, the CLI resolves it in this order:

1. Explicit CLI argument (e.g. `--user-id <ID>`) — highest priority
2. Credential store (`sino auth add`)
3. Error with setup guide if neither is available

---

## CGM: Sinocare (三诺)

### Setup

```bash
# Check if sino CLI is installed and get version
sino --version
# Expected: sino 0.1.0 or higher

# Install if missing
npm install -g @shukrichiu/icancgm-cli

# Update if version is too old (needs 0.1.0+ for auth commands)
npm update -g @shukrichiu/icancgm-cli

# Verify installation
sino health --json

# Configure credentials (one-time)
sino auth add sinocare --user-id <USER_ID>
```

**Version requirements:**
- `v0.1.0+` — `sino auth` credential management (add, list, remove, guide)
- If `sino auth list` returns "unrecognized subcommand", the CLI needs updating.

Production endpoint: `https://ican.sinocare.com/api/scrm-mcp`

### Parameters

- `user_id` — Auto-read from credential store if configured. Can be overridden
  with `--user-id`. Never invent one; ask the user or guide them with
  `sino auth guide sinocare`.
- Dates — Always `YYYY-MM-DD` format.

### Command Reference

**Health check:**
```bash
sino health --json
```

**User profile:**
```bash
sino user info --json
# or override: sino user info --user-id <USER_ID> --json
```
Returns: `{ user_id, user_info }`

**Single-day CGM readings:**
```bash
sino cgm day --date 2025-02-04 --json
# or override: sino cgm day --user-id <USER_ID> --date 2025-02-04 --json
```
Returns: `{ user_id, data, count, ... }` — array of timestamped glucose readings.

Use when: user asks about a specific day's glucose, wants to see a daily curve.

**Date-range CGM readings:**
```bash
sino cgm range \
  --start-date 2025-02-01 \
  --end-date 2025-02-04 \
  --json
```
Returns: same shape as single-day, but spanning multiple days.

Use when: user wants to compare days, look at trends, or analyze a period.

**Daily aggregate (all signals):**
```bash
sino daily --date 2025-02-04 --json
```
Returns: single-day aggregate JSON with multiple arrays (glucose summary stats,
events, etc.).

Use when: user wants a full picture of one day including events, not just raw
glucose values.

**Event detail:**
```bash
sino event get \
  --event-id <EVENT_ID> \
  --json
```
Returns: `{ event: ... }`

Use when: user references a specific event/record and wants details.

### Output Format

- Use `--json` when you need to process the data (calculate averages, find spikes, etc.)
- Use `--pretty` when the user just wants a human-readable summary

### Error Handling

| Error | Action |
|-------|--------|
| `sino` not installed | `npm install -g @shukrichiu/icancgm-cli`, then retry |
| Version too old / `auth` unrecognized | `npm update -g @shukrichiu/icancgm-cli` to get v0.1.0+ |
| Health check fails | Report error, stop — don't guess |
| No credential configured | Run `sino auth guide sinocare` to show setup steps |
| 400 parameter error | Check date format (`YYYY-MM-DD`) |
| 500 query error | Report backend error text as-is |
| 401/403 auth failure | Credential may be wrong — suggest `sino auth add sinocare --user-id <ID>` |
| Empty response | Tell user no data exists for that user_id + time window |

---

## CGM: Other Brands

### Dexcom

Status: Planned. Currently ask the user to export from Dexcom Clarity and paste
the CSV data or share key numbers.

Typical export columns: timestamp, glucose_value (mg/dL), event_type, event_subtype.

### Freestyle Libre (Abbott)

Status: Planned. Ask the user to export from LibreView or share the PDF report.

Typical data: glucose readings every 15 minutes, daily summary stats (TIR, average,
estimated HbA1c), AGP (ambulatory glucose profile).

### General CGM Data Handling

Regardless of brand, when you receive CGM data:

1. Identify the time range covered
2. Calculate or extract: average glucose, standard deviation, time in range (TIR),
   time above range (TAR), time below range (TBR)
3. Look for patterns: dawn phenomenon, post-meal spikes, overnight stability
4. Cross-reference with the interpretation targets in the Quick Reference section below

---

## Wearables: Smartwatch

### Apple Watch (via Apple Health export)

Status: Planned. Ask the user to export from Apple Health or share specific metrics.

Key data points to look for:
- **Resting heart rate** — lower is generally better (50–70 bpm for healthy adults)
- **HRV (heart rate variability)** — higher is better; indicates recovery capacity
- **Steps / active calories** — activity level for carb timing recommendations
- **Workout sessions** — type, duration, intensity for pre/post nutrition advice
- **Sleep** — duration, stages if available

### Garmin / Fitbit / Samsung

Status: Planned. Similar data structure to Apple Watch. Ask for exports or screenshots.

---

## Wearables: Oura Ring

Status: Planned. Ask the user to share Oura app data or export.

### Key Metrics

| Metric | What it tells us | Nutrition connection |
|--------|-----------------|---------------------|
| Sleep score | Overall sleep quality | Poor sleep → insulin resistance next day; recommend magnesium, limit late carbs |
| Deep sleep % | Recovery depth | Low deep sleep → prioritize glycine, tart cherry, limit alcohol |
| REM sleep % | Cognitive recovery | Low REM → reduce evening caffeine window |
| HRV | Autonomic balance | Low HRV → anti-inflammatory protocol (omega-3, berries, turmeric); avoid alcohol, excess sugar |
| Readiness score | Recovery state | Low readiness → lighter training, higher protein, anti-inflammatory focus |
| Body temperature | Baseline deviation | Elevated → potential illness, increase vitamin C, zinc, hydration |
| Resting HR | Cardiovascular fitness | Elevated from baseline → overtraining or stress; adjust calorie/carb intake up |

---

## Blood Work & Lab Results

Status: **Active** — user pastes lab results into conversation.

When the user shares blood work, extract and interpret these markers using the
optimal ranges below. Prioritize markers that directly connect to actionable
nutrition changes.

### Metabolic Panel

| Marker | Optimal | Concern | Nutrition action |
|--------|---------|---------|-----------------|
| Fasting glucose | 70–85 mg/dL | ≥100 | Meal sequencing, fiber pairing, post-meal walks, TRE |
| HbA1c | <5.4% | ≥5.7% | Comprehensive metabolic optimization protocol |
| Fasting insulin | <5 µIU/mL | >10 | Reduce refined carbs, increase fiber, resistance training |
| Triglycerides | <100 mg/dL | >150 | Reduce sugar/refined carbs, increase omega-3, add fiber |
| HDL | >60 mg/dL | <40 | Increase EVOO, avocado, fatty fish, exercise |
| LDL particle size | Large | Small dense | Reduce refined carbs, increase healthy fats |

### Inflammation

| Marker | Optimal | Concern | Nutrition action |
|--------|---------|---------|-----------------|
| hs-CRP | <1.0 mg/L | >3.0 | Full anti-inflammatory protocol (omega-3, polyphenols, eliminate ultra-processed) |
| Homocysteine | <10 µmol/L | >15 | Methylated B vitamins (B12, folate, B6) |

### Nutrients & Hormones

| Marker | Optimal | Action if low |
|--------|---------|--------------|
| Vitamin D | 50–70 ng/mL | Supplement D3 2000–5000 IU + K2, sun exposure |
| B12 | >500 pg/mL | Methylcobalamin supplement, animal products or fortified foods |
| RBC Magnesium | test-specific | Magnesium glycinate 400–500mg, pumpkin seeds, dark chocolate |
| Omega-3 Index | >8% | Fatty fish 3x/week or supplement 2–3g EPA/DHA |
| Ferritin | 50–150 ng/mL | If low: iron-rich foods + vitamin C; if high: reduce red meat, donate blood |
| Testosterone (men) | 500–900 ng/dL | Sleep, zinc, vitamin D, resistance training, reduce alcohol |

---

## Data Interpretation Quick Reference

### CGM Glucose Targets

| Metric | Optimal | Acceptable | Needs attention |
|--------|---------|------------|-----------------|
| Fasting glucose | 70–85 mg/dL | 86–99 | ≥100 |
| Post-meal peak | <120 mg/dL | 120–140 | >140 |
| Glucose variability (SD) | <30 mg/dL | 30–45 | >45 |
| Time in range (70–140) | >90% | 70–90% | <70% |
| HbA1c | <5.4% | 5.4–5.6% | ≥5.7% |

### Cross-Source Pattern Recognition

The real power comes from combining data across sources. Look for these patterns:

| Pattern | Data sources | What it means | Recommendation |
|---------|-------------|---------------|----------------|
| High glucose + poor sleep | CGM + Oura/watch | Sleep deprivation causes insulin resistance | Fix sleep first: magnesium, sleep hygiene, limit late eating |
| Post-meal spikes + low activity | CGM + watch steps | Sedentary post-meal = bigger spikes | 10-min walk after meals |
| High glucose variability + low HRV | CGM + Oura | Stress-driven metabolic disruption | Stress management + anti-inflammatory nutrition |
| Good glucose + high inflammation (hs-CRP) | CGM + blood work | Glucose control alone isn't enough | Add omega-3, polyphenols, reduce processed foods |
| Low HRV + high resting HR | Oura + watch | Overtraining or chronic stress | Increase calories, reduce training intensity, prioritize recovery nutrition |
| Morning glucose spike + normal meals | CGM | Dawn phenomenon | Evening protein snack, apple cider vinegar before bed, earlier dinner |
