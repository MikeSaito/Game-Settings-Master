# Release and configuration recovery

This change starts from release `v1.4.0` (`6bfa20a`). The prepared-change commands
keep ownership of the final encoded bytes, revision checks, grouped selection,
and preview expiration. `changes/` separates DTOs, cache, preview preparation,
semantic checks, INI encoding, rendering, and application.

## Configuration transactions

Applying a preview (including text edits and snapshot restoration) and resetting
overrides use `transaction::execute`. The executor serializes writers across
threads and processes and rechecks the preview's original bytes under its lock.
It saves exact original bytes, their SHA-256 hashes, file absence and Windows DOS
attributes before modifying any config. Each file is replaced atomically.

Journals live in `UESettingsMaster/transactions` under the application's existing
data directory. Preparation uses `.preparing-<UUID>` directories. These can never
have modified targets and are ignored when scanning unfinished writes. A complete
prepared journal is renamed into place before the first config change.

Pending or recovering operations block every transaction affecting their folders.
Startup displays recovery with Restore and Later. Later leaves the block active.
Other folders remain writable. A corrupt journal with untrustworthy target paths
blocks all config writes; it is displayed as an error, never silently discarded.
Restore checks the current game profile, path ownership, symlinks/reparse points,
running game and every backup checksum before restoring the first file. Each
attempt first saves the current files and an inventory in `before-recovery-*`.
Repeated restoration is safe after another process interruption. Completed
journals and snapshots are retained for diagnosis; no cleanup deletes user backups.

This is crash recovery, not an all-files atomic commit or a guarantee against
power loss. A missing or corrupt backup requires repair of the journal store;
the UI deliberately offers no action that discards recovery protection.

## Releases

Both tag and manual release runs reject mismatched package/Cargo/Tauri versions,
an existing release (including a draft), or a tag on another commit. API errors
are fatal. A failed draft is retained for diagnosis; rerunning never replaces it.
Use a new version or explicitly inspect and remove a failed unpublished draft
outside this workflow before retrying. Published tags/assets must never be moved.

The final installer is verified with the public updater key and its exact manifest,
installed and launched on Windows. Only then is a draft created. Its uploaded
assets are downloaded and verified again before publication. The validation-only
mode performs checks and saves a signed artifact without publishing or changing
tags. This verifies Tauri updater signatures, not Windows publisher certificates.

## Tests

Standard checks:

```text
npm run build
npm test
node --test scripts/release-policy.test.mjs
npm run e2e -- --workers=1
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Windows desktop tests require `tauri-driver` and Microsoft Edge Driver matching
the installed Edge. `scripts/install-native-driver.ps1` installs these on CI.

```text
npm exec -- tauri build --debug --no-bundle --features native-test --config native-e2e/tauri.conf.json
node native-e2e/configs.mjs
```

The `native-test` feature isolates profiles and backups and enables explicit crash
checkpoints. Normal application builds contain no environment-based crash controls
or test data-directory override. Config tests use real UI controls, Tauri commands,
files, a forced exit between files, a deferred recovery and actual restoration.
Their updater endpoint is a loopback server returning no update.

On an isolated CI runner, `scripts/build-updater-fixtures.ps1` generates an ephemeral
signing key and two test installers, then `node native-e2e/updater.mjs` verifies
rejection of tampered packages/signatures and installation/restart into the new
version. Its identity and key differ from the production app. Private test keys
are deleted and excluded from uploaded artifacts. The reusable Windows workflow
runs config tests on PRs and also runs updater tests as a prerequisite of release.

`shared/validation-rules.json` defines common combination rules and numeric syntax.
Both real validators consume it and run `shared/validation-cases.json` through
their adapters. Catalog metadata remains the source of ranges and UE compatibility;
the existing generated validation index and its synchronization check remain.
Filesystem/process checks remain server-side; UI loading checks remain client-side.
