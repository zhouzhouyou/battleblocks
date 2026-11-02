"""Versioned match storage and subprocess adapter; never implements game rules."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess


def seed_for(root, pair, index):
    raw = f"battleblocks-seed-v1:{root}:{pair[0]}:{pair[1]}:{index}".encode()
    return int.from_bytes(hashlib.sha256(raw).digest()[:8], "big") or 1


def pair_key(manifest, pair, root, level, max_steps):
    heroes = {h["id"]: h["fingerprint"] for h in manifest["heroes"]}
    value = [1, manifest["shared"], [(i, heroes[i]) for i in pair], root, level, max_steps]
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def scores(record):
    if any(r["termination"] == "aborted" for r in record):
        return None
    first, second = record
    return (0.5 if first["winner"] is None else float(first["winner"] == 0),
            0.5 if second["winner"] is None else float(second["winner"] == 1))


def valid_blocks(records):
    return {pair: [value for record in rows if (value := scores(record)) is not None]
            for pair, rows in records.items()}


class Engine:
    def __init__(self, binary, workers=1):
        self.binary = str(Path(binary).resolve())
        self.workers = workers
        result = subprocess.run([self.binary, "--headless", "--evaluation-manifest"],
                                text=True, encoding="utf-8", capture_output=True, check=True, timeout=30)
        self.manifest = json.loads(result.stdout)
        if self.manifest["schema"] != 1:
            raise ValueError("unsupported executable manifest")

    def block(self, pair, seed, level, max_steps):
        jobs = [(pair[0], pair[1], seed, level, max_steps),
                (pair[1], pair[0], seed, level, max_steps)]
        text = "".join(" ".join(map(str, job)) + "\n" for job in jobs)
        run = subprocess.run([self.binary, "--headless", "--evaluate-jobs"], input=text,
                             text=True, encoding="utf-8", capture_output=True, check=True, timeout=180)
        records = [json.loads(line) for line in run.stdout.splitlines()]
        if len(records) != 2:
            raise ValueError("engine did not return exactly two games")
        for job, row in zip(jobs, records):
            if tuple(row[k] for k in ("hero", "enemy", "seed", "level", "max_steps")) != job:
                raise ValueError("engine returned a different job")
            if row["termination"] not in ("win", "draw", "aborted"):
                raise ValueError("unknown termination")
            if row["termination"] == "win" and row["winner"] not in (0, 1):
                raise ValueError("invalid winner")
            if row["termination"] != "win" and row["winner"] is not None:
                raise ValueError("nonterminal winner")
        return records

    def blocks(self, jobs):
        # One process per independent seed pair; completion order does not alter
        # sampling decisions, RNG streams or the order records enter the cache.
        with ThreadPoolExecutor(max_workers=self.workers) as pool:
            return list(pool.map(lambda args: self.block(*args), jobs))


class Cache:
    def __init__(self, path):
        Path(path).parent.mkdir(parents=True, exist_ok=True)
        self.db = sqlite3.connect(path, timeout=5)
        self.db.execute("CREATE TABLE IF NOT EXISTS blocks (pair_key TEXT, sample INTEGER, "
                        "records TEXT NOT NULL, PRIMARY KEY(pair_key, sample))")

    def load(self, key):
        rows = self.db.execute("SELECT sample, records FROM blocks WHERE pair_key=? ORDER BY sample", (key,)).fetchall()
        if [r[0] for r in rows] != list(range(len(rows))):
            raise ValueError("cache has missing sample indices")
        return [json.loads(r[1]) for r in rows]

    def put(self, key, index, record):
        value = json.dumps(record, sort_keys=True)
        with self.db:
            self.db.execute("INSERT OR IGNORE INTO blocks VALUES (?, ?, ?)", (key, index, value))
            stored = self.db.execute("SELECT records FROM blocks WHERE pair_key=? AND sample=?", (key, index)).fetchone()[0]
            if stored != value:
                raise ValueError("same version and seed produced different outcomes")

    def close(self):
        self.db.close()
