# INI text editor

The Files tab edits the selected game's Engine.ini, GameUserSettings.ini,
Scalability.ini, Game.ini and Input.ini. Available since version 1.4.0.

Each file keeps a separate in-memory draft when switching files or editor tabs.
Drafts are discarded when leaving the game editor. Search selects the next match;
Ctrl+S opens a preview of all modified files. Reloading a modified file requires
explicitly discarding its draft. Reading remains available while the game runs;
saving is blocked by both the interface and the backend.

The preview shows the complete before/after text. A file is one selectable
operation so INI array directives and duplicate entries retain their order.
Excluded files keep their drafts. Missing files are created only when saving
nonempty text. There is no file deletion action.

Existing UTF-8, UTF-8 BOM and UTF-16 LE encodings are preserved. Line endings
outside the edited range remain unchanged; inserted newlines use CRLF when the
source contains CRLF, otherwise LF. New files use UTF-8. Unsupported encodings
and files larger than 1 MiB are reported without conversion. Both submitted text
and encoded output are limited to 1 MiB.

Reading is restricted to the five allowed filenames in an authorized game config
folder. The read command returns a byte checksum and resolved folder. Preparation
rejects a changed source checksum or folder. Applying rechecks the existing
prepared-change context, game process, GPU and file revisions. Writes share the
normal backup and rollback transaction. Empty selections write nothing and create
no backup. The diagnostics baseline tracks whole-file checksums without storing
raw text. Manual edits are checked against known scalar validation rules and
require acknowledging that arbitrary INI syntax and game behavior cannot all be
validated automatically.

Raw edits are not exported as structured presets. The normal parameter and Input
editors remain available for structured edits and preset creation.
