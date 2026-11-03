"""Budgeted connected exploration followed by uncertainty/residual allocation."""
import itertools
import math
import random
from .engine import valid_blocks
from .statistics import Model


def select_pairs(roster, records, mode, batch_size, round_number, seed, caps=None):
    pairs = list(itertools.combinations(sorted(roster), 2))
    counts = {p: len(records.get(p, [])) for p in pairs}
    rng = random.Random(f"schedule-v1:{seed}:{round_number}")
    # Randomized ties prevent identity/order bias; the ring gives a connected
    # graph in O(N) matches even when the budget cannot cover every edge.
    order = list(sorted(roster))
    random.Random(f"ring-v1:{seed}").shuffle(order)
    ring = {tuple(sorted((order[i], order[(i + 1) % len(order)]))) for i in range(len(order))}
    blocks = valid_blocks(records)
    model = Model(sorted(roster), blocks) if mode == "adaptive" else None
    chosen = []
    for _ in range(batch_size):
        available = [p for p in pairs if caps is None or counts[p] < caps[p]]
        if not available:
            break
        unvisited_ring = [p for p in available if p in ring and counts[p] == 0]
        if unvisited_ring:
            candidate = rng.choice(unvisited_ring)
        elif mode == "uniform" or (round_number + len(chosen)) % 4 == 0:
            minimum = min(counts[p] for p in available)
            candidate = rng.choice([p for p in available if counts[p] == minimum])
        else:
            def priority(pair):
                data = blocks.get(pair, [])
                count = counts[pair]
                if not data:
                    # Explore unseen edges before trusting model extrapolation.
                    return 10 / math.sqrt(count + 1)
                mean = sum((x + y) / 2 for x, y in data) / len(data)
                residual = abs(mean - model.predict(*pair))
                # Estimate variability of the PAIRED outcome, not an individual
                # Bernoulli game: swapped positions can be strongly correlated.
                # Two prior observations keep a short all-equal sample exploratory.
                variance = (sum(((x + y) / 2 - mean)**2 for x, y in data) + .25) / (len(data) + 2)
                # Heuristic marginal reduction of variance/n plus squared bias;
                # not the exact MSE of the jointly fitted shrinkage estimator.
                # Unlike a static residual weight, this stops spending heavily
                # on a predictable counter once its local estimate is precise.
                noise_gain = variance / ((count + 1) * (count + 2))
                bias_gain = 64 * residual**2 * (1 / (count + 8)**2 - 1 / (count + 9)**2)
                return noise_gain + bias_gain
            rng.shuffle(available)
            candidate = max(available, key=priority)
        chosen.append(candidate)
        counts[candidate] += 1
    return chosen
