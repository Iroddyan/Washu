# 和習 (Washū)

Washū is a native, offline-first Japanese learning workstation for Linux.

## Development

The application stores its database in the XDG data directory (normally
`~/.local/share/washu/washu.db`). Configuration and cache directories follow
the equivalent XDG conventions.

Run the application with:

```sh
cargo run
```

Run the tests with:

```sh
cargo test
```

## v0.1 persistence closeout

Before starting the kanji work, complete this short manual check against a
local build:

1. Add a vocabulary entry, start a review session, reveal it, and select a
   grade.
2. Confirm the next card loads (or the completed-session message appears).
3. Return to Home and confirm the reviewed-today and learning progress figures
   have refreshed.
4. Quit and restart Washū. Confirm the same Home figures and the card's
   scheduled state remain present.
5. Create a backup from Settings and confirm that the displayed success message
   names a new timestamped SQLite snapshot in the shown backup directory.

The automated suite covers the same grade → restart → Home-summary path and
verifies that a backup preserves the graded state and review record. The
migration runner is also checked by reopening a v0.1 database and confirming
that its initial migration remains recorded exactly once.
