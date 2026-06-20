# UE reference snapshots (local only)

Epic **Unreal Engine** ships default config templates as `BaseEngine.ini` and `BaseScalability.ini`.
Game Settings Master uses them **only as a local build input** — not shipped as Epic source trees.

## Supported UE versions

`4.27`, `5.0`, `5.1`, `5.2`, `5.3`, `5.4`, `5.5`, `5.6`, `5.7`, `5.8`

Committed **fixtures** (CI/bootstrap): `fixtures/UE_4.27`, `fixtures/UE_5.4`.  
Full coverage: fetch from your Epic git clone (see below).

## Layout

```
tools/ue-reference/
  README.md
  fixtures/          # small committed samples (CI / bootstrap)
    UE_4.27/
    UE_5.4/
  UE_4.27/           # your local snapshots (gitignored)
    source/          # Scalability.cpp, GameUserSettings.h/.cpp snippets
  UE_5.0 … UE_5.8/
```

## Fetch with git checkout per release tag

Requires a local **Unreal Engine git clone** (Epic account). **Do not commit the engine repo.**  
Setup guide: [`docs/epic-clone-setup.md`](../../docs/epic-clone-setup.md)

```powershell
# Auto-detect clone (D:\, C:\, UE_ENGINE_ROOT, %USERPROFILE%\UnrealEngine) + tags 4.27–5.8
.\scripts\fetch-ue-reference.ps1 -AutoTags

# Or explicit path
.\scripts\fetch-ue-reference.ps1 -EngineRoot "D:\UnrealEngine" -AutoTags
```

The script checks out each release tag, copies `Engine/Config/BaseEngine.ini` + `BaseScalability.ini` into `tools/ue-reference/UE_X.Y/`, copies `Scalability.cpp` + `GameUserSettings.h/.cpp` into `source/`, then restores your original branch.

## Rebuild catalog

```powershell
python tools/ue-catalog-builder/extract_sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract_gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
```

Output:

| File | Purpose |
|------|---------|
| `src-tauri/catalog/ue_reference_index.json` | Schema v2: `introduced_in`, `removed_in`, `min`/`max`, tier A RU/EN |
| `src-tauri/catalog/generated/merge_stats.json` | Build stats |

## Version-aware UI

Advanced Editor uses `GameProfile.engine_version` (auto-detected) to filter reference metadata:
- UE5-only keys (e.g. Nanite) hidden from reference layer on UE4 games
- Keys **present in your ini** are always shown (even if not in catalog)

Curated human descriptions in `engine.json`, `scalability.json`, etc. **always win** on collision.

## License

- Epic engine source/ini templates remain under Epic’s license.
- The installer bundles **generated JSON only**, not `Base*.ini`.
