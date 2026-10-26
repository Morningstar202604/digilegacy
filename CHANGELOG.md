# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **备份格式 v3**: exports now carry the `recovery_seal`, so a recovery key still works after moving to a new machine. v1/v2 backups remain importable; importing a backup without a seal clears the stale local one.
- **Read-only DMS status + notification history**: the dashboard and the Legacy Care page now call `get_dms_status` (no SMTP, no outbox write) and `get_dms_history` (latest 50 entries from the local outbox), so past notifications stay visible after a refresh.
- **Auto-backup status** in Profile: snapshot count, latest file name and size, with an explicit "no snapshot yet" state and a visible error path.
- **English money formatting**: amounts now render as `123.5k` / `2.5M` in the English UI instead of reusing the Chinese 万 divisor.
- **Per-user file permissions** for backups, reports and the temporary SMTP netrc (`0600` on Unix) via a shared `fsutil::write_private`.
- **Bilingual UI is now real**: the header language switch drives the whole desktop UI, the dictionary key sets are asserted equal in tests, and `<html lang>` follows the selected locale.
- **Frontend visual pass**: Naive UI theme bridge and every emoji icon replaced by Ionicon / FontAwesome vectors through the new `PlatformIcon.vue` (24+ platforms, 16 categories); `main.css` gained the warm-ambient / glass-header / pill-nav / layered-card / dark-mode layers.

### Fixed

- **Opening the dashboard could send real email.** The frontend called `evaluate_dms`, which dispatches SMTP mail and writes the outbox; merely viewing a page performed a notification round. UI now reads the read-only status.
- **Plaintext backups could bypass encryption.** Importing a backup without `security_config` into an encrypted vault wrote plaintext into columns that must hold `enc:v1:` ciphertext; the import is now rejected while locked and encrypts field-by-field while unlocked.
- **Remaining days were truncated.** `num_days()` made "20 hours left" render as 0 days and the UI treated it as overdue; the value now rounds up.
- **Policy patch and backup import left stale platform caches.** `platformsExtended` was never invalidated, so the heir guide kept showing pre-patch policy text.
- **The ZIP toolkit shipped a broken report.** `01-继承报告.html` was an in-page fragment without `<!DOCTYPE html>` or print styles; it now uses the same standalone document as the single-file export.
- **`set_disposal_wish` bypassed every input check** (arbitrary length text, invalid enum) even though it writes straight to the asset table.
- **`import_backup` and `export_report_to_file` had no size caps**, unlike every other IPC entry point.
- **Heartbeat endpoint validation could be bypassed** through environment variables; non-http(s) schemes are now rejected inside `post_heartbeat`.
- **Report templates opened a blank WebView window** (`target="_blank"`) instead of the system browser; the form link now goes through `open_external`.
- **Chinese dictionary no longer carries emoji in UI strings**, and the English dictionary no longer has keys the Chinese one lacks.
- **DiscoverView step indicator** rendered raw dictionary keys (missing `t()`); route titles are resolved through the testable `resolveTitleKey`.

### Scenario simulation (2026-10-26)

- **Backend scenario suite** (`src-tauri/src/scenarios.rs`): 246 combinations / **2824 checkpoints** over real SQLite + real AES — asset lifecycle (3 categories × 7 disposals × 3 value shapes × encrypted/plaintext), backup round-trips (30 rounds incl. v1 compatibility and 4 malformed inputs), credential vault (20 assets × encrypted/plaintext), DMS timeline (7 intervals × 10 offsets) and policy patches (30 rounds).
- **Frontend scenario suite** (`src/scenarios.test.ts`): ~300 combinations / **1692 checkpoints** across the asset store, filter/sort/search, money formatting × locale, locked-state write paths, mock lifecycle and platform cache invalidation.
- **Browser scenario suite** (`e2e/scenarios.spec.ts`): **39 scenarios** — 8 routes × 2 locales × 2 themes, 20 platform-search keywords, 4 filters × 4 sorts, 6 legal categories, heir-guide tabs/difficulty/collapse, DMS idempotence, report actions and per-page language/theme round-trips.

