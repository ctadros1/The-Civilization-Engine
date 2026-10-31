# GitHub Actions runners

The repository has two permanent Ubuntu 24.04 runner VMs and optional capacity on servernode4, each running one job at a time:

| Runner | Host | vCPUs | RAM | Disk | Labels |
| --- | --- | ---: | ---: | ---: | --- |
| `tce-ci-node1` | servernode1 | 4 | 4 GiB | 160 GiB | `tce-ci`, `tce-build`, `tce-fast`, `node1` |
| `tce-ci-node3` | servernode3 | 12 | 16 GiB | 200 GiB | `tce-ci`, `tce-build`, `tce-long`, `node3` |
| `tce-ci-node4` (optional) | servernode4 | 16 | 24 GiB | 240 GiB | `tce-ci`, `tce-build`, `tce-fast`, `node4` |

Rust compilation and test concurrency are limited to 2 on node1, 6 on node3, and 8 on node4. These are initial allocations that leave memory for existing guests; host core count is not runner parallelism. The installed tools are Rust 1.97.0 with rustfmt/clippy, Node 22, flatc 25.12.19, C/C++ build tools, and Chromium's system dependencies. The repository lockfile selects the Playwright browser installed by a job.

## Routing and use

The management VM runs a small scheduler that moves shared policy labels among the existing runners every five seconds. It prefers an **online, idle node4**, then node3, then node1. Busy machines spill work to other idle capacity; when all eligible machines are busy, jobs remain in a common queue and the label moves to whichever eligible machine becomes free. Removing a runner's required `tce-ci` label drains it; the scheduler respects that choice. Each VM still runs one job at a time.

| Work | Required policy label | Eligible machines |
| --- | --- | --- |
| Bounded work, declared budget at most six hours | `tce-auto` | node4 → node3 → node1, preferring idle capacity |
| Work that may exceed six hours, or has no safe short estimate | `tce-auto-long` | node3 → node1; never node4 |
| Timing benchmarks | explicit `node3` | node3 only, for comparable measurements |

All jobs also require `self-hosted, Linux, X64, tce-ci`. The policy labels are managed by the scheduler; do not add them manually. `tce-long` remains a legacy node3 label for older workflow revisions. New scheduled simulation checks, web/schema checks and verification chains use the automatic policy instead of that bottleneck. Windows checks and pull-request checks stay GitHub-hosted.

`Homelab validation` can be dispatched from **the workflow on main**, with `ref` set to the commit or working branch to test. This keeps current routing separate from an older branch's workflow definition. Leaving `ref` empty tests the dispatch revision. Pushing `ci/candidate` also starts the full chain; it never automatically merges code.

- `suite=full`: commons/kernel formatting, clippy and tests; content/smoke; schema; web build/unit/browser checks.
- `suite=long`: ten-year smoke, consistency, notables and fifty-year dashboard.
- `suite=demo`: run the selected milestone example and preserve its record, including partial output on timeout.
- `suite=bench`: time the selected revision against `base` on node3.
- `suite=health`: check both permanent runners, or only node4 when explicitly selected. It does not require optional node4 by default.
- `suite=routing-check`: a small operational probe; `hold_seconds=30` allows a concurrent probe to verify spillover without running a simulation.

`runner=tce-ci` means automatic routing. The long/demo `max_minutes` budget defaults to 360; choosing 720 or 1440 sends work only to permanent machines, including when node4 was requested. For work that might exceed six hours, choose one of those longer budgets instead of relying on a six-hour timeout to cut it short. Explicit node1/node3 pins remain available. Explicit bounded node4 pins wait if it is offline and intentionally bypass fallback. Chains accept legacy shared-label JSON as automatic routing; explicit node pins remain explicit.

GitHub cannot migrate an executing process. Losing a machine during a job can interrupt it; the scheduler moves **queued work**, not half-finished simulations. Existing queued/running workflows retain their original definitions and labels; dispatch from main with `ref` to use the new policy. Existing timing benchmarks stay on node3.

