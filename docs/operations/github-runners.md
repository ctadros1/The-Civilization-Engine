# GitHub Actions runners

The repository has two dedicated Ubuntu 24.04 VMs, each running one job at a time:

| Runner | Host | vCPUs | RAM | Disk | Labels |
| --- | --- | ---: | ---: | ---: | --- |
| `tce-ci-node1` | servernode1 | 4 | 4 GiB | 160 GiB | `tce-ci`, `tce-build`, `tce-fast`, `node1` |
| `tce-ci-node3` | servernode3 | 12 | 16 GiB | 200 GiB | `tce-ci`, `tce-build`, `tce-long`, `node3` |

Rust compilation and test concurrency are limited to 2 on node1 and 6 on node3. These are initial allocations that leave memory for existing guests; host core count is not runner parallelism. The installed tools are Rust 1.97.0 with rustfmt/clippy, Node 22, flatc 25.12.19, C/C++ build tools, and Chromium's system dependencies. The repository lockfile selects the Playwright browser installed by a job.

## Routing and use

GitHub routes `[self-hosted, Linux, X64, tce-ci]` to any matching online, idle runner. There is no local load balancer. Trusted owner pushes on `main`, `ci/*`, `codex/*`, and `claude/*` use this pool for CI's Linux web/schema jobs. Windows checks and DLL builds remain on GitHub-hosted Windows. Pull requests remain on GitHub-hosted runners.

`Homelab validation` runs the full Linux chain on pushes to `ci/*`. It can also be dispatched from Actions with:

- `suite=full`: commons/kernel formatting, clippy and tests; content and smoke; generated schema; web build/unit tests; host release build and browser tests.
- `suite=health`: check both permanent runners explicitly, their tools, and lack of sudo.
- `suite=long`: ten-year smoke, consistency, notables, and fifty-year dashboard on `tce-long`.

The full-chain runner choice defaults to `tce-ci` (the available pool); `node1` and `node3` explicitly pin it. Health checks deliberately address both permanent nodes. Neither full nor long jobs depend on both health jobs. Scheduled long jobs use node3; PR versions keep their GitHub-hosted Linux lane. Each runner processes only one job at a time, so long jobs queue rather than compete inside a VM.

A push to `ci/candidate` is an ordinary branch push, never an automatic merge. Logs and reports are visible in Actions. Job tokens have `contents: read`, and checkouts do not persist their credentials.

## Isolation and maintenance

This is a **public repository**. The VMs use a dedicated host-only bridge with scoped NAT and host-enforced firewall rules. New runner-originated connections to private, loopback, link-local, metadata, multicast, and reserved networks are dropped. IPv6 forwarding is blocked. Management SSH is accepted only from the management VM. The runner user has no sudo or host credentials; a separate administrator account handles installation.

A root-owned job-start hook refuses events other than owner-authored pushes, dispatches, and schedules on the allowed branch families. Runner binaries, registration files, and the hook are root-owned; the job account can write its work directory, diagnostic directory, tool cache, and home. This is a persistent runner, not a clean VM per job. Only trusted code should be dispatched or approved. The hook is an extra guard, not permission to run untrusted code. Fork approval is required for all external contributors.

GitHub runner self-updates are disabled for the unprivileged account. A daily root-owned updater checks the official release and its SHA-256 digest, defers while a worker is active, and restarts the service after an idle update. Builds and dependency caches persist in these disposable guests. Diagnostic logs expire after 14 days. The full chain refuses to start below 20 GiB free. Watch disk availability; maintenance must stay within the runner VM's work/cache paths. Do not delete project saves or other guests' data.

Provisioning, bootstrap, VM addresses, firewall details, and recovery commands live in the private `homelab-infrastructure` operational record. The VM build disks are excluded from backup because they are rebuildable; there is no claim of a tested backup/restore. Source and durable evidence belong in GitHub, not only on a runner.

## Adding servernode4 later

Register an isolated runner with the shared `tce-ci` label when servernode4 can be provisioned. It can add capacity while online without making ordinary pool jobs depend on it. Do not add it to the permanent health matrix or give it the sole `tce-long` label. Before shutting it down or switching operating systems, stop accepting new jobs and let the current job finish. Abrupt loss during execution interrupts that job; the shared pool does not resume a half-finished process. Queued jobs can use an available matching runner, while an explicitly node4-pinned job waits for it.

Benchmarks must be pinned to one machine with a baseline measured on that same VM, CPU allocation, profile, and workload. No performance improvement or 10% budget comparison is claimed from installing runners.

Sources: [GitHub runner routing](https://docs.github.com/en/actions/reference/runners/self-hosted-runners), [self-hosted runner security](https://docs.github.com/en/actions/reference/security/secure-use).
