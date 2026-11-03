"""Explainable reports: formal empirical bounds stay distinct from model estimates."""
import itertools
from .engine import valid_blocks
from .statistics import Model, confidence_sequence, estimate, sigmoid


def make_report(manifest, roster, records, config, new_games, reused_games):
    roster = sorted(roster)
    all_pairs = list(itertools.combinations(roster, 2))
    blocks = valid_blocks(records)
    model = Model(roster, blocks)
    alpha = .05 / (2 * len(all_pairs))  # Family-wise across score AND seat CSs.
    pairs = []
    lookup = {}
    for pair in all_pairs:
        data, raw = blocks.get(pair, []), records.get(pair, [])
        values = [(x + y) / 2 for x, y in data]
        aborted = sum(r["termination"] == "aborted" for row in raw for r in row)
        interval = confidence_sequence(values, alpha) if not aborted else [0.0, 1.0]
        side = confidence_sequence([(x + 1 - y) / 2 for x, y in data], alpha) if not aborted else [0.0, 1.0]
        observed = sum(values) / len(values) if values else None
        completed = [row for row in raw if all(r["termination"] != "aborted" for r in row)]
        wins_a = sum(row[0]["winner"] == 0 for row in completed) + sum(row[1]["winner"] == 1 for row in completed)
        prediction = estimate(model, pair, data)
        enough = bool(data) and interval[1] - interval[0] <= config["width"]
        row = {"a": pair[0], "b": pair[1], "games": 2 * len(raw), "seed_pairs": len(data),
               "draws": sum(r["termination"] == "draw" for block in raw for r in block),
               "aborted": aborted, "observed_score": observed, "score_cs95": interval,
               "wins_a_completed_pairs": wins_a,
               "observed_win_rate": wins_a / (2 * len(completed)) if completed else None,
               "bt_prediction": model.predict(*pair), "prediction": prediction,
               "residual": observed - model.predict(*pair) if observed is not None else None,
               "first_seat_score": sum((x + 1 - y) / 2 for x, y in data) / len(data) if data else None,
               "first_seat_cs95": side,
               "a_first_score": sum(x for x, _ in data) / len(data) if data else None,
               "a_second_score": sum(y for _, y in data) / len(data) if data else None,
               "status": "interrupted" if aborted else "verified" if enough else "uncertain" if data else "predicted",
               "extreme": interval[0] > config["extreme"] or interval[1] < 1 - config["extreme"],
               "suspected_extreme": observed is not None and (observed > config["extreme"] or observed < 1 - config["extreme"]),
               "seat_bias": side[0] > .5 or side[1] < .5}
        pairs.append(row)
        lookup[pair] = row
    heroes = []
    names = {h["id"]: h["name"] for h in manifest["heroes"]}
    for hero in roster:
        predictions, lower, upper = [], [], []
        opponents = []
        for other in roster:
            if other == hero:
                continue
            row = lookup[tuple(sorted((hero, other)))]
            p, (lo, hi) = row["prediction"], row["score_cs95"]
            if hero != row["a"]:
                p, lo, hi = 1 - p, 1 - hi, 1 - lo
            predictions.append(p)
            lower.append(lo)
            upper.append(hi)
            opponents.append({"opponent": other, "prediction": p, "interval": [lo, hi]})
        lo, hi = sum(lower) / len(lower), sum(upper) / len(upper)
        status = "overpowered" if lo > config["upper"] else "underpowered" if hi < config["lower"] else "balanced" if lo >= config["lower"] and hi <= config["upper"] else "needs_samples"
        rating, rating_interval = model.rating(hero)
        heroes.append({"id": hero, "name": names[hero], "rating": rating,
                       "rating_interval95_approx": rating_interval,
                       "predicted_score": sum(predictions) / len(predictions),
                       "score_cs95": [lo, hi], "status": status,
                       "games": sum(p["games"] for p in pairs if hero in (p["a"], p["b"])),
                       "reason": f"uniform-opponent score bounds [{lo:.3f}, {hi:.3f}], target [{config['lower']:.3f}, {config['upper']:.3f}]",
                       "opponents": sorted(opponents, key=lambda p: p["prediction"])})
    # Report directed triangles; no claim that untested cycles are recovered.
    edges, suspected = set(), set()
    for p in pairs:
        a, b = p["a"], p["b"]
        if p["score_cs95"][0] > .5:
            edges.add((a, b))
        elif p["score_cs95"][1] < .5:
            edges.add((b, a))
        if p["observed_score"] is not None and p["observed_score"] != .5:
            suspected.add((a, b) if p["observed_score"] > .5 else (b, a))
    def cycles(graph):
        result = []
        for a, b, c in itertools.combinations(roster, 3):
            for triple in ((a, b, c), (a, c, b)):
                x, y, z = triple
                if {(x, y), (y, z), (z, x)} <= graph:
                    result.append(list(triple))
        return result
    return {"schema": 1, "manifest": manifest, "config": config, "roster": roster,
            "new_games": new_games, "reused_games": reused_games,
            "games": sum(p["games"] for p in pairs), "aborted": sum(p["aborted"] for p in pairs),
            "heroes": sorted(heroes, key=lambda h: -h["predicted_score"]), "pairs": pairs,
            "confirmed_cycles": cycles(edges), "candidate_cycles": cycles(suspected),
            "first_seat_model_score": sigmoid(model.theta[-1]),
            "extreme_pair_fraction": sum(p["extreme"] for p in pairs) / len(pairs),
            "coverage": sum(p["seed_pairs"] > 0 for p in pairs) / len(pairs),
            "uncertainty": "Empirical intervals: simultaneous 95% anytime-valid paired-seed confidence sequences; rating intervals: conditional Laplace approximations, not used for balance flags. Predictions on unobserved edges are model assumptions, not validation.",
            "metric": "expected score = P(win) + 0.5 P(draw); aborted games are not draws. Any interrupted pair receives [0,1] empirical bounds.",
            "policy": "Strength is conditional on the specified AI, normal starting-state distribution, roster and executable version."}


