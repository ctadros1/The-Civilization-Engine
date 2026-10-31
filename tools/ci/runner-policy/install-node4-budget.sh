#!/usr/bin/env bash
set -euo pipefail
[[ $EUID == 0 && $(hostname) == tce-ci-node4 ]]
cd "$(dirname "$0")"
python3 -c 'import yaml'
install -o root -g root -m 755 node4_budget.py /usr/local/lib/tce-runner/node4_budget.py
install -o root -g root -m 755 node4_watchdog.py /usr/local/lib/tce-runner/node4_watchdog.py
hook=/usr/local/lib/tce-runner/job-started.sh
if ! grep -q '^/usr/local/lib/tce-runner/node4_budget.py$' "$hook"; then
  printf '\n/usr/local/lib/tce-runner/node4_budget.py\n' >> "$hook"
fi
chown root:root "$hook"
chmod 755 "$hook"
cat > /etc/systemd/system/tce-node4-watchdog.service <<'UNIT'
[Unit]
Description=Six-hour runtime ceiling for node4 Actions workers
After=tce-actions-runner.service
[Service]
ExecStart=/usr/bin/python3 /usr/local/lib/tce-runner/node4_watchdog.py
Restart=always
RestartSec=5
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable tce-node4-watchdog.service
systemctl restart tce-node4-watchdog.service
systemctl is-active tce-node4-watchdog.service
