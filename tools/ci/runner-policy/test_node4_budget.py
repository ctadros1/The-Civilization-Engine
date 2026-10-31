import json
from pathlib import Path
import tempfile
import unittest
from node4_budget import budget
from node4_watchdog import expired_workers


class BudgetTests(unittest.TestCase):
    def test_declared_six_hours_is_allowed(self):
        self.assertEqual(budget({"jobs": {"demo": {"timeout-minutes": 360}}}, "demo", {}), 360)

    def test_long_unknown_and_expressions_are_refused(self):
        for value in (361, 720, None, True, "${{ some.expression }}"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                budget({"jobs": {"demo": {"timeout-minutes": value}}}, "demo", {})

    def test_dynamic_budget_is_checked_not_evaluated(self):
        workflow = {"jobs": {"demo": {"timeout-minutes": "${{ fromJSON(inputs.max_minutes || '360') }}"}}}
        self.assertEqual(budget(workflow, "demo", {}), 360)
        with self.assertRaises(ValueError):
            budget(workflow, "demo", {"inputs": {"max_minutes": "720"}})
        with self.assertRaises(ValueError):
            budget(workflow, "demo", {"inputs": {"max_minutes": "__import__('os')"}})

    def test_benchmark_is_refused(self):
        with self.assertRaises(ValueError):
            budget({"jobs": {"bench": {"timeout-minutes": 60}}}, "bench", {})

    def test_watchdog_targets_only_expired_workers_in_the_correct_service(self):
        with tempfile.TemporaryDirectory() as directory:
            proc = Path(directory)
            (proc / "uptime").write_text("21601 0")
            for pid, name, group, start in ((1, "Runner.Worker", "tce-actions-runner.service", 0),
                                            (2, "Runner.Worker", "other.service", 0),
                                            (3, "Runner.Listener", "tce-actions-runner.service", 0),
                                            (4, "Runner.Worker", "tce-actions-runner.service", 2),
                                            (5, "Runner.Worker", "tce-actions-runner.service-other", 0)):
                entry = proc / str(pid)
                entry.mkdir()
                (entry / "comm").write_text(name)
                (entry / "cgroup").write_text(f"0::/system.slice/{group}")
                (entry / "stat").write_text(f"{pid} ({name}) " + " ".join(["S"] + ["0"] * 18 + [str(start * 100)]))
            self.assertEqual(list(expired_workers(proc, 100)), [(1, 0)])


if __name__ == "__main__":
    unittest.main()
