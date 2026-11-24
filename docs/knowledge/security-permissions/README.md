# Security and Permissions

Section 14. Which OS permissions COSOUND needs, when to ask, how to sign/distribute, how to keep
secrets, and what may leave the machine. Sources: `sources.md`. Verified 2026-11-24.
**Proposal** = design suggestion. **(to verify)** = not confirmed.

## 1. Permission matrix

| Feature | macOS | Windows | When requested (Proposal) |
|---|---|---|---|
| Voice commands / voice-to-music | Microphone (TCC): `NSMicrophoneUsageDescription` in Info.plist + `audio-input` entitlement under Hardened Runtime; `AVCaptureDevice` request [S1][S2][S3][S4] | Settings › Privacy & security › Microphone: global switch + "Let desktop apps access your microphone" (desktop apps can't be toggled individually) [S5] | First time the user presses the mic button |
| Library folders on internal disk (Documents/Desktop/Downloads) | Files & Folders consent (TCC); user-picked folder via Open panel; sandbox: `files.user-selected.read-write` + security-scoped bookmark [S8][S9][S12] | NTFS ACLs only; no consent prompt for the user's own files | When the user adds a library root |
| Library on USB/external drive | "Removable Volumes" prompt (`NSRemovableVolumesUsageDescription`) unless file picked/dropped [S10] | none | When adding a root on that drive |
| Library on NAS/SMB share | "Network Volumes" prompt (`NSNetworkVolumesUsageDescription`) [S11]; Local Network privacy if the app itself talks to LAN hosts [S19] | none (Windows Firewall may prompt for inbound listeners) **(to verify)** | When adding a network root |
| Whole-disk scan | Full Disk Access – user must add the app in System Settings manually [S12] | n/a | **Never requested** (design rule) |
| DAW UI automation | Accessibility: `AXIsProcessTrusted` / `AXIsProcessTrustedWithOptions(prompt)`; user enables in System Settings [S12][S13][S14] | UI Automation needs no prompt; to drive elevated DAWs, `UIAccess` manifest + signed + secure location; normal apps run at medium integrity [S26][S27][S28] | When the user enables "control my DAW" for a DAW without an official API |
| Scripting DAWs/apps via AppleScript | Automation (Apple Events): `NSAppleEventsUsageDescription` + `automation.apple-events` entitlement; prompt per target app [S12][S15][S16] | n/a (COM/other per DAW) | First command sent to that app |
| OSC/network to DAW or devices on LAN | Local Network privacy (macOS 15+), `NSLocalNetworkUsageDescription` [S19][S20] | Firewall prompt for listening sockets **(to verify)** | When enabling OSC / network devices |
| Store login tokens / API keys | Keychain (no prompt for own items) [S21] | Credential Manager / DPAPI (no prompt) [S22][S23] | At sign-in |
| Notifications, launch at login | Standard system prompts **(not researched)** | – | Opt-in in settings |

Rules (Proposal): ask **just in time** with an in-app explanation screen first (IT/EN), never at
first launch in bulk; every feature works in a degraded mode if a permission is denied; a
Permissions page shows status + "open System Settings" links.

## 2. Microphone
- macOS: users must explicitly grant mic access per app; system shows a prompt with the app's
  usage string; the choice is remembered and changeable in System Settings [S2]. Check
  `authorizationStatus(for: .audio)`; if `.notDetermined`, request. Without permission recordings
  contain **silence** – the API does not fail loudly, so check status before recording [S1].
  `NSMicrophoneUsageDescription` is required [S3]; a hardened-runtime app also needs the
  `com.apple.security.device.audio-input` entitlement [S4].
- Windows: microphone access for desktop (Win32) apps is a single global toggle "Let desktop apps
  access your microphone", not per app [S5]; Electron notes the same global setting applies to all
  win32 apps [S6]. Detect silence/denied and point the user to Settings.

## 3. File-system access (macOS)
- App Sandbox limits access to resources requested via entitlements; required for the Mac App
  Store [S7]. A Developer ID build distributed outside the store need not be sandboxed.
- **TCC still applies without the sandbox:** since macOS 10.15 the system requires consent for
  Documents, Downloads, Desktop, iCloud Drive and network volumes; full-disk access requires the
  user to add the app manually; Accessibility and Automation are separate privacy items [S12].
- Folders/files chosen in an Open panel, dropped on the app or opened in Finder are implicitly
  granted, including on removable and network volumes [S10][S11].
- In a sandboxed app, persist access with security-scoped bookmarks (`.withSecurityScope`), which
  on resolution give a read/write security-scoped URL [S9]; the app-scope bookmark entitlement
  page could not be opened **(to verify)**.
- FSEvents only delivers events for directories the user can reach [S37].
- **Proposal:** ship non-sandboxed Developer ID first (simpler for DAW automation), but still use
  the picker + bookmarks so a future sandboxed/App Store build is possible.

## 4. Accessibility, Automation, UI Automation
- macOS: `AXIsProcessTrusted()` returns whether the process is a trusted accessibility client;
  `AXIsProcessTrustedWithOptions` with `kAXTrustedCheckOptionPrompt` asks the system to inform the
  user asynchronously (the return value is not affected) [S13][S14]. The user must toggle the app in
  Privacy & Security › Accessibility [S12]. Electron exposes `isTrustedAccessibilityClient(prompt)`
  [S6]. Re-check after re-signing/updates (TCC grants are tied to the code signature
  **(to verify)**).
- Apple Events: `NSAppleEventsUsageDescription` required for apps that send Apple events; Apple
  warns this can expose another app's data indirectly [S15]. The entitlement is needed under the
  Hardened Runtime unless the target is the app itself or signed by the same team [S16].
- Windows: UI Automation is the accessibility framework for programmatic access to other apps'
  UIs [S26]. Without `UIAccess` an app runs at medium integrity and cannot access elevated
  processes' UI; `UIAccess` must be declared in the manifest and the app launched appropriately
  [S27]; Windows marks processes with integrity levels [S28]. `UIAccess` apps must also be signed and
  installed in a secure location such as Program Files **(to verify)**. Proposal: don't use
  UIAccess unless a target DAW runs elevated.

## 5. Code signing and notarization
- macOS: sign with **Developer ID**, enable **Hardened Runtime** (protects against code injection,
  dylib hijacking; disallows e.g. JIT unless an exception entitlement is added; only add needed
  entitlements) [S17], then **notarize** with `notarytool` (altool/Xcode ≤ 13 no longer accepted
  since 1 Nov 2023). Notarization is an automated malware/signing scan, not App Review; staple the
  ticket so Gatekeeper can verify offline; apps, disk images and flat installer packages can be
  notarized [S18]. Tauri documents signing + notarization in its bundler [S32].
- Windows: **Authenticode** signing (SignTool) for .exe/.msi/.msix/.cab etc. [S29]. SmartScreen
  checks reputation of the file and signing certificate; with no reputation the user sees a
  warning [S30] – a new certificate still starts without reputation **(to verify how long it takes)**.
  Microsoft **Artifact Signing** (Azure, managed certificates, zero-touch lifecycle) is an option
  instead of buying/storing an EV/OV cert [S31] – eligibility and price **(to verify)**.

## 6. Secrets storage
| Store | Platform | Notes |
|---|---|---|
| Keychain Services | macOS | Encrypted store for small secrets (passwords, keys, tokens) [S21] |
| Credential Manager (`CredWrite`) | Windows | Blob max `CRED_MAX_CREDENTIAL_BLOB_SIZE` = 5×512 bytes [S23] – fine for refresh tokens, too small for big JSON |
| DPAPI `CryptProtectData` | Windows | Typically only the same user (logon credentials) on the same computer can decrypt [S22] |
| `keyring` crate (Rust/Tauri) | both | Wraps platform stores; MIT or Apache-2.0 [S24] |
| Electron `safeStorage` | both | Key kept in Keychain (macOS) / generated via DPAPI (Windows); protects from other users, not from other apps of the same user on Windows [S25] |

Store: Supabase refresh token, any user-provided API keys. Never in plain config files or logs.
The Supabase **service-role key never ships** in the desktop app; only the publishable/anon key.

## 7. What goes to the cloud (Proposal, design rule)
| Data | Default | Opt-in |
|---|---|---|
| Account, subscription, device registration | Supabase (already in schema) | – |
| Prompts to the LLM | Text + small metadata (BPM, key, plug-in names, file *names* of a few results) | – |
| Library file lists / index | **Local only** | Possibly sync of tags, never paths by default |
| Raw audio, stems, projects | **Never** | Explicit per-action consent (e.g. "analyze this stem in the cloud") |
| Microphone audio | Processed locally (STT local or explicit cloud STT opt-in) | Cloud STT opt-in |
| Music memory | Local | Sync to Supabase with RLS (see `../music-memory/`) |
| Telemetry/crash logs | Off or anonymous, no paths/file names | Opt-in |

Data minimization: GDPR requires personal data be limited to what is necessary and data
protection by design and by default (Art. 5(1)(c), Art. 25) [S36] **(to verify against EUR-Lex)**.

## 8. Supabase RLS per user
- Enable RLS on every table in an exposed schema; without RLS, any role with a grant can read and
  write it; adding policies doesn't remove existing grants – revoke unneeded anon/authenticated
  grants [S34].
- Use `auth.uid()` in policies, wrapped as `(select auth.uid())` as in Supabase examples; for
  insert/delete check only the user who may create/remove the row [S34].
- New tables for the desktop app (memory, library tag sync, device sessions) follow the existing
  "read own / write own" pattern in `supabase/migrations/`.

## 9. GDPR basics
- Rights: informed, access, rectification, erasure, restriction, portability, objection, not
  being subject to solely automated decisions; answer requests within one month (extendable with
  notice) [S35]. Provide self-service export and delete in the app and website.
- Required before public sign-ups: privacy policy, terms (already listed in `docs/SETUP.md`).
- LLM providers are processors: DPA and data-retention settings must be checked per provider
  **(to verify when the provider is chosen)**.

## 10. Stack implications
- **Tauri 2** (Apache-2.0/MIT, Rust backend): capability-based permissions restrict which commands
  each window can call [S33]; native Rust crates (`notify`, `keyring`, Symphonia) fit directly.
- **Electron** (MIT): ready wrappers for mic prompt, accessibility check and safeStorage [S6][S25];
  larger footprint.
- Either way, TCC/Accessibility/Automation are tied to the signed app bundle; sign every helper
  binary (e.g. ffprobe sidecar) too **(to verify for TCC attribution of helpers)**.
