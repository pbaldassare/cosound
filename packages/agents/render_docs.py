#!/usr/bin/env python3
"""Regenerate docs/agents/AGENTS.md and docs/agents/REGISTRIES.md from the manifests in this folder.

The JSON manifests are the source of truth (compiled into the app and checked by the Rust tests);
run this after changing one:  python3 packages/agents/render_docs.py
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
DOCS = os.path.join(HERE, "..", "..", "docs", "agents")
ORDER = ["music_producer", "rhythm_engineer", "sound_designer", "music_theory", "voice_to_music", "audio_analysis",
         "reference_track", "smart_library", "plugin_intelligence", "mixing_engineer", "mastering_engineer",
         "daw_controller", "adaptive_workflow"]
LETTERS = "ABCDEFGHIJKLM"


def load(name):
    with open(os.path.join(HERE, name), encoding="utf-8") as f:
        return json.load(f)


AGENTS = [load(a + ".json") for a in ORDER]
TOOLS = load("tools.json")
PERMS = load("permissions.json")


def available(a):
    return sum(o["status"] == "available" for o in a["operations"])


def anchor(letter, a):
    return f"{letter.lower()}-" + a["name"].lower().replace(" ", "-")


def type_of(p):
    t = p.get("type", "any")
    if "enum" in p:
        vals = p["enum"]
        return f"{t} ({', '.join(map(str, vals))})" if len(vals) <= 6 else f"{t} (enum)"
    if t == "array" and "items" in p:
        return f"array of {type_of(p['items'])}"
    return t


def inputs(schema):
    props = (schema or {}).get("properties") or {}
    if not props:
        return "—"
    req = set((schema or {}).get("required") or [])
    return "; ".join(f"`{k}`{'*' if k in req else ''}: {type_of(v)}" for k, v in props.items())


def status(o):
    return o["status"] if o["status"] == "available" else f"{o['status']} – requires {o.get('requires', 'a missing component')}"


def agents_md():
    out = [
        "# COSOUND – Specialized agents: responsibilities (Prompt 06, updated by Prompt 07)",
        "",
        "Generated from `packages/agents/<id>.json` by `packages/agents/render_docs.py` (the manifests are the source of truth,",
        "compiled into the app and checked by tests).",
        "Status: **available** = runs today on real engines; **planned** = declared with the missing component, the runtime answers `NOT_IMPLEMENTED` (never simulated).",
        "Fields marked * are required. All agents: contract version 1.",
        "",
        "| | Agent | Role | Operations available | Permissions |",
        "|---|---|---|---|---|",
    ]
    for letter, a in zip(LETTERS, AGENTS):
        perms = ", ".join(sorted(a["permissions"])) or "—"
        out.append(f"| {letter} | [{a['name']}](#{anchor(letter, a)}) | {a['role']['en']} | {available(a)}/{len(a['operations'])} | {perms} |")
    for letter, a in zip(LETTERS, AGENTS):
        out += ["", f"## {letter}. {a['name']}", "", f"`{a['id']}` · v{a['version']} · {a['role']['en']}", "", "**Competencies**", ""]
        out += [f"- {c}" for c in a["competencies"]["en"]]
        out += ["", "**Operations**", "", "| Operation | Status | Risk | Basis | Input | Output |", "|---|---|---|---|---|---|"]
        for o in a["operations"]:
            appr = " (approval)" if o.get("requires_approval") else ""
            out.append(f"| `{o['name']}` – {o['summary']['en']} | {status(o)} | {o['risk']}{appr} | {o.get('basis') or '—'} | {inputs(o.get('input'))} | {o.get('output') or '—'} |")
        tools = ", ".join(f"`{t}`" for t in sorted(a["tools"])) or "none"
        perms = ", ".join(f"`{p}`" for p in sorted(a["permissions"])) or "none"
        out += ["", f"**Tools (level 3):** {tools}", "", f"**Permissions:** {perms}", "",
                f"**Input formats:** {', '.join(a['input_formats'])} · **Output formats:** {', '.join(a['output_formats'])}", ""]
        if a.get("collaborates_with"):
            out += [f"**Works with (through the central Agent, never directly):** {', '.join(a['collaborates_with'])}", ""]
        out += ["**Limits**", ""] + [f"- {l}" for l in a["limits"]["en"]]
    return "\n".join(out) + "\n"


def registries_md():
    out = [
        "# COSOUND – Agent Registry, Tool Registry, permissions",
        "",
        "Generated from `packages/agents/` (`<agent>.json`, `tools.json`, `permissions.json`) by `packages/agents/render_docs.py`.",
        "At runtime the same data, with live state (agent enabled, permission granted, which agents use a tool), is returned by the",
        "Agent API message `registry.get`.",
        "",
        "## 1. Agent Registry",
        "",
        "One JSON manifest per agent, schema `packages/contracts/agent-manifest.schema.json`. Each declares: id, name, version, role,",
        "competencies, operations (status, risk, approval, input JSON Schema, output, tools, permissions, basis, requires), tools, permissions,",
        "limits, input/output formats, the agents whose outputs it can use. A test fails if a manifest and the code disagree",
        "(available operations = implemented operations; tools and permissions exist; available operations use only available tools).",
        "",
        "| Agent | Version | State | Operations (available/total) | Tools |",
        "|---|---|---|---|---|",
    ]
    for a in AGENTS:
        n, tot = available(a), len(a["operations"])
        state = "active" if n == tot else "planned" if n == 0 else "partial"
        out.append(f"| `{a['id']}` | {a['version']} | {state} | {n}/{tot} | {', '.join(sorted(a['tools']))} |")
    total = sum(len(a["operations"]) for a in AGENTS)
    out += ["", f"Total: {sum(available(a) for a in AGENTS)} of {total} operations available.", "",
            "Enable/disable: per agent, by the user (setting `agent_enabled.<id>`, logged). A disabled agent answers `AGENT_DISABLED`.", "",
            "## 2. Tool Registry (level 3: engines and tools)", "",
            "Agents never call each other: they only use these tools, through the execution context (`Ctx`), which records `tools_used` in every result.", "",
            "| Tool | Status | Module | What it does | Used by |", "|---|---|---|---|---|"]
    for t in sorted(TOOLS, key=lambda t: t["status"] != "available"):
        st = t["status"] if t["status"] == "available" else f"{t['status']} – {t.get('requires', '')}"
        users = ", ".join(a["id"] for a in AGENTS if t["id"] in a["tools"]) or "—"
        out.append(f"| `{t['id']}` | {st} | {('`' + t['module'] + '`') if t.get('module') else '—'} | {t['description']['en']} | {users} |")
    out += ["", "## 3. Permissions", "",
            "Checked by the runtime before an operation runs (`PERMISSION_DENIED` otherwise). User toggles live in Settings; non-grantable permissions",
            "belong to components that do not exist yet.", "",
            "| Permission | Setting | Default | Grantable | Meaning |", "|---|---|---|---|---|"]
    for p in PERMS:
        out.append(f"| `{p['id']}` | {('`' + p['setting'] + '`') if p.get('setting') else '—'} | {'on' if p['default'] else 'off'} | {'yes' if p['grantable'] else 'no'} | {p['description']['en']} |")
    out += ["", "File access is enforced in code, not only declared: `Ctx::authorized_file` accepts only files inside online authorized library folders,",
            "imported files, or COSOUND's own data folder; `Ctx::write_file` writes only inside `generated/` of COSOUND's data folder and returns the SHA-256."]
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    for name, text in (("AGENTS.md", agents_md()), ("REGISTRIES.md", registries_md())):
        with open(os.path.join(DOCS, name), "w", encoding="utf-8") as f:
            f.write(text)
        print("wrote", os.path.normpath(os.path.join(DOCS, name)))
