# Unreal Engine improvements — version 1.3.0

Version **1.3.0** combines the implementation planned for stages 1.1, 1.2 and 1.3.
Compatibility profiles, Enhanced Input and other engines remain deferred. The
validation record below distinguishes automated and local hardware checks from
installation and hardware scenarios that still need additional coverage.

## Implemented scope

| Planned stage | Implementation |
| --- | --- |
| 1.1 | Prepared changes with confirmation, grouped selections, stale-context rejection, rollback, lossless ordered INI documents, named snapshots, comparisons, parameter/file/full restore, v2 presets and legacy JSON import |
| 1.2 | DXGI hardware enumeration, dedicated/shared memory, independent D3D12 ray tracing checks, per-game GPU selection, persisted diagnostics, background process observation and post-exit file stabilization |
| 1.3 | GOG registry/local metadata, accessible current-user Xbox packages and XboxGames, package activation or validated EXE launch, structured classic Input.ini bindings |

Existing hints for Subnautica 2, Palworld and PUBG remain in place. New profile,
preset and backup fields have compatible defaults. Unselected parameters,
comments and other array entries are preserved.

## Change transaction

`prepare_changes` returns an opaque identifier, operations with stable group IDs,
warnings/errors and SHA-256 revisions for all six supported files. The backend
holds original bytes, profile identity, active config folder and selected GPU
capabilities. The client supplies only selected operation IDs to
`apply_prepared_changes`; it cannot submit replacement bytes.

Plans expire after ten minutes. A changed profile, GPU, folder or file requires a
fresh preview. Linked rows are indivisible and selected sets are revalidated.
No-op selections neither write files nor create snapshots. Preparation, comparison,
reading and backup creation preserve source modification times and read-only
attributes. Writes are atomic per file, preceded by a backup; failure restores
only the affected files, including removal of newly created files. Multi-file
commit is not crash-atomic: the backup remains available after process/power loss.

The INI document retains order, repeated sections/keys, comments and line endings.
Supported encodings are UTF-8, UTF-8 BOM and UTF-16 LE BOM. Malformed UTF-16 and
unsupported encodings are rejected. Scalar editors retain their adapter view;
array operations use individual Input entry IDs. Scalar updates change all
occurrences of the selected key, and the preview includes each changed occurrence.

Whole-file restore returns exact snapshot bytes. Parameter restore preserves other
lines, removes keys absent from the snapshot subject to protections, and refuses
ambiguous repeated keys. Array restoration is available through whole-file restore.
A trusted snapshot can recover a deleted GameUserSettings.ini in an owned folder;
ordinary edits and initial manual folder binding still require a valid config.

Preset format 2 stores description, source game, engine, game build and hardware
requirements. Full profiles set saved editable values without replacing unrelated
target settings. Input profiles use per-action occurrence IDs rather than source
line offsets. Unknown hardware or mismatched metadata produces a warning.

## GPU and observation limits

GPU selection controls filters and validation. It does not assign a GPU to the
game process; the UI opens Windows graphics settings for that purpose. Plugin
availability is separate from hardware: enabling DLSS/FSR/XeSS/frame generation
requires an explicit warning acknowledgement when game support is unverified.

Diagnostics run only while GSM is open. Process matching uses executable path,
PID and creation time, including Shipping executables within the installation.
Missing identity/access produces an explicit unknown result. Files are sampled
every five seconds and require two stable reads, with a thirty-second settling
limit. The last report and expected applied values survive restart. Reports state
that changes were detected, without attributing them to a particular writer.
Input formatting changes in numbers, booleans, quoting or field order are compared
by value. A failed profile/baseline save after a successful config write is reported
as a warning and does not pretend the config transaction failed.

## Store and input coverage limits

GOG uses installed registry records plus bounded `goggame-*.info` metadata.
Primary executable tasks with unsupported arguments are not auto-imported; the
user can select an EXE. Default working directory is the installation root; an
explicit metadata working directory must remain inside that installation.

Xbox coverage is accessible current-user packages and flat-file installations.
Registered packages launch through Windows. Protected/unavailable installations
may be omitted; source failures do not erase results from other stores. No access
rights are changed to inspect protected folders.

Classic ActionMappings, AxisMappings and AxisConfig fields already present in
Input.ini can be edited. Unknown fields remain intact. Inherited arrays without a
known local reset, removal operations and entries superseded by later resets are
read-only. Selected changes that would merge `+` entries are blocked. A simultaneous
binding swap is validated as a complete selected set. Enhanced Input and game-specific
binding schemes are outside this release.

The controls tab displays the resolved Input.ini path and distinguishes a missing
file, an empty file (including whitespace/BOM only), and a file without classic
bindings. PUBG's CustomInputSettins in GameUserSettings.ini is detected as a separate
unsupported format; an empty Input.ini does not provide editable PUBG bindings.

Editor headers, warnings and preset metadata have bounded scroll regions. Expanding
preset options keeps the save/apply controls within the available editor height,
including short windows with large text and configuration conflict warnings.

## Validation and remaining coverage

Automated checks cover preview/commit equivalence, cancellation, exclusions,
linked sets, stale files/profile/GPU, injected write failure, exact-byte restore,
partial restore, duplicate INI entries, empty versus missing values, encoding,
Input occurrence identity, unknown GPU capabilities and discovery failure isolation.
Browser scenarios cover normal application/restore plus GPU selection, Input
partial application, named snapshots, legacy preset import and Russian UI.

Local validation on 2026-10-03: 238 Rust tests passed (one hardware-session test
is intentionally ignored by the normal suite), 206 TypeScript tests and 21
Playwright scenarios passed. Catalog builder tests (12), frontend production
build, Clippy with warnings denied, Rust formatting, locale parity, generated
validation index and package alignment checks passed.

The local Windows session confirmed enumeration and D3D12 ray tracing on an
NVIDIA RTX 4060 Ti and integrated AMD Radeon. Intel selection/unknown capabilities
are covered by fixtures; physical Intel hardware has not been checked.

The following installation and hardware scenarios still need additional validation:

- Apply, exclusions, config rewrite after a real foreground/background game run,
  folder migration and partial/full restoration on actual supported game installs.
- Physical AMD/Intel/NVIDIA adapters, hybrid laptops, GPU disappearance, unknown
  feature checks, and dedicated/shared memory reporting.
- Actual GOG/Xbox installations, inaccessible package locations, native package
  activation, multiple manual EXEs, and classical Input inheritance/array behavior.
- Test installation and upgrade from 1.0.8, preserving saved
  profiles, presets, backups and metadata; validate updater signature and install.

`Release app` accepts `validation_only=true` on a branch to build a signed updater
installer as an Actions artifact. This mode does not move tags, create a GitHub
Release, or publish `latest.json`. Prerelease versions are blocked from stable
publication. This is the existing updater signature, not an Authenticode publisher
certificate.

## Primary references

- [D3D12 ray tracing options](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/ns-d3d12-d3d12_feature_data_d3d12_options5)
- [Unreal Engine configuration array semantics](https://dev.epicgames.com/documentation/en-us/unreal-engine/configuration-files-in-unreal-engine)
- [Xbox Flat File Install](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/packaging/packaging-flatfileinstall)
- [GOG executable tasks and working directories](https://docs.gog.com/bc-file-tasks/)
- [Manual workflow dispatch by branch](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow?tool=cli)
