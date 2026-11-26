#!/usr/bin/env python3
"""Build apple-au.json from Apple's official header AudioToolbox/AudioUnitParameters.h (macOS SDK).

Each parameter keeps the header's own comment (scope, unit, range, default). Run on a Mac with the
Command Line Tools:  python3 packages/plugin-knowledge/extract_apple_au.py
"""
import json, os, re, subprocess, datetime

sdk = subprocess.check_output(["xcrun", "--show-sdk-path"], text=True).strip()
ver = subprocess.check_output(["xcrun", "--show-sdk-version"], text=True).strip()
hdr = os.path.join(sdk, "System/Library/Frameworks/AudioToolbox.framework/Headers/AudioUnitParameters.h")
text = open(hdr, encoding="utf-8").read()

# AU codes of Apple's units, as registered on macOS (auval -a)
codes = {}
out = subprocess.run(["auval", "-a"], capture_output=True, text=True).stdout
for line in out.splitlines():
    m = re.match(r"^(\w{4}) (.{4}) (\w{4})\s+-\s+Apple: (\S+)", line)
    if m:
        codes[m.group(4)] = f"{m.group(1)}:{m.group(2)}:{m.group(3)}"

units = []
for m in re.finditer(r"// Parameters for (?:the )?(\w+)(?: unit)?[^\n]*\n(?:(?://[^\n]*|\s*)\n)*\s*CF_ENUM\(AudioUnitParameterID\)\s*\{(.*?)\};", text, re.S):
    name, body = m.group(1), m.group(2)
    params, comment = [], None
    for line in body.splitlines():
        s = line.strip()
        if s.startswith("//"):
            comment = s.lstrip("/ ").strip()
            continue
        e = re.match(r"(k\w+)\s*=\s*(\d+)", s)
        if e:
            p = {"constant": e.group(1), "id": int(e.group(2))}
            if comment:
                parts = [x.strip() for x in comment.split(",")]
                p["doc"] = comment
                if len(parts) >= 4:
                    p.update({"scope": parts[0], "unit": parts[1], "range": parts[2], "default": parts[3]})
            params.append(p)
            comment = None
    name = {"Distortion": "AUDistortion", "iOS": "AUReverb2"}.get(name, name)  # header headings that differ from the unit name
    if params:
        units.append({"unit": name, "au": codes.get(name), "parameters": params})

doc = {
    "source": {"title": "AudioToolbox/AudioUnitParameters.h", "publisher": "Apple", "sdk_version": ver,
               "kind": "official SDK header", "extracted": datetime.date.today().isoformat()},
    "note": "Parameter IDs, units, ranges and defaults exactly as documented in the header. Runtime discovery "
            "(cosound-auhost describe) remains the authority for what an installed unit exposes.",
    "units": units,
}
dst = os.path.join(os.path.dirname(os.path.abspath(__file__)), "apple-au.json")
json.dump(doc, open(dst, "w"), indent=1, ensure_ascii=False)
print(f"{len(units)} units, {sum(len(u['parameters']) for u in units)} parameters -> {dst}")
print("without AU code:", [u["unit"] for u in units if not u["au"]])