## Isolation and maintenance

This is a **public repository**. The VMs use a dedicated host-only bridge with scoped NAT and host-enforced firewall rules. New runner-originated connections to private, loopback, link-local, metadata, multicast, and reserved networks are dropped. IPv6 forwarding is blocked. Management SSH uses the existing source-restricted administration paths; the scheduler opens no inbound port. The runner user has no sudo or host credentials; a separate administrator account handles installation.

A root-owned job-start hook refuses events other than owner-authored pushes, dispatches, and schedules on the allowed branch families. Runner binaries, registration files, and the hook are root-owned; the job account can write its work directory, diagnostic directory, tool cache, and home. This is a persistent runner, not a clean VM per job. Only trusted code should be dispatched or approved. The hook is an extra guard, not permission to run untrusted code. Fork approval is required for all external contributors.

GitHub runner self-updates are disabled for the unprivileged account. A daily root-owned updater checks the official release and its SHA-256 digest, defers while a worker is active, and restarts the service after an idle update. Compiler output persists under the runner home outside the checkout, with links recreated after checkout cleaning. Source checkouts are cleaned normally. This preserves build caches without keeping stale untracked source files. Dependency caches also persist in these disposable guests. Diagnostic logs expire after 14 days. The full chain refuses to start below 20 GiB free. Watch disk availability; maintenance must stay within the runner VM's work/cache paths. Do not delete project saves or other guests' data.

Provisioning, bootstrap, VM addresses, firewall details, and recovery commands live in the private `homelab-infrastructure` operational record. The VM build disks are excluded from backup because they are rebuildable; there is no claim of a tested backup/restore. Source and durable evidence belong in GitHub, not only on a runner.

## Node4 six-hour policy

A root-owned admission hook checks the exact workflow definition at `GITHUB_WORKFLOW_SHA` before checkout. It refuses missing, unknown or over-360-minute budgets and benchmark jobs. Only static budgets and the documented `max_minutes` expression are interpreted; arbitrary expressions are not evaluated. No GitHub credential is copied to node4 for this check; this public repository's workflow definition is read over HTTPS.

An independent root-owned watchdog bounds the worker service tree to six hours, including workload children. GitHub job timeouts provide the normal stop; the watchdog is the backstop for legacy or incorrect workflows. At the ceiling it terminates the service tree, and the existing restart policy reconnects the runner. A timed-out job fails; it is not moved to another host. Runtime policy source and focused tests are in `tools/ci/runner-policy/`.

Scheduler installation, current state, credentials placement, drain/recovery commands and rollback are recorded in the private homelab runbook. It uses the management VM's existing GitHub CLI authentication; workload tokens remain read-only. If that VM or its GitHub API access is unavailable, the last policy-label assignment remains until the scheduler recovers. Monitoring its service and state timestamp distinguishes this from an ordinary busy queue.

## Optional servernode4

Servernode4 has an isolated VM with the shared `tce-ci` label. It adds capacity while Proxmox is online without making ordinary pool jobs depend on it. Guest and runner autostart are enabled; it is offline while the PC is off or running Windows. It is excluded from the default permanent health matrix and has no legacy `tce-long` label. Its `tce-auto` policy label is present only when selected by the scheduler. Before shutting it down or switching operating systems, stop accepting new jobs and let the current job finish. Abrupt loss during execution interrupts that job; the shared pool does not resume a half-finished process. Queued pool jobs use an available matching runner, while an explicitly node4-pinned job waits for it. Its private guest network uses host NAT, so location changes require no guest IP or GitHub registration changes. The home location remains physically untested.

Benchmarks must be pinned to one machine with a baseline measured on that same VM, CPU allocation, profile, and workload. No performance improvement or 10% budget comparison is claimed from installing runners.

Sources: [GitHub runner routing](https://docs.github.com/en/actions/reference/runners/self-hosted-runners), [self-hosted runner security](https://docs.github.com/en/actions/reference/security/secure-use).
