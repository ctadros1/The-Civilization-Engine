#!/usr/bin/python3
"""Bound only this service's runner worker processes to six hours."""
import os
from pathlib import Path
import subprocess
import time

LIMIT_SECONDS = 6 * 60 * 60


def expired_workers(proc=Path("/proc"), clock_ticks=None):
    now = float((proc / "uptime").read_text().split()[0])
    ticks = clock_ticks or os.sysconf("SC_CLK_TCK")
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            if (entry / "comm").read_text().strip() != "Runner.Worker":
                continue
            if "/tce-actions-runner.service" not in (entry / "cgroup").read_text():
                continue
            fields = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            started = int(fields[19]) / ticks
            if now - started >= LIMIT_SECONDS:
                yield int(entry.name), started
        except (FileNotFoundError, ProcessLookupError):
            continue


def main():
    while True:
        for pid, started in expired_workers():
            # Verify the PID still identifies the same expired worker before killing.
            if (pid, started) in list(expired_workers()):
                print(f"Node4 six-hour limit: stopping runner service tree for worker {pid}", flush=True)
                # Killing the whole service cgroup also stops workload children.
                # Restart=always reconnects its listener; this watchdog is separate.
                subprocess.run(["/usr/bin/systemctl", "kill", "--kill-whom=all", "--signal=SIGKILL",
                                "tce-actions-runner.service"], check=True, timeout=15)
                break
        time.sleep(5)


if __name__ == "__main__":
    main()
