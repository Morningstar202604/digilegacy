# Architecture

DigiLegacy is a single-process Tauri 2 desktop application. There is no backend service, no database server, and no network call other than the optional heartbeat the user configures themselves.

```
┌──────────────────────────────────────────────────────────────┐
│ WebView (Vue 3 + TypeScript + Pinia)                         │
│   views/ ── stores/ ── api/invoke.ts ──┐                     │
└───────────────────────────────────────┼──────────────────────┘
                                        │ Tauri IPC (67 commands)
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
| Scenario matrices | `scenarios.rs` (Rust) + `scenarios.test.ts` (Vitest) | Combination sweeps that assert invariants after every step rather than per-function: 2824 backend checkpoints over real SQLite + AES, and 1714 frontend checkpoints across ten matrices. Counts are quoted as checkpoints only — the "combination" definition varies per family, so a single combo total was a drift source. These caught the missing backup-import validation and the negative `totalValue` sum. |
| E2E | Playwright (browser preview) | Real component tree, router, and store interactions against the mock backend. Catches view-layer regressions unit tests miss. `e2e/scenarios.spec.ts` adds 39 matrix scenarios (8 routes × 2 locales × 2 themes, 20 search keywords, 4×4 filter/sort, etc.). |
| Release | CI `release-build` + `release.yml` | The installers actually build, and version numbers agree across three manifests. |

E2E does not feed the v8 coverage numbers. The two are complementary: unit coverage measures branch breadth, E2E measures whether the screens actually work.

## DMS: read vs. dispatch

`notify::status_snapshot` (`get_dms_status`) only reads `dms_config` and computes the stage — no SMTP, no outbox write. `notify::evaluate_dms` is the dispatching path and is only called by the background loop. The UI must never call the dispatcher: opening a page used to send real mail. `get_dms_history` reads the newest 50 entries from `dms_outbox/*.jsonl`, and `get_check_in_status` returns `recent_check_ins` (latest 5 confirmations) so the check-in log is actually read by the UI, not just written.

## Report rendering

`report.rs::render_report_html` is the only place report HTML is produced. Both the single-file export and the ZIP toolkit call it through the `render_report` command, so the preview, the exported file and the toolkit cannot drift. Every user-controlled value passes through `escape_html`; print styles live in the module. `contains_placeholder` refuses to render when any sensitive field is still undecrypted — exporting a report that says "locked" would mislead the heir.

Why not scrape the DOM (the old approach): the page shows placeholders while locked, carries Naive UI's injected styles, and required the ZIP path to duplicate the wrapper markup — which is how the toolkit once shipped a bare fragment.

## Module layout

| File | Responsibility |
|---|---|
| `db.rs` | connection lifecycle, schema, migrations, assets, profile, security, backup, DMS, report queries |
| `db_platforms.rs` | policy-patch import/export/clear, `platforms` mirroring for the asset foreign key |
| `db_tests.rs` | the db regression suite (split out so production code stays readable) |
| `report.rs` | report HTML rendering (escaping, print styles, placeholder refusal) |
| `fsutil.rs` | owner-only file writes/appends |
| `db_custody.rs` | custody locations (where the backup and recovery code live), field-encrypted |
| `action_guide.rs` | the family's printable, platform-grouped action checklist |
| `toolkit.rs` | action-guide assembly + family drill (metadata only, no credentials) |
| `family_faq.rs` | 12 deterministic bilingual FAQ entries embedded in the guide (no LLM, no network) |
| `db_heirs.rs` | multiple heirs + exactly one executor; whole-list replace in a transaction |
| `ai_assist.rs` | optional bring-your-own-key assistant: whitelist-built outbound payload, Rust-side curl, no built-in service |
| `review.rs` | generic review state machine: AI can only advance to `AwaitingHuman`, `Approved` requires a human verdict |
| `review_profiles.rs` | the scene registry — adding a scene is data, not pipeline code |
| `db_review.rs` | append-only review audit trail; stale detection by content fingerprint |
| `notify.rs` | DMS staging, outbox, SMTP (UI config first, env vars as fallback); read-only status snapshot |
| `report.rs` | report HTML rendering (escaping, print styles, placeholder refusal, bilingual) |

## Placeholder is read-only

`vault::decrypt_or_placeholder` degrades undecryptable values to `【加密数据 · 需解锁】` so one bad row cannot fail a whole list. That placeholder is a **display artifact, never data**: `db::reject_placeholder` blocks it in every write path (`update_asset_db`, `save_user_profile_db`), and the asset-detail / profile pages block saving before the IPC call. Without this, a locked read followed by a save re-encrypted the placeholder and destroyed the original ciphertext.

`delete_asset_db` also goes through `ensure_vault_writable`: a locked vault must not permit destruction any more than it permits reading.

## Policy patches and the platform foreign key

`digital_assets.platform_id` references `platforms`. Patches therefore have to be mirrored into `platforms` (`sync_platform_overrides_into_platforms`) or selecting a patched platform fails on insert. Asset queries join `platform_overrides` and select `COALESCE(o.name, p.name)` so the patched name wins. Clearing a patch removes only unused mirror rows; rows still referenced by assets are kept and renamed `（补丁已清除）` so user data is never cascaded away.

## Backup format

`version: 3` adds `security_config.recovery_seal` to the export so a recovery code survives a machine move. Imports accept v1/v2/v3. A backup **without** `security_config` is treated as plaintext: importing it into an encrypted vault is rejected while locked and field-encrypted while unlocked — plaintext must never land in an `enc:v1:` column.

## Content review pipeline

The review pipeline is deliberately **scene-agnostic**. A scene is a `ReviewProfile` (id, title key, criteria, whether AI participates); the pipeline itself contains no business nouns. Today only `platform-policy` is registered — not because the pipeline is policy-specific, but because other candidate scenes (legal templates, family notes, asset classification) each need a product decision about *what* to send and *by which criteria* to judge. Registering one is a function plus a line in `all_profiles()`.

Three invariants are enforced by types rather than comments:

1. **AI never approves.** `ReviewStatus::from_verdicts` takes `Option<&HumanVerdict>`; `AiVerdict::Pass` can only reach `AwaitingHuman`. There is no code path where an AI verdict yields `Approved`.
2. **Graceful degradation.** With no endpoint configured, `run_ai_review` records `Skipped` and leaves items in the human queue — it does not error, and it does not fabricate a verdict.
3. **Append-only trail.** Every AI run and every human decision appends a `review_records` row carrying the content fingerprint, both verdicts and the actor. Records are never updated, only added.

Content changes invalidate prior decisions: `summarise_states_db` compares each item's current fingerprint against its latest record and reports the item as `stale` (back to pending) when they differ, so a review can never apply to text it did not see.

## Known limits

- The desktop UI ships in Simplified Chinese and English; `src/i18n/index.ts` holds both dictionaries, `setLocale` also updates `<html lang>`, and a unit test asserts the two key sets are identical. Legal template bodies are Chinese-only by design (they are jurisdiction-specific documents).
- Migrations are still additive `ALTER TABLE` checks; `PRAGMA user_version` now records the baseline (`SCHEMA_VERSION = 4`), so the first destructive migration should branch on it rather than on column presence.
- `db.rs` is still the largest module (1882 lines): connection lifecycle, asset CRUD, profile, security, backup, and DMS live there. Platform patches already moved to `db_platforms.rs` (293 lines) and the regression suite to `db_tests.rs` (1722 lines). Splitting the remaining domains is the next step, but it is not urgent while the test suite covers the seams.
- The public demo is a static build of the real UI over the mock backend. It is a showcase, not a security boundary: its data lives unencrypted in `localStorage` and the banner in the UI says so.