def markdown(report):
    lines = ["# BattleBlocks 角色平衡评估", "", report["policy"], "", report["metric"], "", report["uncertainty"], "",
             f"对局 {report['games']}；新跑 {report['new_games']}；复用 {report['reused_games']}；中断 {report['aborted']}；配对覆盖 {report['coverage']:.1%}。", "",
             "| 角色 | 评分（近似 95% 区间） | 综合预测得分率 | 实测同时置信序列 | 状态 |", "|---|---:|---:|---|---|"]
    for h in report["heroes"]:
        lo, hi = h["score_cs95"]
        rlo, rhi = h["rating_interval95_approx"]
        lines.append(f"| {h['name']} | {h['rating']:.0f} ({rlo:.0f}–{rhi:.0f}) | {h['predicted_score']:.1%} | {lo:.1%}–{hi:.1%} | {h['status']} |")
    lines += ["", "## 配对：实测、模型与先手", "", "| A / B | 局数 | 实测得分率 | 预测 | 实测区间 | 先手得分率 | 状态 / 极端关系 |", "|---|---:|---:|---:|---|---:|---|"]
    for p in report["pairs"]:
        observed = "—" if p["observed_score"] is None else f"{p['observed_score']:.1%}"
        first = "—" if p["first_seat_score"] is None else f"{p['first_seat_score']:.1%}"
        lo, hi = p["score_cs95"]
        flag = "confirmed extreme" if p["extreme"] else "suspected extreme" if p["suspected_extreme"] else ""
        lines.append(f"| {p['a']} / {p['b']} | {p['games']} | {observed} | {p['prediction']:.1%} | {lo:.1%}–{hi:.1%} | {first} | {p['status']} {flag} |")
    lines += ["", f"确认循环：{report['confirmed_cycles']}", f"候选循环（证据尚不足）：{report['candidate_cycles']}", "", "## 标记原因", ""]
    lines += [f"- {h['name']}: {h['reason']}" for h in report["heroes"]]
    return "\n".join(lines) + "\n"
