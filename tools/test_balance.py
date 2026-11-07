"""Deterministic statistics/scheduling/cache tests. No GUI and no game-rule clone."""
import copy
import gzip
import itertools
import json
from pathlib import Path
import random
import tempfile
import unittest
from unittest.mock import patch

from balance.engine import Cache, pair_key, scores, seed_for, valid_blocks
from balance.report import make_report
from balance.sampling import select_pairs
from balance.statistics import Model, confidence_sequence, inverse, solve
from balance_fingerprint import manifest
from evaluate_balance import evaluate, main
from benchmark_balance import benchmark, read_run

CONFIG = dict(budget=48, seed=42, level=1, max_steps=20000, sampler="adaptive",
              batch_size=4, lower=.45, upper=.55, extreme=.75, width=.2)
MANIFEST = {"schema": 1, "shared": "shared-v1", "heroes": [{"id": i, "name": str(i), "fingerprint": str(i)} for i in range(4)]}


def block(a, b, seed, results=(0, 1)):
    return [dict(hero=x, enemy=y, seed=seed, winner=w, termination="draw" if w is None else "win")
            for (x, y), w in zip(((a, b), (b, a)), results)]


class FakeEngine:
    """A deterministic data source for orchestration tests, not a game simulator."""
    def __init__(self, data=None):
        self.manifest = copy.deepcopy(data or MANIFEST)
        self.calls = []

    def blocks(self, jobs):
        self.calls.extend(jobs)
        return [block(*pair, seed, (seed % 2, (seed >> 1) % 2)) for pair, seed, _, _ in jobs]


class StatisticsTests(unittest.TestCase):
    def test_covariance_matches_linear_solves(self):
        matrix = [[4., -1., 2.], [-1., 5., 1.], [2., 1., 6.]]
        inv = inverse(matrix)
        for i in range(3):
            column = solve(matrix, [float(i == j) for j in range(3)])
            for j in range(3):
                self.assertAlmostEqual(column[j], inv[j][i])

    def test_empty_and_all_win_intervals(self):
        self.assertEqual(confidence_sequence([]), [0, 1])
        lo, hi = confidence_sequence([1.] * 100)
        self.assertGreater(lo, .9)
        self.assertEqual(hi, 1)
        low = confidence_sequence([0.] * 100)
        self.assertAlmostEqual(low[1], 1 - lo)

    def test_paired_outcomes_are_one_observation(self):
        self.assertEqual(scores(block(0, 1, 3, (0, 0))), (1, 0))
        self.assertEqual(scores(block(0, 1, 3, (None, None))), (.5, .5))
        row = block(0, 1, 3)
        row[0].update(winner=None, termination="aborted")
        self.assertIsNone(scores(row))

    def test_rating_handles_separation_and_disconnection(self):
        model = Model([0, 1, 2], {(0, 1): [(1, 1)] * 100})
        self.assertGreater(model.predict(0, 1), .95)
        self.assertLess(model.predict(1, 0), .05)
        self.assertGreater(model.rating(2)[1][1] - model.rating(2)[1][0], 100)

    def test_seat_bias_not_mistaken_for_strength(self):
        model = Model([0, 1, 2], {p: [(1, 0)] * 100 for p in itertools.combinations(range(3), 2)})
        self.assertGreater(model.theta[-1], 2)
        self.assertAlmostEqual(model.predict(0, 1), .5, places=6)

    def test_cycle_remains_balanced_and_visible(self):
        rows = {(0, 1): [block(0, 1, i) for i in range(600)],
                (1, 2): [block(1, 2, i) for i in range(600)],
                (0, 2): [block(0, 2, i, (1, 0)) for i in range(600)]}
        report = make_report(MANIFEST, [0, 1, 2], rows, CONFIG, 3600, 0)
        self.assertIn([0, 1, 2], report["confirmed_cycles"])
        for hero in report["heroes"]:
            self.assertAlmostEqual(hero["predicted_score"], .5)
            self.assertNotIn(hero["status"], ("overpowered", "underpowered"))
        self.assertTrue(all(p["extreme"] for p in report["pairs"]))

    def test_missing_edges_cannot_be_declared_verified(self):
        report = make_report(MANIFEST, [0, 1, 2], {}, CONFIG, 0, 0)
        self.assertTrue(all(h["status"] == "needs_samples" for h in report["heroes"]))
        self.assertTrue(all(p["status"] == "predicted" for p in report["pairs"]))

    def test_aborted_pair_abstains_even_with_many_wins(self):
        rows = [block(0, 1, i) for i in range(300)]
        rows[-1][0].update(winner=None, termination="aborted")
        result = make_report(MANIFEST, [0, 1], {(0, 1): rows}, CONFIG, 600, 0)
        self.assertEqual(result["pairs"][0]["score_cs95"], [0, 1])
        self.assertFalse(result["pairs"][0]["extreme"])

    def test_cs_coverage_under_optional_stopping(self):
        rng = random.Random(391)
        failures = 0
        for _ in range(200):
            values = []
            for n in range(1, 101):
                values.append((float(rng.random() < .3) + float(rng.random() < .3)) / 2)
                if n % 5 == 0:
                    lo, hi = confidence_sequence(values)
                    if not lo <= .3 <= hi:
                        failures += 1
                        break
        self.assertLessEqual(failures, 10)


