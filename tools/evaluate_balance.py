"""Budgeted balance evaluation using the canonical headless executable."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import sys
import subprocess
import time

from balance.engine import Cache, Engine, pair_key, seed_for
from balance.report import make_report, markdown
from balance.sampling import select_pairs

ROOT = Path(__file__).resolve().parents[1]


def atomic_text(path, text):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + f".{os.getpid()}.tmp")
    try:
        temporary.write_text(text, encoding="utf-8")
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


def evaluate(engine, cache, roster, config, progress=True):
    pairs = list(itertools.combinations(sorted(roster), 2))
    keys = {p: pair_key(engine.manifest, p, config["seed"], config["level"], config["max_steps"]) for p in pairs}
    records = {p: cache.load(keys[p]) for p in pairs}
    reused = 2 * sum(map(len, records.values()))
    if reused > config["budget"]:
        raise ValueError(f"cache already contains {reused} compatible games; budget must be at least that large (or choose another seed/cache)")
    total, round_number = reused, 0
    started = time.perf_counter()
    while total < config["budget"]:
        count = min(config["batch_size"], (config["budget"] - total) // 2)
        chosen = select_pairs(roster, records, config["sampler"], count, round_number, config["seed"])
        indices = {p: len(records[p]) for p in pairs}
        jobs, samples = [], []
        for pair in chosen:
            index = indices[pair]
            indices[pair] += 1
            jobs.append((pair, seed_for(config["seed"], pair, index), config["level"], config["max_steps"]))
            samples.append(index)
        outcomes = engine.blocks(jobs)
        for pair, index, record in zip(chosen, samples, outcomes):
            cache.put(keys[pair], index, record)
            records[pair].append(record)
        total += 2 * count
        round_number += count
        if progress:
            print(f"games={total}/{config['budget']} new={total-reused} elapsed={time.perf_counter()-started:.1f}s", file=sys.stderr, flush=True)
    report = make_report(engine.manifest, roster, records, config, total - reused, reused)
    sources = sorted((ROOT / "tools/balance").glob("*.py")) + [Path(__file__)]
    report["analysis_fingerprint"] = hashlib.sha256(b"\0".join(p.read_bytes().replace(b"\r\n", b"\n") for p in sources)).hexdigest()
    report["elapsed_seconds"] = time.perf_counter() - started
    return report, records


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release" / ("battleblocks.exe" if os.name == "nt" else "battleblocks"))
    parser.add_argument("--roster", default="8,7,9,10,11", help="comma-separated stable ids, or all")
    parser.add_argument("--budget", type=int, default=400, help="maximum total compatible games, including cache hits; must be even")
    parser.add_argument("--seed", type=int, default=1200000, help="experiment seed; derived independent pair seeds are reproducible")
    parser.add_argument("--level", type=int, choices=(0, 1, 2), default=1)
    parser.add_argument("--max-steps", type=int, default=20000)
    parser.add_argument("--sampler", choices=("uniform", "adaptive"), default="adaptive")
    parser.add_argument("--batch-size", type=int, default=8, help="seed pairs between model updates (independent of worker count)")
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--cache", type=Path, default=ROOT / "analysis/evaluation-cache.sqlite")
    parser.add_argument("--out", type=Path, default=ROOT / "analysis/evaluation")
    parser.add_argument("--lower", type=float, default=.45)
    parser.add_argument("--upper", type=float, default=.55)
    parser.add_argument("--extreme", type=float, default=.75)
    parser.add_argument("--width", type=float, default=.20, help="maximum CS width to label a pair verified")
    args = parser.parse_args(argv)
    if args.budget < 2 or args.budget % 2 or min(args.max_steps, args.workers, args.batch_size) <= 0:
        parser.error("budget must be positive/even; steps, workers and batch size must be positive")
    if not 0 <= args.seed < 2**64 or args.max_steps >= 2**64:
        parser.error("seed and max-steps must fit u64")
    if not 0 <= args.lower < .5 < args.upper <= 1 or not .5 < args.extreme < 1 or not 0 < args.width < 1:
        parser.error("invalid balance thresholds or interval width")
    engine = Engine(args.binary, args.workers)
    ids = {h["id"] for h in engine.manifest["heroes"]}
    roster = sorted(ids) if args.roster == "all" else [int(i) for i in args.roster.split(",")]
    if len(roster) < 2 or len(set(roster)) != len(roster) or not set(roster) <= ids:
        parser.error("roster must contain at least two distinct valid hero ids")
    # Never let an output overwrite the executable or the database.
    outputs = [args.out / name for name in ("report.json", "report.md", "matches.jsonl")]
    protected = {args.binary.resolve(), args.cache.resolve()}
    if len(protected) != 2 or any(p.resolve() in protected for p in outputs):
        parser.error("binary, cache and output files must be distinct")
    config = {k: getattr(args, k) for k in ("budget", "seed", "level", "max_steps", "sampler", "batch_size", "lower", "upper", "extreme", "width")}
    cache = Cache(args.cache)
    try:
        report, records = evaluate(engine, cache, roster, config)
    finally:
        cache.close()
    atomic_text(outputs[0], json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    atomic_text(outputs[1], markdown(report))
    atomic_text(outputs[2], "".join(json.dumps({"pair": pair, "sample": i, "pair_key": pair_key(engine.manifest, pair, args.seed, args.level, args.max_steps), "games": block}) + "\n"
                                    for pair, rows in sorted(records.items()) for i, block in enumerate(rows)))
    print(f"report={outputs[0]} games={report['games']} new={report['new_games']} reused={report['reused_games']}")
    return 2 if report["aborted"] else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        raise SystemExit(f"EVALUATION_ERROR: {error}")
