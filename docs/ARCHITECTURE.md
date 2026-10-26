# Architecture

DigiLegacy is a single-process Tauri 2 desktop application. There is no backend service, no database server, and no network call other than the optional heartbeat the user configures themselves.

```
┌──────────────────────────────────────────────────────────────┐
│ WebView (Vue 3 + TypeScript + Pinia)                         │
│   views/ ── stores/ ── api/invoke.ts ──┐                     │
└───────────────────────────────────────┼──────────────────────┘
                                        │ Tauri IPC (45 commands)
┌───────────────────────────────────────▼──────────────────────┐
│ Rust core (src-tauri/src)                                    │
│   commands.rs   thin command layer, input caps, envelopes    │
│   vault.rs      master key in memory, field encrypt/decrypt   │
│   crypto.rs     AES-256-GCM + PBKDF2 primitives               │
│   recovery.rs   one-time recovery key seal                    │
│   db.rs         schema, migrations, all SQL                   │
│   fsutil.rs     0600 file writes (backups / reports / netrc)   │
│   notify.rs     dead-man's-switch evaluation + outbox + SMTP  │
│   heartbeat.rs  zero-content heartbeat over curl              │
│   backup.rs     daily snapshot, 7-day retention, atomic write │
│   platform_data.rs  built-in 152-platform policy database     │
└───────────────────────────────────────┬──────────────────────┘
                                        │ rusqlite (bundled SQLite, WAL)
                                        ▼
                             %APPDATA%/com.digilegacy.app/digilegacy.db
```

## Layering rules

1. **`commands.rs` is a thin adapter.** It validates IPC input (size caps, enum values, existence checks), wraps results in `ApiResponse<T>`, and delegates to `db::*`. Business logic and SQL belong in `db.rs`; cryptographic primitives belong in `crypto.rs` / `vault.rs` / `recovery.rs`.
2. **Frontend never talks to the filesystem or SQLite directly.** Every side effect goes through `invoke()`. The only exception is downloading a file in browser-preview mode, which the mock handles.
3. **`api/invoke.ts` is the single IPC seam.** In a Tauri webview it calls `@tauri-apps/api/core`; in `npm run dev` — and in the `VITE_DEMO=1` static build used for the website demo — it lazily loads `api/mock.ts`. The mock must implement the same command names and response shape; it is the only thing E2E and the public demo talk to. In a normal production build that branch is dead code, so Rollup drops `mock.ts` from the installer bundle entirely (verify: `ls dist/assets | grep mock` returns nothing after `npm run build`).
4. **`db.rs` owns the connection.** A process-global `Mutex<Option<Connection>>` holds one WAL connection with `busy_timeout=5000`. Tests serialize on `vault::test_lock()` because both the connection and the master key are process-global. The schema version lives in `PRAGMA user_version` (`SCHEMA_VERSION`), so a future migration can branch on a number instead of re-deriving intent from `PRAGMA table_info`.

## Data flow: saving an asset

```
DiscoverView form
  → assets store addAsset()
    → invoke('add_asset', { req })
      → commands::add_asset
        → db::add_asset_db
          → validate_category / validate_disposal            (reject unknown enums)
          → ensure_vault_writable                            (fail closed when locked)
          → vault::encrypt_if_needed per sensitive field
          → INSERT INTO digital_assets
```

Reading reverses it: `row_to_asset` calls `vault::decrypt_or_placeholder`, which returns a placeholder string rather than failing the whole list when one row cannot be decrypted (for example, while locked). Credential reads are stricter: `get_asset_credential_db` returns `Err` while locked instead of an empty string, because an empty string is indistinguishable from "never saved".

## Encryption model