### Scenario-found defects

- **Backup import accepted invalid asset fields.** A hand-edited backup could write an unknown category, an unknown disposal method, or a negative/absurd value straight into the asset table, poisoning statistics and reports. The import now validates every row inside the transaction and leaves existing data untouched on rejection.
- **Total estimated value could go negative.** `assets.totalValue` used `|| 0`, so a negative `estimated_value` was added as-is; it now sums only finite, non-negative amounts, matching the backend.
- **The web-demo mock accepted invalid assets**, so the demo behaved differently from the desktop build; it now enforces the same category/disposal/value bounds.

### Changed

- Frontend unit tests grew from 296 to **341**, branch coverage 76.82% → **78.54%**; Rust tests 86 → **102**; E2E 37 → **76**. README badges synced.

## [0.2.0] - 2026-10-24

### Added

- **Asset search, sort and composition chart.** Full-text search over platform / account / notes with AND semantics, four sort orders, and an ECharts donut breaking assets into property / personality / mixed. `AssetsView`, `AssetCompositionChart`.
- **Platform search in the discovery wizard**, so a 152-entry policy database is navigable by typing.
- **Interactive web demo** (`VITE_DEMO=1` build) deployed alongside the website: the same UI on mock data, hash routing, with a persistent banner warning that demo data lives in `localStorage` and must not hold real credentials.
- **Backup format v2**: exports now include `heartbeat_config` and `platform_overrides`. v1 backups remain importable and no longer wipe those settings on import.
- **`PRAGMA user_version`** schema versioning (`SCHEMA_VERSION = 2`) so future migrations have a baseline instead of guessing from `PRAGMA table_info`.
- **Command-layer input validation**: 8 KB text caps, finite/non-negative value bounds, positive platform id, 1–365 day DMS interval, 500-char profile fields.
- **Heartbeat server hardening**: `/check` requires `Authorization: Bearer $HEARTBEAT_ADMIN_TOKEN` (503 when unset — fail closed), unregistered tokens are rejected with 403, alert state persists across restarts and resets on the next heartbeat, `/health` probe, 413 on oversized bodies, atomic state writes. Covered by `server/heartbeat-server.test.mjs`.
- **404 route** with a recovery path instead of a blank page; `document.title` now tracks the active route.
- **Bilingual website.** `docs/zh/index.html` is a full Chinese counterpart of the English site (problem, 8 features, 10 screenshots, security, comparison, getting started, roadmap, FAQ, footer), sharing `docs/site.css`. The language switch sits in the nav as an `EN | 中文` pill, stays visible when the section links collapse on narrow screens, marks the current language with `aria-current`, and preserves the section anchor when switching. Bidirectional `hreflang`, per-page `canonical`/`og:locale`, and `zh/` entries in the sitemap.
- **App screenshot above the fold.** The hero now shows the real 2400×1600 dashboard capture; the gallery grew from 7 to 10 images (adding asset detail, profile and dark mode) and every shot links to the full-size PNG.
- **`README.zh-CN.md`** — the Chinese documentation is now a standalone file instead of a section at the bottom of an English README. Both READMEs cross-link at the top and their tables of contents were re-checked (0 broken anchors).
- **Bilingual demo banner** in the static demo build, so the "do not enter real credentials" warning is readable to English visitors too.
- **Demo media pipeline** (`scripts/demo-media.mjs`): ten screenshots (nine screens + dark mode) and a 32-second walkthrough video. Each capture asserts the page header matches the expected title, so a wrong-route screenshot fails the run instead of shipping.

### Fixed

