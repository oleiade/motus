# Security Audit — motus

- **Date:** 2026-06-11
- **Scope:** Full repository at commit `548f0ce` (branch `main`). Crates `motus`, `motus-cli`, `motus-wasm`; build/release config (`.goreleaser.yaml`, `Makefile`, `Cross.toml`); CI/CD (`.github/workflows/*`); dependency policy (`deny.toml`, `Cargo.lock`).
- **Methodology:** OWASP Top 10:2025 manual review, plus dependency/source inspection of `rand` 0.10.1, `getrandom` 0.4.2, and `arboard` 3.6.1. Build and full test suite executed locally.
- **Auditor:** Claude (security-audit skill)

## Executive Summary

Motus is a local, offline CLI password generator with a very small attack surface: it takes validated command-line flags, generates a password from a cryptographically secure RNG (ChaCha12, via `rand` 0.10), and writes it to stdout and/or the clipboard. There is no network, no file parsing of untrusted input, no `unsafe` code, and no deserialization of attacker data, so the classic injection/RCE risk classes do not apply. The core randomness is sound — a user generating a password the normal way gets a genuinely strong, unpredictable result.

The findings are therefore about *handling* of the generated secret and the *supply chain*, not about broken crypto. The single most important item is the clipboard path (M1): motus copies passwords into the OS clipboard without opting out of clipboard history / cloud-clipboard sync, even though the exact `arboard` version in use supports that opt-out. Secondary items are the production-shipped `--seed` flag that yields fully predictable passwords while still being reported as "very strong" (M2), and an over-broad `secrets: inherit` on a mutable CI reference (M3). None are remotely exploitable; all are realistic ways the tool could leak or weaken a secret in practice.

## Findings by Severity

| # | Severity | OWASP | Title | Location |
|---|----------|-------|-------|----------|
| M1 | Medium | A02 | Password copied to clipboard without history/cloud exclusion or auto-clear | `crates/motus-cli/src/main.rs:141-150` |
| M2 | Medium | A06 | `--seed` ships in the release binary; produces predictable passwords reported as strong | `crates/motus-cli/src/main.rs:43-46,114-117` |
| M3 | Medium | A03 | CI passes all secrets (`secrets: inherit`) to an unpinned `@main` reusable workflow | `.github/workflows/ci.yml` |
| L1 | Low | A04 | Secure RNG on the WASM build is not guaranteed by build configuration | `crates/motus-wasm/Cargo.toml`, `Makefile` |
| L2 | Low | A03 | `motus` and `motus-wasm` do not build standalone (`rand` missing `alloc`) | `crates/motus/Cargo.toml:15` |
| L3 | Low | A02 | `debug = true` on the release profile ships symbols/source paths | `Cargo.toml:1-4` |
| L4 | Low | A10 | `expect()`/`unwrap()` panics; `unwrap_used` lint not enforced on the CLI crate | `crates/motus/src/lib.rs:77`, `crates/motus-cli/src/main.rs:175` |
| L5 | Low | A08 | Release binaries are not signed (checksums only, no provenance) | `.goreleaser.yaml` |
| I1 | Info | A04 | Weighted char mix and probabilistic number/symbol inclusion (by design) | `crates/motus/src/lib.rs:194-218` |
| I2 | Info | A03 | CI runs only `cargo deny check advisories`; `wildcards = "allow"` | `.github/workflows/security.yml`, `deny.toml` |

## Findings

### [Medium] [A02] Password copied to clipboard without history/cloud exclusion or auto-clear

**Location:** `crates/motus-cli/src/main.rs:141-150`

**What I spotted:** By default (the `clipboard` feature is on and `--no-clipboard` is not passed) the generated password is written to the system clipboard with a plain set:

```rust
if !opts.no_clipboard
    && let Err(e) = Clipboard::new().and_then(|mut clipboard| clipboard.set_text(&password))
```

`arboard` 3.6.1 — the exact version in `Cargo.lock` — exposes platform builder extensions that this code does not use:
- Windows (`SetExtWindows`): `exclude_from_monitoring()`, `exclude_from_cloud()`, `exclude_from_history()`
- macOS (`SetExtApple`): `exclude_from_history()`

