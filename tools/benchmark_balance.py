"""Replay competing schedules on real recorded games; evaluate on disjoint seeds."""
import argparse
import gzip
import itertools
import json
import math
from pathlib import Path
import random
import statistics

from balance.engine import pair_key, scores, seed_for, valid_blocks
from balance.report import make_report
from balance.sampling import select_pairs
from evaluate_balance import atomic_text


def read_run(directory):
    report = json.loads((directory / "report.json").read_text(encoding="utf-8"))
    rows = {}
    path = directory / "matches.jsonl"
    text = path.read_text(encoding="utf-8") if path.exists() else gzip.decompress((directory / "matches.jsonl.gz").read_bytes()).decode("utf-8")
    for line in text.splitlines():
        row = json.loads(line)
        pair = tuple(row["pair"])
        config = report["config"]
        if len(pair) != 2 or pair[0] >= pair[1] or not set(pair) <= set(report["roster"]):
            raise ValueError("invalid exported pair")
        expected_key = pair_key(report["manifest"], pair, config["seed"], config["level"], config["max_steps"])
        if row["pair_key"] != expected_key or len(row["games"]) != 2:
            raise ValueError("report/export provenance differs")
        expected_seed = seed_for(config["seed"], pair, row["sample"])
        for game, (hero, enemy) in zip(row["games"], (pair, pair[::-1])):
            actual = tuple(game[k] for k in ("hero", "enemy", "seed", "level", "max_steps"))
            if actual != (hero, enemy, expected_seed, config["level"], config["max_steps"]):
                raise ValueError("exported job does not match report")
        if row["sample"] != len(rows.setdefault(pair, [])):
            raise ValueError("non-contiguous sample indices")
        if scores(row["games"]) is None:
            raise ValueError("benchmark refuses interrupted games")
        rows[pair].append(row["games"])
    if 2 * sum(map(len, rows.values())) != report["games"]:
        raise ValueError("report/export game counts disagree")
    return report, rows


def ranks(values):
    return [sum(x < v for x in values) + (sum(x == v for x in values) - 1) / 2 for v in values]


def rank_correlation(x, y):
    x, y = ranks(x), ranks(y)
    mx, my = statistics.mean(x), statistics.mean(y)
    num = sum((a - mx) * (b - my) for a, b in zip(x, y))
    den = math.sqrt(sum((a - mx)**2 for a in x) * sum((b - my)**2 for b in y))
    return num / den if den else None


