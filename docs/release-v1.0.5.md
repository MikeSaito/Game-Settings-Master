# Game Settings Master v1.0.5 — Release Notes

[Русский](#русский) · [English](#english)

---

## Русский

**Дата:** 2026-07-06  
**База:** v1.0.4 → v1.0.5 (14 коммитов на `main`)

### Apply — валидация

**Rust (`semantic_validation.rs`)** — проверка перед `apply_custom_cmd` и `apply_game_override`:

| Код | Уровень | Что |
|-----|---------|-----|
| `sg_exceeds_limit` | error | sg выше лимита игры |
| `version_mismatch` | error | параметр не для версии UE игры |
| `shipped_key_removal` | error | удаление ключа, который был в ini при загрузке (только GUS) |
| `sg_r_conflict` | warning | sg.* и r.* на одну группу (диск + pending) |
| `combo_engine_scalability_dup` | warning | один ключ в Engine.ini и Scalability.ini |
| `combo_rt_shadows` | warning | RT + тени |
| `combo_rt_no_hw` | warning | RT без GPU RT |
| `combo_streaming_texture` | warning | streaming pool + texture quality |
| `value_out_of_range` | warning | значение вне min/max из каталога |

Без `warnings_acknowledged: true` apply с warnings блокируется. Удаление shipped `r.*` из Engine.ini — разрешено.

**Frontend** — `src/lib/editor/validation/`, панель `ApplyValidationPanel` (ошибки / предупреждения / чекбокс подтверждения). Basic и Advanced.

### Редактор

- `ParameterDetailPane` — боковая панель с описанием выбранного параметра (Advanced).
- `ExtraIniPanel` — просмотр Input.ini / DeviceProfiles.ini (read-only, виртуальный список).

### Каталог

- Overlay match: точное совпадение slug вместо `contains` (чтобы короткое имя игры не цепляло чужой overlay).
- Dedupe дублей секций в каталоге — тесты в `dedupe.rs`.
- Обновлены tier_a/b descriptions и expansion JSON в catalog builder.
- `shared/ue-validation-index.json` — индекс min/max для валидации; CI проверяет sync с `ue_reference_index.json`.

### E2E

Было 3, стало **5** (`e2e/apply-restore.spec.ts`, фикстуры `e2e/fixtures.ts`):

1. scan → apply basic → restore backup  
2. sg/r conflict → reset r.* → apply  
3. reset override ini из backups  
4. **sg выше лимита → apply disabled**  
5. **есть warnings → чекбокс → apply enabled**

### CI

- `npm run validation:index:check`
- `npm run i18n:parity` (9 namespaces en/ru)
- `npx tauri build --no-bundle`
- Исправлен bash-шаг версии в `ci.yml` (landing-build job)
- `package-lock.json` синхронизирован под Node 22 / `npm ci` (emnapi 1.11.2)

### Зависимости

| Пакет | Было → стало |
|-------|----------------|
| tauri | 2.11.3 → 2.11.5 |
| vite | 7.x → 8.1.3 |
| @vitejs/plugin-react | 5.x → 6.0.3 |
| vitest | 3.2.6 → 4.1.9 |
| lucide-react | 0.562 → 1.23 |
| cross-env | 7.0.3 → 10.1.0 |
| open (Rust) | 5.3.5 → 5.3.6 |

### Проверки перед релизом

| Проверка | Результат |
|----------|-----------|
| CI (`main`, run 28779768645) | ✅ |
| Deploy landing | ✅ |
| `npm test` | 176 |
| `cargo test` | 179 |
| `npm run e2e` | 5/5 |
| `npm run catalog:test` | 12 |

### Версии

| Файл | Версия |
|------|--------|
| `package.json` | 1.0.5 |
| `src-tauri/Cargo.toml` | 1.0.5 |
| `src-tauri/tauri.conf.json` | 1.0.5 |
| `landing/package.json` | 1.0.5 |
| README / landing | 1.0.5 |

### Установщик

`Game-Settings-Master_1.0.5_x64-setup.exe` — GitHub Actions **Release app** (`workflow_dispatch`).

### Основные пути

| Область | Пути |
|---------|------|
| Серверная валидация | `src-tauri/src/commands/helpers/semantic_validation.rs` |
| Клиентская валидация | `src/lib/editor/validation/` |
| UI | `ApplyValidationPanel.tsx`, `ParameterDetailPane.tsx`, `ExtraIniPanel.tsx` |
| Overlay / dedupe | `src-tauri/src/catalog/overlay.rs`, `dedupe.rs` |
| E2E | `e2e/apply-restore.spec.ts`, `e2e/fixtures.ts` |
| CI scripts | `scripts/verify-validation-index.mjs`, `scripts/verify-i18n-parity.mjs` |

---

## English

**Date:** 2026-07-06  
**Base:** v1.0.4 → v1.0.5 (14 commits on `main`)

### Apply validation

**Rust (`semantic_validation.rs`)** runs before `apply_custom_cmd` and `apply_game_override`:

| Code | Level | What |
|------|-------|------|
| `sg_exceeds_limit` | error | sg above game limit |
| `version_mismatch` | error | parameter not for game UE version |
| `shipped_key_removal` | error | removing a key present at load (GUS only) |
| `sg_r_conflict` | warning | sg.* and r.* on same group (disk + pending) |
| `combo_engine_scalability_dup` | warning | same key in Engine.ini and Scalability.ini |
| `combo_rt_shadows` | warning | RT + shadows |
| `combo_rt_no_hw` | warning | RT without RT-capable GPU |
| `combo_streaming_texture` | warning | streaming pool + texture quality |
| `value_out_of_range` | warning | value outside catalog min/max |

Apply with warnings blocked unless `warnings_acknowledged: true`. Removing shipped `r.*` from Engine.ini is allowed.

**Frontend** — `src/lib/editor/validation/`, `ApplyValidationPanel` (errors / warnings / ack checkbox). Basic and Advanced.

### Editor

- `ParameterDetailPane` — side panel with selected parameter details (Advanced).
- `ExtraIniPanel` — read-only view of Input.ini / DeviceProfiles.ini (virtual list).

### Catalog

- Overlay match: exact slug instead of `contains` (short game id must not pick up another game's overlay).
- Catalog section dedupe — tests in `dedupe.rs`.
- Updated tier_a/b descriptions and expansion JSON in catalog builder.
- `shared/ue-validation-index.json` — min/max index for validation; CI checks sync with `ue_reference_index.json`.

### E2E

Was 3, now **5** (`e2e/apply-restore.spec.ts`, fixtures `e2e/fixtures.ts`):

1. scan → apply basic → restore backup  
2. sg/r conflict → reset r.* → apply  
3. reset override ini from backups  
4. **sg above limit → apply disabled**  
5. **warnings present → checkbox → apply enabled**

### CI

- `npm run validation:index:check`
- `npm run i18n:parity` (9 namespaces en/ru)
- `npx tauri build --no-bundle`
- Fixed bash version step in `ci.yml` (landing-build job)
- `package-lock.json` synced for Node 22 / `npm ci` (emnapi 1.11.2)

### Dependencies

| Package | Was → now |
|---------|-----------|
| tauri | 2.11.3 → 2.11.5 |
| vite | 7.x → 8.1.3 |
| @vitejs/plugin-react | 5.x → 6.0.3 |
| vitest | 3.2.6 → 4.1.9 |
| lucide-react | 0.562 → 1.23 |
| cross-env | 7.0.3 → 10.1.0 |
| open (Rust) | 5.3.5 → 5.3.6 |

### Pre-release checks

| Check | Result |
|-------|--------|
| CI (`main`, run 28779768645) | ✅ |
| Deploy landing | ✅ |
| `npm test` | 176 |
| `cargo test` | 179 |
| `npm run e2e` | 5/5 |
| `npm run catalog:test` | 12 |

### Versions

| File | Version |
|------|--------|
| `package.json` | 1.0.5 |
| `src-tauri/Cargo.toml` | 1.0.5 |
| `src-tauri/tauri.conf.json` | 1.0.5 |
| `landing/package.json` | 1.0.5 |
| README / landing | 1.0.5 |

### Installer

`Game-Settings-Master_1.0.5_x64-setup.exe` — GitHub Actions **Release app** (`workflow_dispatch`).

### Key paths

| Area | Paths |
|------|-------|
| Server validation | `src-tauri/src/commands/helpers/semantic_validation.rs` |
| Client validation | `src/lib/editor/validation/` |
| UI | `ApplyValidationPanel.tsx`, `ParameterDetailPane.tsx`, `ExtraIniPanel.tsx` |
| Overlay / dedupe | `src-tauri/src/catalog/overlay.rs`, `dedupe.rs` |
| E2E | `e2e/apply-restore.spec.ts`, `e2e/fixtures.ts` |
| CI scripts | `scripts/verify-validation-index.mjs`, `scripts/verify-i18n-parity.mjs` |