| Concern | Where | Notes |
|---|---|---|
| Key derivation | `crypto.rs` | PBKDF2-HMAC-SHA256, 600k iterations for new vaults; salt carries a `v2.` prefix so old 100k salts still resolve. |
| Field encryption | `vault.rs` | `enc:v1:` + base64(nonce ‖ ciphertext), one random 12-byte nonce per value. |
| Key lifetime | `vault.rs` | `Zeroizing<[u8; 32]>` in a `Mutex<Option<_>>`; cleared on lock and on idle timeout. |
| Recovery | `recovery.rs` | The master key itself is encrypted under a key derived from a 24-character code. Changing the password rotates the code and re-encrypts every field in one transaction. |
| Re-encryption | `db.rs` | Three paths share `PROFILE_SENSITIVE_COLUMNS` and the asset column list: enabling encryption, changing the password, and recovery reset. They must stay in lockstep — a column added to one path only is a data-loss bug. |

## Failure isolation

- The background loop (`lib.rs::spawn_background_loop`) wraps each round in `catch_unwind`: a panic in DMS evaluation or heartbeat is logged and the next round still runs. Without this, one panic silently kills the reminder chain and the user believes they are still protected.
- Daily backup failure never blocks startup.
- Heartbeat and SMTP failures degrade to the local outbox rather than surfacing as errors to the user.

## Testing strategy

| Layer | Tool | What it proves |
|---|---|---|
| Rust units | `cargo test` | SQL, encryption round-trips, migration, re-encryption, DMS staging, SMTP header sanitisation, backup retention. |
| Frontend units | Vitest + jsdom | Store behaviour, view rendering, error paths, accessibility attributes. Coverage thresholds are enforced over all of `src/**`. |
| Scenario matrices | `scenarios.rs` (Rust) + `scenarios.test.ts` (Vitest) | Combination sweeps that assert invariants after every step rather than per-function: 246 backend combinations / 2824 checkpoints over real SQLite + AES, and ~339 frontend combinations / 1692 checkpoints. These caught the missing backup-import validation and the negative `totalValue` sum. |
| E2E | Playwright (browser preview) | Real component tree, router, and store interactions against the mock backend. Catches view-layer regressions unit tests miss. `e2e/scenarios.spec.ts` adds 39 matrix scenarios (8 routes × 2 locales × 2 themes, 20 search keywords, 4×4 filter/sort, etc.). |
| Release | CI `release-build` + `release.yml` | The installers actually build, and version numbers agree across three manifests. |

E2E does not feed the v8 coverage numbers. The two are complementary: unit coverage measures branch breadth, E2E measures whether the screens actually work.

## DMS: read vs. dispatch

`notify::status_snapshot` (`get_dms_status`) only reads `dms_config` and computes the stage — no SMTP, no outbox write. `notify::evaluate_dms` is the dispatching path and is only called by the background loop. The UI must never call the dispatcher: opening a page used to send real mail. `get_dms_history` reads the newest 50 entries from `dms_outbox/*.jsonl`, and `get_check_in_status` returns `recent_check_ins` (latest 5 confirmations) so the check-in log is actually read by the UI, not just written.

## Backup format

`version: 3` adds `security_config.recovery_seal` to the export so a recovery code survives a machine move. Imports accept v1/v2/v3. A backup **without** `security_config` is treated as plaintext: importing it into an encrypted vault is rejected while locked and field-encrypted while unlocked — plaintext must never land in an `enc:v1:` column.

## Known limits

- The desktop UI ships in Simplified Chinese and English; `src/i18n/index.ts` holds both dictionaries, `setLocale` also updates `<html lang>`, and a unit test asserts the two key sets are identical. Legal template bodies are Chinese-only by design (they are jurisdiction-specific documents).
- Migrations are still additive `ALTER TABLE` checks; `PRAGMA user_version` now records the baseline (`SCHEMA_VERSION = 2`), so the first destructive migration should branch on it rather than on column presence.
- `db.rs` is the largest module (~3.4k lines). Platform queries, asset CRUD, profile, security, backup, and DMS all live there. Splitting it by domain is the highest-value refactor left, but it is not urgent while the test suite covers the seams.
- The public demo is a static build of the real UI over the mock backend. It is a showcase, not a security boundary: its data lives unencrypted in `localStorage` and the banner in the UI says so.
