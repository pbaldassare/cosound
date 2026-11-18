# COSOUND internal knowledge base (not public)

Verified technical knowledge the COSOUND agents will use to operate DAWs, synths and plugins.
Lives in the repo only: never served by the website.

## Structure
One folder per platform (`logic-pro/`, `ableton-live/`, `fl-studio/`, `pro-tools/`, `cubase/`,
`synthesizers/`, `plugins-effects/`, `integration-sdks/`). Each has a `sources.md` registry.

## Rules for every entry
- **Official sources only** (manuals, API/SDK docs, vendor guides). Record URL and retrieval date.
- **Versioned**: every fact is tied to the software version it was verified on.
- **Access level** must be one of:
  - `api`: documented programmatic interface (scripting API, SDK)
  - `limited`: partial or unofficial route (control surface protocol, UI automation, file import)
  - `none`: not accessible from outside the application
- Keep "what the DAW can do" separate from "what COSOUND can drive from outside".

In a later phase these notes become a structured, indexed knowledge base (one record per fact, per version).