def benchmark(training, reference, budgets, repetitions):
    train_report, pool = training
    ref_report, ref = reference
    if train_report["manifest"] != ref_report["manifest"] or sorted(train_report["roster"]) != sorted(ref_report["roster"]):
        raise ValueError("benchmark versions and rosters must match")
    for option in ("level", "max_steps"):
        if train_report["config"][option] != ref_report["config"][option]:
            raise ValueError(f"benchmark {option} differs")
    roster = sorted(train_report["roster"])
    pairs = list(itertools.combinations(roster, 2))
    if any(not pool.get(p) or not ref.get(p) for p in pairs):
        raise ValueError("benchmark requires complete training and reference pair coverage")
    if any({r[0]["seed"] for r in pool[p]} & {r[0]["seed"] for r in ref[p]} for p in pairs):
        raise ValueError("training/reference seeds overlap")
    if not budgets or min(budgets) < 2 or any(b % 2 for b in budgets) or max(budgets) > 2 * sum(map(len, pool.values())):
        raise ValueError("invalid budgets or insufficient training pool")
    if repetitions < 1:
        raise ValueError("repetitions must be positive")
    truth = {p: statistics.mean((x + y) / 2 for x, y in valid_blocks(ref)[p]) for p in pairs}
    def hero_scores(predictions):
        return [statistics.mean(predictions[tuple(sorted((h, o)))] if h < o else 1 - predictions[tuple(sorted((h, o)))]
                                for o in roster if o != h) for h in roster]
    truth_heroes = hero_scores(truth)
    extreme = train_report["config"]["extreme"]
    truth_edges = {(p, int(v > .5)) for p, v in truth.items() if v > extreme or v < 1 - extreme}
    results = []
    for repetition in range(repetitions):
        permuted = {}
        for p in pairs:
            permuted[p] = list(pool[p])
            random.Random(f"benchmark-pool:{repetition}:{p}").shuffle(permuted[p])
        for sampler in ("uniform", "adaptive"):
            data = {p: [] for p in pairs}
            consumed = 0
            for budget in sorted(set(budgets)):
                while consumed < budget:
                    selected = select_pairs(roster, data, sampler, min(8, (budget - consumed) // 2), consumed // 2,
                                            repetition + 319001, {p: len(permuted[p]) for p in pairs})
                    if not selected:
                        raise ValueError("training pool exhausted")
                    for p in selected:
                        data[p].append(permuted[p][len(data[p])])
                    consumed += 2 * len(selected)
                report = make_report(train_report["manifest"], roster, data, train_report["config"], budget, 0)
                pred = {(p["a"], p["b"]): p["prediction"] for p in report["pairs"]}
                empirical = {p: statistics.mean((x + y) / 2 for x, y in valid_blocks(data)[p]) if data[p] else .5 for p in pairs}
                detected = {(p, int(v > .5)) for p, v in pred.items() if v > extreme or v < 1 - extreme}
                precision = len(detected & truth_edges) / len(detected) if detected else None
                recall = len(detected & truth_edges) / len(truth_edges) if truth_edges else None
                brier = statistics.mean((pred[p] - (x + y) / 2)**2 for p in pairs for x, y in valid_blocks(ref)[p])
                results.append({"repetition": repetition, "sampler": sampler, "budget": budget,
                                "pair_rmse": math.sqrt(statistics.mean((pred[p] - truth[p])**2 for p in pairs)),
                                "raw_empirical_pair_rmse": math.sqrt(statistics.mean((empirical[p] - truth[p])**2 for p in pairs)),
                                "heldout_score_brier": brier,
                                "rank_spearman": rank_correlation(hero_scores(pred), truth_heroes),
                                "candidate_extreme_precision": precision, "candidate_extreme_recall": recall,
                                "confirmed_extremes": sum(p["extreme"] for p in report["pairs"]),
                                "reference_mean_in_cs_fraction": statistics.mean(p["score_cs95"][0] <= truth[(p["a"], p["b"])] <= p["score_cs95"][1] for p in report["pairs"]),
                                "candidate_cycles": report["candidate_cycles"], "confirmed_cycles": report["confirmed_cycles"]})
    summaries = []
    for budget in sorted(set(budgets)):
        arms = {s: [r for r in results if r["budget"] == budget and r["sampler"] == s] for s in ("uniform", "adaptive")}
        differences = [a["pair_rmse"] - u["pair_rmse"] for a, u in zip(arms["adaptive"], arms["uniform"])]
        summaries.append({"budget": budget,
                          "uniform_rmse": statistics.mean(r["pair_rmse"] for r in arms["uniform"]),
                          "uniform_raw_empirical_rmse": statistics.mean(r["raw_empirical_pair_rmse"] for r in arms["uniform"]),
                          "adaptive_rmse": statistics.mean(r["pair_rmse"] for r in arms["adaptive"]),
                          "mean_paired_rmse_difference": statistics.mean(differences),
                          "replay_difference_standard_error": statistics.stdev(differences) / math.sqrt(repetitions) if repetitions > 1 else None,
                          "adaptive_wins": sum(d < 0 for d in differences), "repetitions": repetitions})
    return {"schema": 1, "manifest": train_report["manifest"], "roster": roster,
            "level": train_report["config"]["level"], "train_games": train_report["games"], "reference_games": ref_report["games"],
            "training_seed": train_report["config"]["seed"], "reference_seed": ref_report["config"]["seed"],
            "reference_pair_scores": [{"pair": p, "score": truth[p]} for p in pairs],
            "summaries": summaries, "results": results,
            "limitations": "Real-game pools, disjoint training/reference seeds; replay permutations reuse the same finite pools, so replay standard errors are descriptive, not independent-game confidence intervals. Reference rates are noisy estimates, not ground truth. Extreme precision/recall use point estimates and must not be confused with confirmed flags. No universal adaptive advantage is claimed."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--training", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--budgets", default="100,200,400,800")
    parser.add_argument("--repetitions", type=int, default=12)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    result = benchmark(read_run(args.training), read_run(args.reference), [int(b) for b in args.budgets.split(",")], args.repetitions)
    atomic_text(args.out, json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result["summaries"], indent=2))


if __name__ == "__main__":
    main()
