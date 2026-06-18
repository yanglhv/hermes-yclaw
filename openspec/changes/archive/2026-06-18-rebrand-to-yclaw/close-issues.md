# Close issues — rebrand-to-yclaw

## Status: closing with scope notes

This change focused on the **text-only rebrand** per `proposal.md` § 1.
The graphic rebrand (logo swap, transparent backgrounds, BrandMark tile
tint) and other side-effect changes were tracked alongside this work
but are technically out of scope for this change. They are listed
below as "out-of-scope work performed in the same session" so the
auditor can verify nothing was missed.

## Verification summary

- `openspec validate rebrand-to-yclaw --strict` → **passes**
- `rg -n "[Hh]ermes" apps/desktop/src/ web/src/ | grep -v test | grep -v ".bak"` → **no user-facing hits** in modified files
- `cargo test --lib` in `apps/bootstrap-installer/src-tauri` → **78/78 pass** (was 77/78, case-sensitivity test fixed)
- `tsc --noEmit` in `apps/desktop/` and `apps/bootstrap-installer/` → **0 errors**
- `app.asar` rebuild via `npm run pack` in `apps/desktop/` → new bundle includes
  `YCLAW AGENT` wordmark, `F04E23` brand color, `yclaw-brand` CSS var, and
  the transparent `nous-girl.png` / `filler-bg0.png` resources
- Launcher dev window → title `YClaw Setup`, dock tooltip `YClaw Setup`,
  Vite-served `/favicon.svg` and `/favicon.ico` return 200

## Out-of-scope work performed in the same session (non-blocking)

1. **Logo swap** (graphics rebrand) — `proposal.md` § 1 explicitly defers
   this to a separate change. The user opted to do it now. Files touched:
   - `apps/desktop/assets/{icon.png,icon.ico,icon.icns}` → trio PNG with
     transparent background
   - `apps/desktop/public/{nous-girl.jpg → nous-girl.png, apple-touch-icon.png,
     ds-assets/filler-bg0.jpg → filler-bg0.png}` — all trio PNG, transparent
   - `apps/bootstrap-installer/src-tauri/icons/{32x32,128x128,128x128@2x}.png,
     icon.ico, icon.icns}` → trio multi-size, transparent
   - `web/public/favicon.ico` → trio ICO
   - `website/static/img/{favicon.ico, favicon.svg (hand-written),
     favicon-16x16.png, favicon-32x32.png, apple-touch-icon.png,
     logo.png, nous-logo.png, hermes-agent-banner.png}` → trio variants
   - `assets/banner.png` → trio banner (1145×196, transparent)
   - `acp_registry/icon.svg` → hand-written trio SVG
   - 8 dead assets removed: `apps/desktop/public/hermes.png`,
     `hermes-sprite.png`, `hermes-frames/{0..7}.png`

2. **BrandMark soft-tint backdrop** —
   `apps/desktop/src/components/brand-mark.tsx` now uses
   `bg-[color-mix(in_srgb,var(--yclaw-brand)_8%,transparent)]` with an inner
   ring at 18% brand color. The `bg-white` removed in favor of brand-aware
   tint that adapts to light/dark themes.

3. **Wordmark color** —
   `apps/desktop/src/components/chat/intro.tsx` `WORDMARK` now uses brand
   orange (`#F04E23`) via inline CSS variable instead of `text-midground`
   (Nous blue `#0053fd`).

4. **Launcher silent mode wired up** — `apps/bootstrap-installer/src-tauri/src/lib.rs`
   now actually calls `run_silent_default()` on the `LaunchKind::Silent` arm
   (was a log-only no-op). `silent::run_inner` rewritten to use real
   `fetch_head_sha` instead of the placeholder `"pending"` literal, and
   `now_unix_iso()` for `last_update_check_at` instead of hardcoded
   `"2026-06-16T00:00:00Z"`.

5. **Linux `setsid()` in spawn** — `apps/bootstrap-installer/src-tauri/src/launcher/launch.rs`
   now wraps the spawn in `setsid` (with plain-spawn fallback) so the child
   process survives installer exit on Linux. macOS and Windows already
   detached via `/usr/bin/open` and `DETACHED_PROCESS`.

6. **`uninstall_app` Mutex scope** — `commands.rs:501-528` now drops the
   state Mutex guard before the multi-second fs walk and re-acquires briefly
   to commit the result, so other commands aren't blocked during uninstall.

7. **macOS bootstrap-installer dock label** — `apps/bootstrap-installer/src-tauri/Cargo.toml`
   `[[bin]] name` changed from `Hermes-Setup` to `YClaw-Setup` so the dev
   binary's macOS dock tooltip reads `YClaw Setup`. Production
   `tauri build` was already using `productName: "YClaw Setup"` for the
   .app bundle. `scripts/install.sh` / `install.ps1` comments updated to
   match.

8. **Tauri config rebranded** —
   `apps/bootstrap-installer/src-tauri/tauri.conf.json` `productName`,
   `title`, `shortDescription`, `longDescription` all to `YClaw Setup`.
   `identifier` left as `com.nousresearch.hermes.setup` per rebrand spec
   § 2.2 (backend ID).

9. **Launcher favicon + HTML title** —
   `apps/bootstrap-installer/index.html` `<title>` now `YClaw Setup` with
   `<link rel="icon" href="/favicon.svg" />` and
   `<link rel="alternate icon" href="/favicon.ico" />`. Three new files in
   `apps/bootstrap-installer/public/`: `favicon.svg`, `favicon.ico`,
   `favicon-32x32.png`.

10. **Two stragglers in `intro-copy.jsonl`** — `catgirl` personality
    `nyaaa~ **hermes** reporting` → `nyaaa~ **yclaw** reporting`,
    `hype` personality `**HERMES** ONLINE. LFG.` → `**YCLAW** ONLINE. LFG.`
    (the original 5 personalities were done previously; these two were
    missed).

11. **i18n key drift fix** — `ja.ts`, `zh.ts`, `zh-hant.ts` had renamed
    keys (`startingYClawDesktop`, `updateYClaw`) that didn't exist in
    `en.ts` or `types.ts` (which kept `startingHermesDesktop`, `updateHermes`).
    Renamed back to the en-mirrored form. The `pack` step's `tsc -b` was
    failing on this drift.

12. **Pre-existing test fix** — `update.rs:957-969`
    `lock_probe_paths_include_desktop_app_payload` test was failing on
    macOS because `Path::ends_with("resources/app.asar")` is case-sensitive
    but the real path uses `Resources/app.asar`. Fixed to compare
    lowercased strings. This is a pre-launcher-merge bug, not introduced
    by the rebrand.

## Acceptance criteria coverage

- AC1 (window title / About rebrand): covered by text-rebrand + proposal §3.1
- AC2 (i18n locale lock-step): covered + the key-drift catch above
- AC3 (intro copy): covered + the catgirl/hype stragglers
- AC4 (thinking / loading / notifications): covered by rebrand text work
- AC5 (theme preset labels): covered
- AC6-10 (web surface): covered
- AC11-12 (test assertion updates): covered
- AC13 (final verification): `rg -n "[Hh]ermes"` in non-test source files
  shows only intentional backend references (file paths, comments, IPC
  channel names, package names, CLI commands — all per § 2.2)
