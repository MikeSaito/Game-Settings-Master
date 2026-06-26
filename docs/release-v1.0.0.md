# Game Settings Master v1.0.0 — Release Notes

[Русский](#русский) · [English](#english)

---

## Русский

**Дата:** 2026-06  
**Каталог:** 548 bundled reference keys (fixtures UE 4.27 + 5.4). Полный каталог 1000+ — [`docs/epic-clone-setup.md`](epic-clone-setup.md).

### Advanced Editor

- **Две зоны:** «Масштабируемость» (sg.*, GUS) и «Engine» (экспертный режим)
- **Expert warning** RU/EN на вкладке Engine + badge «Эксперт»
- **Фильтр «Рекомендуемые»** — `catalog_recommended` + эвристика; default ON
- **Tier hints** для sg.*Quality — что меняет Low/Medium/High/Epic (из `scalability_tiers`)

### Каталог

| Слой | Кол-во |
|------|--------|
| Tier A overlays | 83 |
| Tier B overlays | 150 |
| Reference entries | 548 |
| scalability_tiers | 50 |

- Schema v2: `introduced_in`, version filter по `engine_version`
- Merge order: curated > tier A > tier B > auto

### merge_stats (bundled)

```json
{
  "sources": ["UE_4.27", "UE_5.4"],
  "total_reference_entries": 548,
  "tier_a_overlays": 83,
  "tier_b_overlays": 150,
  "scalability_tiers": 50
}
```

### Прочее

- UE-only (Unity / presets / ReShade удалены из v1.1+)
- GPU-aware clamp, бэкапы перед apply

---

## English

**Date:** 2026-06  
**Catalog:** 548 bundled reference keys (UE 4.27 + 5.4 fixtures). Full 1000+ catalog — [`docs/epic-clone-setup.md`](epic-clone-setup.md).

### Advanced Editor

- **Two zones:** Scalability (sg.*, GUS) and Engine (expert mode)
- **Expert warning** RU/EN on Engine tab + Expert badge
- **Recommended filter** — `catalog_recommended` + heuristics; default ON
- **Tier hints** for sg.*Quality — UE preset tier CVars (from `scalability_tiers`)

### Catalog

| Layer | Count |
|-------|-------|
| Tier A overlays | 83 |
| Tier B overlays | 150 |
| Reference entries | 548 |
| scalability_tiers | 50 |

- Schema v2, version-aware filter via `engine_version`
- Merge order: curated > tier A > tier B > auto

### merge_stats (bundled)

Same as above — 548 keys, 2 fixture sources.

### Other

- UE-only product
- GPU-aware clamp, backups before apply
