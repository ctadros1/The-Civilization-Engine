"""Summarise long-run reports side by side: notables comparisons and dashboard rows and worlds.

usage: python3 -I compare.py <label=dir> [<label=dir> ...]
"""
import json
import os
import sys


def load(d, name):
    p = os.path.join(d, name)
    return json.load(open(p)) if os.path.exists(p) else None


runs = [a.split("=", 1) for a in sys.argv[1:]]
print("== notables")
for label, d in runs:
    n = load(d, "notables.json")
    if not n:
        print(f"{label}: none")
        continue
    for c in n.get("comparisons", []):
        print(f"{label}: {json.dumps(c)[:300]}")
print("== dashboard")
for label, d in runs:
    b = load(d, "dashboard.json")
    if not b:
        print(f"{label}: no dashboard (not finished)")
        continue
    print(f"{label}: {b.get('seconds', 0) / 3600:.1f} h; worlds {len(b.get('worlds', []))}")
    for r in b.get("rows", []):
        print(f"  [{r.get('grade')}] {r.get('name')}: {r.get('text', '')[:220]}")