- **CI was red on the Rust job for two consecutive runs.** `cargo audit --deny vulnerabilities` is rejected by cargo-audit 0.22 (`--deny` only accepts warnings/unmaintained/unsound/yanked). The step now runs plain `cargo audit`, whose default is to fail on vulnerabilities.
- **Asset load failures were rendered as "no assets yet".** The store swallowed the error and the view treated an empty array as an empty result; failures now surface a `role="alert"` card with retry.
- **Asset detail edited the store object directly**, so "Cancel" kept the unsaved values and a failed save produced an unhandled rejection. Detail now works on a deep-copied draft, stays on the page on failure, and disables the buttons while saving.
- **Danger confirmations focused the destructive button.** `FeedbackLayer` now lands on Cancel for `danger: true`, traps Tab inside the dialog, and restores focus to the trigger on close.
- **Importing a backup left every store stale**, making a successful restore look like a failure. Import now refreshes security, dashboard, assets and platforms.
- **DMS settings had two sources of truth.** Saving the profile silently rewrote the interval set on the DMS page. Profile is now read-only for DMS and points at the single editor; interval options live in `constants/dms.ts`.
- **Dark-mode contrast**: `--gray-0` was never defined (lock card, help drawer and tour fell back to near-white), sidebar text reused theme-flipping tokens (1.72:1), solid buttons used "text" tokens as backgrounds (2.26–3.49:1), and the heir-guide prerequisite card was hard-coded light yellow. All now use dedicated `--surface` / `--solid-*` / `--on-solid` / `--sidebar-*` tokens (verified 5.88–9.22:1).
- **Report page rendered a blank shell with clickable-but-dead export buttons** while loading. Added a loading state, an error state with retry, and hid the action bar until the report exists.

### Changed

- **README rewritten as a showcase, not a manual.** Both `README.md` and `README.zh-CN.md` now open with the real dashboard screenshot, a one-line pitch, the live-demo/download links, a four-stat band (152 platforms · 45 commands · AES-256-GCM · 0 servers), an eight-shot guided tour, a capability grid, a verification table and a tech stack. Module layering and the encryption data flow stay in `docs/ARCHITECTURE.md`, verification evidence in `docs/VERIFICATION.md`, and the long-form comparison, security notes and FAQ live on the website (EN + 中文) instead of bloating the front page.
- README and `docs/VERIFICATION.md` re-derived from real runs: 86 Rust tests, 265 frontend unit tests, 37 E2E, coverage 79.4/79.0/73.2/80.8.
- `src/router.ts` and `src/api/invoke.ts` — previously 0% / partially covered despite being the single entry points for navigation and IPC — now have dedicated unit tests (`router.test.ts`, extended `invoke.test.ts`), covering the 404 fallback, per-route `document.title`, scroll restoration, mock routing, and the "no backend" error path.

## [0.1.0] - 2026-10-22

Initial public release.

### Added

- Digital asset discovery over a built-in policy database of 152 platforms across 16 categories, with a guided self-check questionnaire and de-duplication against already-registered assets.
- Asset classification (property / personality / mixed) and disposal planning (inherit, liquidate, memorialize, authorize viewing, delete, donate, no action).
- Inheritance report (print-ready HTML) and a ZIP toolkit containing the report, asset list, summary, and instructions.
- Dead man's switch with a three-stage confirmation chain, system-tray residency, and an optional zero-content cloud heartbeat with a reference self-hosted server.
- Heir guidance wizard with per-platform procedures, required documents, and a monetization/transfer path card.
- Legal template library across six categories.
- AES-256-GCM field-level encryption with PBKDF2-SHA256 (600k) key derivation, master-password lock with idle auto-lock, one-time recovery key, and encrypted backup export/import.
- Optional encrypted credential vault, excluded from report and toolkit exports.
- Light/dark theme following the system preference, reduced-motion support, print styles.
- Daily automatic backup (7-day retention, atomic writes).
- GitHub Actions CI: frontend typecheck/tests/coverage/build, Playwright E2E, Rust fmt/clippy/test, heartbeat server behaviour, Windows installer build, `npm audit` and `cargo audit`.