**Why it matters:** A freshly generated password is the crown-jewel secret of this tool, and the clipboard is a shared, surprisingly persistent surface:
- On Windows 10/11 with clipboard history (Win+V) or "sync across devices" enabled, the password is retained in the local history ring and uploaded to the user's Microsoft cloud clipboard.
- On macOS with Universal Clipboard/Handoff, it is broadcast to the user's other Apple devices; any installed clipboard-manager app records it too.
- The password also stays on the clipboard indefinitely — there is no timeout clear (1Password-style tools wipe after ~90s) — so the next "paste anywhere" or a malicious app polling the clipboard can recover it long after generation.

Blast radius is a single user's generated secret, leaking from the local machine to cloud history or other local apps. It requires the attacker to already have some foothold (another local app, a synced/compromised cloud account), which is why this is Medium rather than High, but for a password tool it is the most consequential gap.

**Suggested fix:** Build the clipboard write through the platform `Set` extension and request exclusion from history/monitoring. Use `cfg` to pick the right trait per OS, e.g.:

```rust
#[cfg(target_os = "windows")]
use arboard::SetExtWindows;
#[cfg(target_os = "macos")]
use arboard::SetExtApple;

let mut clipboard = Clipboard::new()?;
let set = clipboard.set();
#[cfg(target_os = "windows")]
let set = set.exclude_from_monitoring(); // also keeps it out of cloud + history
#[cfg(target_os = "macos")]
let set = set.exclude_from_history();
set.text(&password)?;
```

