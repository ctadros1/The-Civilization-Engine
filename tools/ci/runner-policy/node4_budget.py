#!/usr/bin/python3
"""Refuse node4 workloads without a declared budget of at most six hours."""
import json
import os
from pathlib import Path
import re
import urllib.request

REPO = "ctadros1/The-Civilization-Engine"


def budget(workflow, job_id, event):
    job = workflow.get("jobs", {}).get(job_id, {})
    if job_id == "bench":
        raise ValueError("Timing benchmarks belong on node3")
    value = job.get("timeout-minutes")
    if isinstance(value, str):
        # Interpret only our documented input expression; never evaluate YAML code.
        expression = re.fullmatch(r"\$\{\{\s*fromJSON\(inputs\.max_minutes\s*\|\|\s*'([0-9]+)'\)\s*\}\}", value)
        if expression:
            value = event.get("inputs", {}).get("max_minutes") or expression[1]
        if isinstance(value, str) and re.fullmatch(r"[0-9]+", value):
            value = int(value)
    if type(value) is not int or not 1 <= value <= 360:
        raise ValueError("Node4 requires an explicit workload budget of 1–360 minutes; use tce-auto-long otherwise")
    return value


def main():
    import yaml
    if os.environ.get("GITHUB_REPOSITORY") != REPO:
        raise SystemExit("Node4 refuses this repository")
    reference = os.environ.get("GITHUB_WORKFLOW_REF", "")
    match = re.fullmatch(re.escape(REPO) + r"/(\.github/workflows/[A-Za-z0-9_.-]+\.ya?ml)@.+", reference)
    revision = os.environ.get("GITHUB_WORKFLOW_SHA", "")
    if not match or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise SystemExit("Node4 cannot establish the workflow definition; refusing before checkout")
    url = f"https://raw.githubusercontent.com/{REPO}/{revision}/{match[1]}"
    try:
        with urllib.request.urlopen(url, timeout=20) as response:
            source = response.read(1024 * 1024 + 1)
        if len(source) > 1024 * 1024:
            raise ValueError("Workflow exceeds policy parser size limit")
        workflow = yaml.safe_load(source)
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
        minutes = budget(workflow, os.environ["GITHUB_JOB"], event)
    except Exception as error:
        raise SystemExit(f"Node4 admission refused: {error}") from None
    print(f"Node4 admitted declared {minutes}-minute workload; independent six-hour watchdog enabled")


if __name__ == "__main__":
    main()
