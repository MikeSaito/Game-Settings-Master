# UE catalog builder

Parses local `BaseEngine.ini` / `BaseScalability.ini` snapshots into `src-tauri/catalog/ue_reference_index.json`.

```powershell
python tools/ue-catalog-builder/extract/sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract/gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
```

## Layout

- `build.py` — parser, multi-version merge, supplemental CVars
- `extract/` — UE source extractors (`sg_from_cpp.py`, `gus_from_header.py`)
- `tier_a/` — tier A expansion generators and v6 family modules
- `analysis/` — catalog gap and coverage scripts
- `fixtures/` — regenerate committed sample ini under `tools/ue-reference/fixtures/`
- `shared/` — `ue_versions.json` and loader
- `data/supplemental_cvars.json` — extra well-known CVars when snapshots are small
- `generated/sg_registry_merged.json` — generated `sg.*` registry with version bounds
- `generated/gus_registry_merged.json` — generated display/user settings registry
- `test_build.py` — unit tests

### Tier overlays

| File | Purpose |
|------|---------|
| `data/tier_a_descriptions.json` | Human RU/EN from curated catalog (~83 keys) |
| `data/tier_b_descriptions.json` | Semi-human for top reference-only CVars (~150 keys) |

Merge order: **curated JSON > tier A > tier B > auto-generated**.

Reference entries include `catalog_recommended: bool` (curated keys, tier A/B, sg.*).