Additionally, document the residual exposure in `--help`/README, and consider an opt-in auto-clear (spawn a short-lived process to clear the clipboard after N seconds, or use `arboard`'s Linux `wait`/`wait_until` to control ownership lifetime).

**Why this addresses it:** `exclude_from_monitoring`/`exclude_from_history` set the OS-level flags (`ExcludeClipboardContentFromMonitorProcessing`, `org.nspasteboard.ConcealedType`, cloud-exclusion) that tell Windows and macOS not to persist or sync the entry, removing the cloud/history leak path at the source rather than relying on the user to remember to clear it.

**How to verify:** On Windows, copy a password with the patched build, open clipboard history (Win+V) and confirm the entry is absent; check the cloud clipboard does not receive it. On macOS, confirm a clipboard manager (e.g. Maccy/Pasteboard viewer) does not retain it. Add an integration test asserting the code path compiles with the extension traits on each target.

**References:** OWASP A02 Security Misconfiguration; CWE-522 (Insufficiently Protected Credentials), CWE-316 (Cleartext Storage of Sensitive Information in Memory/clipboard).

---

### [Medium] [A06] `--seed` ships in the release binary and produces predictable passwords reported as "very strong"

**Location:** `crates/motus-cli/src/main.rs:43-46` (flag) and `:114-117` (use)

**What I spotted:** The top-level `--seed <u64>` flag is always compiled in (no `#[cfg(test)]`, no debug-only gate) and, when supplied, replaces the CSPRNG with a fully deterministic generator:

```rust
let mut rng: Box<dyn Rng> = match opts.seed {
    Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
    None => Box::new(thread_rng()),
};
```

The flag's help text says only "for deterministic password generation (for testing purposes)". Critically, `--analyze` runs `zxcvbn` on the resulting *string*, which has no idea the string came from a seed — so a seeded password is still reported as "very strong / 10^19 guesses".

**Why it matters:** A security-minded user can reasonably misread "deterministic … reproducible" as "I can regenerate this password later from a memorable seed and it's still safe." It is not: the entire password is a pure function of a 64-bit (often much smaller, human-chosen) seed. An attacker who suspects motus + a seed can brute force the seed space far faster than the apparent 2^64+ keyspace the password's character composition implies, and `--analyze` actively reinforces the false confidence. This is an insecure-design footgun: a sharp edge exposed in the production artifact with misleading feedback.

**Suggested fix:** Keep determinism available for tests but make it impossible to mistake for a secure mode in released binaries. Options, strongest first:
1. Gate the flag behind a build cfg so it is absent from release builds: `#[cfg(any(test, feature = "insecure-testing-seed"))]`, and do not enable that feature in `.goreleaser.yaml`/`Makefile` release builds. Tests already drive the binary via `assert_cmd`, so enable the feature in the test build only.
2. If it must stay, rename to `--insecure-seed`, print a `stderr` warning whenever it is used, and force-downgrade/short-circuit `--analyze` to print "INSECURE: deterministic seed — strength analysis not meaningful" instead of a normal score.

**Why this addresses it:** Removing the flag from release artifacts eliminates the footgun for end users entirely while preserving reproducible tests; the warning + analyze-suppression fallback removes the misleading "very strong" signal that turns a testing aid into a trap.

**How to verify:** Build a release binary (`cargo build --release -p cli`) and confirm `motus --seed 42 random` errors with "unknown argument". Confirm `cargo test` still passes (test build enables the feature). For option 2, confirm `--seed ... --analyze` no longer prints a positive strength score.

**References:** OWASP A06 Insecure Design; CWE-1188 (Insecure Default), CWE-330 (Use of Insufficiently Random Values), CWE-340 (Predictable from Observable State).

---

### [Medium] [A03] CI passes all repository secrets to an unpinned, mutable reusable workflow

**Location:** `.github/workflows/ci.yml`

**What I spotted:**

```yaml
jobs:
  ci:
    uses: oleiade/ci/.github/workflows/rust.yml@main
    secrets: inherit
```

The reusable workflow is referenced by the mutable `@main` ref (not a pinned commit SHA), and `secrets: inherit` forwards *every* secret defined in the `motus` repository to it. The fetched `oleiade/ci` workflow only runs fmt/clippy/build/test and needs no secrets at all.

**Why it matters:** This couples motus's secret material to the moment-to-moment state of another repository. If `oleiade/ci@main` is modified — by a compromised account, a malicious PR that lands, or a force-push — the altered workflow runs with motus's full secret set (which, per `release.yml`, includes a `DEPLOYMENT_ACCESS_TOKEN` used to push to the Homebrew tap). A leaked deployment token lets an attacker publish a malicious motus formula to users. Contrast this with `release.yml`, which correctly pins third-party actions to commit SHAs — the CI job is the weak link.

**Suggested fix:** Pin the reusable workflow to a SHA and pass only the secrets it actually needs (here, none):

```yaml
jobs:
  ci:
    uses: oleiade/ci/.github/workflows/rust.yml@<full-commit-sha>
    # drop `secrets: inherit`; pass nothing, or only named secrets the workflow declares
```

Adopt SHA-pinning for `actions/checkout` and `cargo-deny-action` too (the latter is already pinned; `checkout@v4` is a tag). Consider Dependabot/Renovate's GitHub-Actions manager so pinned SHAs still get updated.

**Why this addresses it:** SHA-pinning makes the third-party workflow immutable from motus's perspective, so a later compromise of `oleiade/ci`'s branch cannot retroactively change what runs in motus CI; dropping `secrets: inherit` removes the blast radius entirely, since a workflow that receives no secrets cannot leak them.

**How to verify:** Confirm CI still passes after removing `secrets: inherit` (the build/test/lint steps use no secrets). Confirm the `uses:` line references a 40-char SHA. Optionally run `zizmor`/`actionlint` on `.github/workflows/` in CI.

**References:** OWASP A03 Software Supply Chain Failures; CWE-829 (Inclusion of Functionality from Untrusted Control Sphere), CWE-1395.

---

### [Low] [A04] Secure RNG on the WASM build is not guaranteed by build configuration

**Location:** `crates/motus-wasm/Cargo.toml` (deps), `Makefile` (`wasm` target)

**What I spotted:** `motus-wasm` depends on `getrandom = "0.4.0"` and `rand = { version = "0.10.0", default-features = false }`, and is built with `wasm-pack build --target web` with no `RUSTFLAGS`/`.cargo/config.toml` selecting a `getrandom` backend and no `wasm_js` feature enabled. On `wasm32-unknown-unknown`, `getrandom` 0.4 has no default OS source; a CSPRNG-backed source (`crypto.getRandomValues`) is only used when the `wasm_js` backend is explicitly selected. Without it the crate either fails to link or resolves to the `unsupported` backend that errors at runtime when `rand::rng()` tries to seed.

**Why it matters:** For the WASM/web build, the entire security of generated passwords rides on the seed source being `crypto.getRandomValues`. The current configuration does not pin that, so the build is fragile: a refactor or a different build invocation could silently produce a module whose RNG fails (best case: visible panic/breakage) or — if a future "custom"/stub backend were wired in to "make it build" — produce predictable passwords (worst case, silent). The CLI is unaffected (it builds with `alloc`/`thread_rng` and a real OS source); this is specific to the web target.

**Suggested fix:** Make the secure WASM backend explicit and non-optional:

```toml
# crates/motus-wasm/Cargo.toml
getrandom = { version = "0.4", features = ["wasm_js"] }
rand = { version = "0.10", default-features = false, features = ["alloc"] }
```

and set the backend cfg for the wasm build (e.g. a `crates/motus-wasm/.cargo/config.toml`):

```toml
[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```

Add the wasm build to CI so this can't regress.

**Why this addresses it:** Pinning `wasm_js` forces `getrandom` to use the browser's `crypto.getRandomValues` (a CSPRNG) and makes the build fail loudly if that source is unavailable, removing both the silent-weak-RNG and the accidental-breakage paths.

**How to verify:** `rustup target add wasm32-unknown-unknown`, run the `make wasm` build to completion, load the module, and confirm `random_password` returns varied output across reloads; add a CI job that builds the wasm target.

**References:** OWASP A04 Cryptographic Failures; CWE-338 (Use of Cryptographically Weak PRNG), CWE-330.

---

### [Low] [A03] `motus` and `motus-wasm` do not build standalone (`rand` missing `alloc`)

**Location:** `crates/motus/Cargo.toml:15`

**What I spotted:** The library declares `rand = { version = "0.10.0", default-features = false }` with no `alloc` feature, yet `lib.rs` uses `rand::distr::weighted::WeightedIndex` and `IndexedRandom::sample`, both gated behind `alloc`. Building the library or the wasm crate in isolation fails:

```
error[E0432]: unresolved import `rand::distr::weighted`
error[E0599]: the method `sample` exists ... but its trait bounds were not satisfied
```

It only compiles inside this workspace because `motus-cli` enables `rand`'s `alloc` feature and Cargo feature-unification turns it on for everyone. The `motus` crate is published (it has `repository`/`homepage` and is not `publish = false`).

**Why it matters:** This is an integrity/reliability issue rather than a direct vulnerability: a third party who runs `cargo add motus` and builds gets a compile error, and the WASM target's security (L1) can't even be exercised because the crate doesn't build on its own. A library whose advertised build is broken erodes the "users can rely on this" goal.

**Suggested fix:** Declare the features each crate actually needs, independent of unification: add `features = ["alloc"]` to `rand` in `crates/motus/Cargo.toml` (and `["alloc"]` for the wasm crate). Add a CI job that builds each crate alone (`cargo build -p motus`, `cargo build -p motus-wasm --target wasm32-unknown-unknown`) so unification can't mask the gap again.

**Why this addresses it:** Stating the real feature requirements makes each crate self-consistent so it builds regardless of what siblings enable; the per-crate CI build prevents silent regression.

**How to verify:** `cargo build -p motus` and `cargo build -p motus-wasm` (with the wasm target) both succeed from a clean checkout.

**References:** OWASP A03 Software Supply Chain Failures; CWE-1104 (Use of Unmaintained/Misconfigured Components — here, misconfigured feature flags).

---

### [Low] [A02] `debug = true` on the release profile ships symbols and source paths

**Location:** `Cargo.toml:1-4`

**What I spotted:**

```toml
[profile.release]
debug = true
incremental = true
lto = "off"
```

The distributed binaries (built `--release` by goreleaser/Makefile) carry full debug info.

**Why it matters:** Not a vulnerability on its own, but for a security tool it is unnecessary exposure: debug symbols embed absolute build-machine source paths and internal symbol names, inflate binary size, and make the binary easier to reverse. `lto = "off"` and `incremental = true` are also build-speed choices that don't belong in a release profile.

**Suggested fix:** Use release-appropriate settings: `debug = false` (or `strip = "symbols"`), `incremental = false`, and enable `lto = "thin"` or `"fat"`. Keep a separate profile if local profiling needs symbols.

**Why this addresses it:** Stripping symbols removes the leaked paths/symbol names and shrinks the artifact; it does not affect runtime security but tightens the distributed footprint.

**How to verify:** `cargo build --release` then `nm`/`dsymutil`/`strings` on the binary shows no source paths; binary size drops noticeably.

**References:** OWASP A02 Security Misconfiguration; CWE-200 (Exposure of Sensitive Information).

---

### [Low] [A10] Panicking `expect()`/`unwrap()` and missing `unwrap_used` lint on the CLI crate

**Location:** `crates/motus/src/lib.rs:77` (`String::from_utf8(bytes).expect(...)`), `crates/motus-cli/src/main.rs:175` (`serde_json::to_string(&output).unwrap()`); also `:112-113`, `:213-217`.

**What I spotted:** The scramble path shuffles a word's raw bytes and then asserts the result is UTF-8:

```rust
let mut bytes = word.clone().into_bytes();
bytes.shuffle(rng);
word = String::from_utf8(bytes).expect("random words should be valid UTF-8");
```

This is safe *today* only because the embedded wordlist is pure ASCII (each byte is independently valid UTF-8, so any permutation is valid). If the wordlist ever gains a multi-byte character, a shuffle can split it and this panics. The library crate enforces `unwrap_used = "deny"`, but `motus-cli` declares no `[lints]`, so the `serde_json::...unwrap()` and other `expect()`s in the binary are unguarded.

**Why it matters:** Worst case is a clean abort (caught by `human-panic`, which writes a panic report to a temp file — note the report contains static panic messages and a backtrace, not the password, so no secret leaks). So this is robustness/DoS-resistance, not a data exposure. Still, a password tool aborting mid-generation is a poor failure mode, and the asymmetry in lint enforcement lets new unguarded panics slip into the binary.

**Suggested fix:** Make scramble independent of UTF-8 validity by shuffling `char`s instead of bytes:

```rust
let mut chars: Vec<char> = word.chars().collect();
chars.shuffle(rng);
word = chars.into_iter().collect();
```

Apply the same `[lints]` block the library uses to `crates/motus-cli/Cargo.toml` (or set it at the workspace level) so `unwrap_used`/`expect_used` are denied everywhere, and replace the remaining `unwrap()`/`expect()` with explicit error handling.

**Why this addresses it:** Shuffling `char`s can never produce invalid UTF-8, removing the latent panic regardless of wordlist contents; enforcing the lint workspace-wide stops new panics from entering the shipped binary.

**How to verify:** Add a unit test that scrambles a word containing a multi-byte char (e.g. `"café"`) and asserts no panic; run `cargo clippy --workspace --all-targets -- -D warnings` and confirm it flags any reintroduced `unwrap`.

**References:** OWASP A10 Mishandling of Exceptional Conditions; CWE-248 (Uncaught Exception), CWE-754 (Improper Check for Unusual Conditions).

---

### [Low] [A08] Release binaries are not signed (checksums only, no provenance)

**Location:** `.goreleaser.yaml`

**What I spotted:** The goreleaser config produces archives, nfpm packages, and a Homebrew formula, but has no `signs:` (cosign/GPG) section and no SLSA provenance/attestation. goreleaser emits a `checksums.txt` by default, and the Debian repo (per README) is GPG-signed, but direct release-page downloads and the macOS/zip/rpm/apk/archlinux artifacts have no signature a user can verify against the author's key.

**Why it matters:** A user who downloads a binary from the releases page can detect accidental corruption (checksum) but cannot verify authenticity — a checksum file hosted next to the artifact is replaced just as easily as the artifact if the release is tampered with. For a tool whose job is to produce secrets, supply-chain authenticity matters.

**Suggested fix:** Add artifact signing to goreleaser (cosign keyless is low-friction):

```yaml
signs:
  - cmd: cosign
    args: ["sign-blob", "--yes", "--output-signature=${signature}", "${artifact}"]
    artifacts: all
```

and enable GitHub's build provenance/attestation (`actions/attest-build-provenance`) in `release.yml`. Document the verification command in the README.

**Why this addresses it:** A cosign signature (or SLSA provenance) binds each artifact to the release pipeline's identity, so a swapped binary fails verification even if its checksum file is swapped too.

**How to verify:** After a test release, run `cosign verify-blob --signature ... <artifact>` and confirm it validates; confirm `gh attestation verify` passes.

**References:** OWASP A08 Software or Data Integrity Failures; CWE-345 (Insufficient Verification of Data Authenticity).

---

### [Informational] [A04] Weighted character mix and probabilistic number/symbol inclusion

**Location:** `crates/motus/src/lib.rs:194-218`

**What I spotted:** `random_password` selects each character set by weight (70% letters / 20% numbers / 10% symbols) and selects uniformly within the chosen set. Two consequences worth documenting, both *by design* and not vulnerabilities: (1) entropy per character is slightly below a flat uniform draw over the full alphabet; (2) `--numbers`/`--symbols` make those classes *possible*, not *guaranteed* — a short password can legitimately contain zero symbols, which may fail sites that mandate one. The minimum lengths (random ≥8, words ≥3, PIN ≥3) keep total entropy high regardless, and `--analyze` reports the real strength of the produced string.

**Why it matters:** No security impact at the default 20-character length; the weighting mirrors 1Password's UX choice. Flagged only so the behavior is a documented decision, not an accident. If "at least one of each requested class" is ever desired, generate one guaranteed character per requested set and fill the remainder by weight, then shuffle.

**References:** OWASP A04; CWE-331 (Insufficient Entropy) — noted as not-applicable at default settings.

---

### [Informational] [A03] CI security job scope and dependency policy

**Location:** `.github/workflows/security.yml`, `deny.toml`

**What I spotted:** `security.yml` runs `cargo deny check advisories` (daily + on push/PR) — good coverage for known CVEs. It does not also run `check bans sources licenses`, and `deny.toml` sets `wildcards = "allow"` and `multiple-versions = "warn"`. There is no lockfile-pinned `cargo audit`/`osv-scanner` beyond cargo-deny, and `Cargo.lock` is committed (good).

**Why it matters:** Posture is already above average for a project this size. Tightening is hardening, not fixing a hole: run `cargo deny check all` (or `bans sources licenses` alongside advisories) to catch untrusted sources and dependency confusion, and set `wildcards = "deny"` to forbid unpinned version ranges.

**References:** OWASP A03; CWE-1104.

## OWASP Top 10:2025 Coverage

- **A01 Broken Access Control** — No issues. No multi-user model, no auth, no server; the tool is a single-user local process with no protected resources. The clipboard exposure is filed under A02.
- **A02 Security Misconfiguration** — M1 (clipboard history/cloud exclusion), L3 (release debug symbols). Checked: no debug endpoints, no default creds, no embedded secrets (grep + git-history sweep clean).
- **A03 Software Supply Chain Failures** — M3 (`secrets: inherit` to `@main`), L2 (broken standalone build), I2 (deny scope). Checked: `Cargo.lock` committed and consistent; `release.yml` SHA-pins third-party actions; `cargo deny advisories` runs daily.
- **A04 Cryptographic Failures** — L1 (WASM RNG config), I1 (weighting). Positive: normal CLI generation uses `rand::rng()`/`StdRng` = ChaCha12 CSPRNG (verified in `rand` 0.10.1 source); no MD5/SHA1, no home-rolled crypto, no data-at-rest.
- **A05 Injection** — No issues. No SQL/template/LDAP/XPath, no shell-out, no `eval`. Generated passwords are written to stdout/clipboard, never interpolated into a command or query. CLI args are integers validated by range parsers; the only string input (`--separator`) is a closed `ValueEnum`.
- **A06 Insecure Design** — M2 (`--seed` footgun + misleading `--analyze`). Input validation (`validate_word_count`/`_character_count`/`_pin_length`) enforces sane minimums.
- **A07 Authentication Failures** — Not applicable. The tool has no accounts, sessions, tokens, or login flow.
- **A08 Software or Data Integrity Failures** — L5 (no artifact signing). No insecure deserialization (only `serde_json::to_string` *output*, never untrusted input); no auto-update mechanism.
- **A09 Security Logging and Alerting Failures** — No issues, and intentionally so for a local tool. Note (positive): the password is *not* written to any log file; `human-panic` reports capture static messages/backtrace, not the secret. The clipboard warning on stderr does not include the password.
- **A10 Mishandling of Exceptional Conditions** — L4 (panics + lint gap). No fail-open security decision exists; panics abort cleanly via `human-panic`.

## Suggested Next Steps

**This week (highest value, low effort):**
1. M1 — Route the clipboard write through `arboard`'s `exclude_from_monitoring`/`exclude_from_history` extensions; document the residual risk; consider opt-in auto-clear.
2. M3 — Drop `secrets: inherit` from `ci.yml` and pin the reusable workflow to a SHA.
3. M2 — Gate `--seed` out of release builds (feature flag), or warn + suppress `--analyze` when it is used.

**This sprint:**
4. L1 + L2 — Pin `getrandom`'s `wasm_js` backend and add `alloc` to `rand` for `motus`/`motus-wasm`; add CI jobs that build each crate standalone and build the wasm target.
5. L4 — Make scramble shuffle `char`s; apply the `[lints]` (deny `unwrap_used`) to the CLI crate / workspace.
6. L3 — Set a proper release profile (`debug = false`/`strip`, `lto = "thin"`, `incremental = false`).

**This quarter (hardening):**
7. L5 — Add cosign signing + build provenance to the release pipeline; document verification.
8. I2 — Expand the security job to `cargo deny check all`; set `wildcards = "deny"`; add `actionlint`/`zizmor` for workflow linting.
9. Add `cargo-deny`/`cargo-audit` and a wasm build to local `make ci-check` so contributors catch these before CI.

## Tooling Run

- `cargo build` / `cargo check --workspace --all-features` — succeeds.
- `cargo test --locked --all-features --release` — all tests pass.
- `cargo check -p motus-wasm` (standalone) — **fails** (`rand` missing `alloc`); evidence for L2.
- Secret sweep (`git grep` for key/token/password patterns + `git log --diff-filter=A` over history) — no secrets found.
- Source inspection of `rand` 0.10.1, `getrandom` 0.4.2, `arboard` 3.6.1 in the local cargo registry — confirmed ChaCha12 CSPRNG, getrandom backend requirements, and the unused clipboard-exclusion APIs.
- `cargo-deny`, `gitleaks`, `semgrep`, `osv-scanner` — not installed in this environment; recommended for CI adoption (cargo-deny already runs in `security.yml`).

## Caveats

- No live/dynamic testing was performed (no clipboard-history extraction on a real Windows/macOS host; no running web build). The clipboard and WASM findings are based on the source of the exact dependency versions in `Cargo.lock` and the build configuration, not on observed exploitation.
- The `oleiade/ci` reusable workflow was read at its current `@main`; because it is a mutable ref, its contents can change after this audit — that mutability is itself finding M3.
- The EFF wordlist content was not audited word-by-word for quality; only its size, encoding (ASCII), and the `len() >= 4` filter (which trims it from 7776 to 7694 entries, a negligible entropy effect) were checked.
- Third-party crate internals were reviewed only where relevant to a finding; a full dependency-tree audit is delegated to `cargo deny`/`cargo audit` in CI.