class PipelineTests(unittest.TestCase):
    def test_focus_excludes_unrelated_pairs_in_both_samplers(self):
        for mode in ("adaptive", "uniform"):
            for focus in ([1], [1, 2]):
                with self.subTest(mode=mode, focus=focus), tempfile.TemporaryDirectory() as tmp:
                    cache = Cache(Path(tmp) / "cache.sqlite")
                    self.addCleanup(cache.close)
                    engine = FakeEngine()
                    config = dict(CONFIG, sampler=mode, focus=focus)
                    report, records = evaluate(engine, cache, [0, 1, 2, 3], config, False)
                    expected = {p for p in itertools.combinations(range(4), 2) if set(p).intersection(focus)}
                    self.assertEqual(set(records), expected)
                    self.assertEqual({pair for pair, *_ in engine.calls}, expected)
                    self.assertEqual(report["new_games"], CONFIG["budget"])
                    self.assertEqual(report["config"]["focus"], focus)
                    for row in report["pairs"]:
                        if (row["a"], row["b"]) not in expected:
                            self.assertEqual(row["games"], 0)
                            self.assertEqual(row["score_cs95"], [0, 1])

    def test_focus_cache_reuse_resume_and_role_invalidation(self):
        with tempfile.TemporaryDirectory() as tmp:
            cache = Cache(Path(tmp) / "cache.sqlite")
            self.addCleanup(cache.close)
            engine = FakeEngine()
            _, original = evaluate(engine, cache, [0, 1, 2, 3], CONFIG, False)
            reused = 2 * sum(len(rows) for pair, rows in original.items() if 1 in pair)
            config = dict(CONFIG, focus=[1], budget=reused + 8)
            engine.calls.clear()
            first, _ = evaluate(engine, cache, [0, 1, 2, 3], config, False)
            self.assertEqual(first["reused_games"], reused)
            self.assertEqual(first["new_games"], 8)
            self.assertTrue(all(1 in pair for pair, *_ in engine.calls))
            engine.calls.clear()
            replay, _ = evaluate(engine, cache, [0, 1, 2, 3], config, False)
            self.assertEqual(replay["new_games"], 0)
            self.assertEqual(engine.calls, [])
            engine.manifest["heroes"][1]["fingerprint"] = "changed"
            reset, _ = evaluate(engine, cache, [0, 1, 2, 3], config, False)
            self.assertEqual(reset["reused_games"], 0)
            self.assertEqual(reset["new_games"], config["budget"])
            engine.calls.clear()
            full, _ = evaluate(engine, cache, [0, 1, 2, 3], dict(CONFIG, budget=96), False)
            self.assertGreater(full["reused_games"], 0)
            self.assertEqual(cache.load(pair_key(engine.manifest, (0, 2), 42, 1, 20000)), original[(0, 2)] + [
                block(*pair, seed, (seed % 2, (seed >> 1) % 2))
                for pair, seed, _, _ in engine.calls if pair == (0, 2)])

    def test_focus_cli_validation(self):
        for focus in ("4", "1,1", "", "bad"):
            with self.subTest(focus=focus), patch("evaluate_balance.Engine", return_value=FakeEngine()), patch("sys.stderr"):
                with self.assertRaises(SystemExit) as raised:
                    main(["--roster", "0,1,2", "--focus", focus])
                self.assertEqual(raised.exception.code, 2)

    def test_focus_cli_reports_and_replays_selected_pairs(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            arguments = ["--roster", "0,1,2,3", "--focus", "1", "--budget", "12",
                         "--cache", str(directory / "cache.sqlite"), "--out", str(directory / "out")]
            engine = FakeEngine()
            with patch("evaluate_balance.Engine", return_value=engine), patch("sys.stdout"), patch("sys.stderr"):
                self.assertEqual(main(arguments), 0)
                report = json.loads((directory / "out/report.json").read_text())
                self.assertEqual(report["config"]["focus"], [1])
                self.assertEqual(report["new_games"], 12)
                matches = (directory / "out/matches.jsonl").read_text().splitlines()
                self.assertEqual(len(matches), 6)
                self.assertTrue(all(1 in json.loads(row)["pair"] for row in matches))
                self.assertIn("定向评估角色 [1]", (directory / "out/report.md").read_text())
                engine.calls.clear()
                self.assertEqual(main(arguments), 0)
                self.assertEqual(engine.calls, [])
                replay = json.loads((directory / "out/report.json").read_text())
                self.assertEqual(replay["reused_games"], 12)

    def test_benchmark_import_checks_provenance_and_compressed_evidence(self):
        pair = (0, 1)
        seed = seed_for(CONFIG["seed"], pair, 0)
        games = block(*pair, seed)
        for game in games:
            game.update(level=1, max_steps=20000)
        report = make_report(MANIFEST, [0, 1], {pair: [games]}, CONFIG, 2, 0)
        entry = {"pair": pair, "sample": 0, "games": games,
                 "pair_key": pair_key(MANIFEST, pair, CONFIG["seed"], 1, 20000)}
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            (path / "report.json").write_text(json.dumps(report), encoding="utf-8")
            (path / "matches.jsonl.gz").write_bytes(gzip.compress(json.dumps(entry).encode()))
            _, imported = read_run(path)
            self.assertEqual(imported[pair], [games])
            entry["games"][0]["seed"] += 1
            (path / "matches.jsonl.gz").write_bytes(gzip.compress(json.dumps(entry).encode()))
            with self.assertRaisesRegex(ValueError, "job"):
                read_run(path)

    def test_benchmark_rejects_leakage_and_compares_equal_budgets(self):
        roster = [0, 1, 2]
        train = {p: [block(*p, i, (i % 2, (i // 2) % 2)) for i in range(16)]
                 for p in itertools.combinations(roster, 2)}
        ref = {p: [block(*p, i + 100, (i % 2, (i // 2) % 2)) for i in range(32)]
               for p in itertools.combinations(roster, 2)}
        tr = make_report(MANIFEST, roster, train, CONFIG, 96, 0)
        rr = make_report(MANIFEST, roster, ref, dict(CONFIG, seed=43), 192, 0)
        with self.assertRaisesRegex(ValueError, "overlap"):
            benchmark((tr, train), (tr, train), [24], 2)
        result = benchmark((tr, train), (rr, ref), [24, 48], 2)
        self.assertEqual(len(result["results"]), 8)
        self.assertEqual({r["budget"] for r in result["results"]}, {24, 48})
        bad = copy.deepcopy(rr)
        bad["config"]["level"] = 2
        with self.assertRaisesRegex(ValueError, "level"):
            benchmark((tr, train), (bad, ref), [24], 2)

    def test_budget_resume_extension_and_invalidation(self):
        with tempfile.TemporaryDirectory() as tmp:
            cache = Cache(Path(tmp) / "cache.sqlite")
            engine = FakeEngine()
            first, records = evaluate(engine, cache, [0, 1, 2], CONFIG, False)
            self.assertEqual(first["new_games"], 48)
            second, _ = evaluate(engine, cache, [0, 1, 2], CONFIG, False)
            self.assertEqual(second["new_games"], 0)
            self.assertEqual(first["heroes"], second["heroes"])
            extended, _ = evaluate(engine, cache, [0, 1, 2], dict(CONFIG, budget=64), False)
            self.assertEqual(extended["new_games"], 16)
            engine.manifest["heroes"][0]["fingerprint"] = "changed"
            changed, _ = evaluate(engine, cache, [0, 1, 2], dict(CONFIG, budget=64), False)
            self.assertGreater(changed["reused_games"], 0)
            self.assertGreater(changed["new_games"], 0)
            engine.manifest["shared"] = "new-rule"
            reset, _ = evaluate(engine, cache, [0, 1, 2], CONFIG, False)
            self.assertEqual(reset["reused_games"], 0)
            cache.close()

    def test_added_hero_reuses_old_pairs(self):
        with tempfile.TemporaryDirectory() as tmp:
            cache = Cache(Path(tmp) / "cache.sqlite")
            engine = FakeEngine()
            evaluate(engine, cache, [0, 1, 2], CONFIG, False)
            result, _ = evaluate(engine, cache, [0, 1, 2, 3], dict(CONFIG, budget=80), False)
            self.assertEqual(result["reused_games"], 48)
            self.assertEqual(result["new_games"], 32)
            self.assertGreater(next(h["games"] for h in result["heroes"] if h["id"] == 3), 0)
            cache.close()

    def test_version_and_condition_keys(self):
        key = pair_key(MANIFEST, (0, 1), 42, 1, 20000)
        for root, level, steps in ((43, 1, 20000), (42, 2, 20000), (42, 1, 100)):
            self.assertNotEqual(key, pair_key(MANIFEST, (0, 1), root, level, steps))
        changed = copy.deepcopy(MANIFEST)
        changed["heroes"][2]["fingerprint"] = "other"
        self.assertEqual(key, pair_key(changed, (0, 1), 42, 1, 20000))

    def test_seed_and_schedule_reproducibility(self):
        seeds = [seed_for(42, (0, 1), i) for i in range(200)]
        self.assertEqual(len(seeds), len(set(seeds)))
        self.assertNotEqual(seeds[0], seed_for(42, (0, 2), 0))
        a = select_pairs([0, 1, 2, 3], {}, "adaptive", 4, 0, 42)
        self.assertEqual(a, select_pairs([3, 1, 0, 2], {}, "adaptive", 4, 0, 42))
        self.assertEqual(set(sum((list(p) for p in a), [])), {0, 1, 2, 3})

    def test_scheduler_values_paired_variance_not_individual_game_noise(self):
        # Every pair has 50% mean score. Two pairs always split the two games,
        # while (0,1) alternates between both wins and both losses.
        data = {p: [block(*p, i, (0, 0)) for i in range(20)]
                for p in itertools.combinations(range(3), 2)}
        data[(0, 1)] = [block(0, 1, i, (0, 1) if i % 2 else (1, 0)) for i in range(20)]
        chosen = select_pairs([0, 1, 2], data, "adaptive", 1, 1, 42)
        self.assertEqual(chosen, [(0, 1)])

    def test_conflicting_cached_results_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            cache = Cache(Path(tmp) / "cache.sqlite")
            cache.put("x", 0, block(0, 1, 2))
            with self.assertRaises(ValueError):
                cache.put("x", 0, block(0, 1, 2, (1, 0)))
            cache.close()

    def test_fingerprint_local_vs_shared_change(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "config").mkdir()
            (root / "src").mkdir()
            (root / "tools").mkdir()
            for f in ("Cargo.toml", "Cargo.lock", "build.rs", "tools/balance_fingerprint.py", "src/main.rs", "config/skills.txt", "config/projectiles.txt"):
                (root / f).write_text("original")
            (root / "config/heroes.txt").write_text("A|a|100|50|0\nB|b|100|50|1\n")
            (root / "config/ability-loadouts.txt").write_text("0|X|X||text\n1|Y|Y||text\n")
            before = manifest(root, "test")
            (root / "config/heroes.txt").write_text("A|a|110|50|0\nB|b|100|50|1\n")
            after = manifest(root, "test")
            self.assertEqual(before["shared"], after["shared"])
            self.assertNotEqual(before["heroes"][0], after["heroes"][0])
            self.assertEqual(before["heroes"][1], after["heroes"][1])
            (root / "config/projectiles.txt").write_text("changed projectile gravity")
            self.assertNotEqual(after["shared"], manifest(root, "test")["shared"])
            (root / "config/skills.txt").write_text("changed shared skill")
            self.assertNotEqual(after["shared"], manifest(root, "test")["shared"])


if __name__ == "__main__":
    unittest.main()
