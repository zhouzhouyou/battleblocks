"""Paired-seed uncertainty and a regularized Bradley-Terry expected-score model."""
import math


def sigmoid(x):
    return 1 / (1 + math.exp(-max(-35, min(35, x))))


def confidence_sequence(values, alpha=0.05):
    """Beta(1/2,1/2) mixture CS for independent [0,1] seed-block scores.

    Fractional observations are valid by the Bernoulli convex upper bound.
    Ville's inequality allows adaptive pair selection and arbitrary stopping.
    The caller divides alpha among all reported pair/seat sequences.
    """
    n, s = len(values), sum(values)
    if not n:
        return [0.0, 1.0]
    log_beta = (math.lgamma(s + .5) + math.lgamma(n - s + .5)
                - math.lgamma(n + 1) - 2 * math.lgamma(.5))
    threshold = math.log(1 / alpha)
    def log_e(p):
        if p <= 0:
            return log_beta if s == 0 else math.inf
        if p >= 1:
            return log_beta if s == n else math.inf
        return log_beta - s * math.log(p) - (n - s) * math.log1p(-p)
    mean = s / n
    lo, hi = 0.0, mean
    for _ in range(55):
        mid = (lo + hi) / 2
        if log_e(mid) > threshold:
            lo = mid
        else:
            hi = mid
    lower = hi if s else 0.0
    lo, hi = mean, 1.0
    for _ in range(55):
        mid = (lo + hi) / 2
        if log_e(mid) > threshold:
            hi = mid
        else:
            lo = mid
    return [lower, lo if s != n else 1.0]


def solve(matrix, vector):
    """Small positive definite systems; partial pivoting, no external runtime."""
    n = len(vector)
    a = [list(row) + [v] for row, v in zip(matrix, vector)]
    for i in range(n):
        pivot = max(range(i, n), key=lambda j: abs(a[j][i]))
        a[i], a[pivot] = a[pivot], a[i]
        if abs(a[i][i]) < 1e-12:
            raise ValueError("singular rating system")
        div = a[i][i]
        a[i] = [v / div for v in a[i]]
        for j in range(i + 1, n):
            factor = a[j][i]
            for k in range(i + 1, n + 1):
                a[j][k] -= factor * a[i][k]
    out = [0.0] * n
    for i in reversed(range(n)):
        out[i] = a[i][n] - sum(a[i][j] * out[j] for j in range(i + 1, n))
    return out


def inverse(matrix):
    """Invert once in O(N^3), rather than N separate O(N^3) solves."""
    n = len(matrix)
    a = [list(row) + [float(i == j) for j in range(n)] for i, row in enumerate(matrix)]
    for i in range(n):
        pivot = max(range(i, n), key=lambda j: abs(a[j][i]))
        a[i], a[pivot] = a[pivot], a[i]
        div = a[i][i]
        if abs(div) < 1e-12:
            raise ValueError("singular rating covariance")
        a[i] = [v / div for v in a[i]]
        for j in range(n):
            if j != i:
                factor = a[j][i]
                a[j] = [v - factor * w for v, w in zip(a[j], a[i])]
    return [row[n:] for row in a]


class Model:
    """Ridge BT with a first-seat intercept. Draws contribute half a point.

    Rating intervals are conditional Laplace approximations, NOT confidence
    sequences. All balance flags instead use the empirical paired-seed CS.
    """
    def __init__(self, roster, pairs):
        self.roster = list(roster)
        index = {h: i for i, h in enumerate(roster)}
        n = len(roster)
        observations = []
        for (a, b), blocks in pairs.items():
            if not blocks:
                continue
            for seat in (0, 1):
                observations.append((index[a], index[b], 1 if seat == 0 else -1,
                                     len(blocks), sum(block[seat] for block in blocks)))
        theta = [0.0] * (n + 1)
        def derivatives(t):
            # The gauge (mean ability) is fixed by a symmetric Gaussian prior.
            g = [-.5 * x for x in t]
            h = [[.5 if i == j else 0.0 for j in range(n + 1)] for i in range(n + 1)]
            for a, b, seat, count, wins in observations:
                p = sigmoid(t[a] - t[b] + seat * t[n])
                # Each block has two correlated observations: half likelihood
                # tempers information; empirical CS carries formal uncertainty.
                err, v = (wins - count * p) / 2, count * p * (1 - p) / 2
                x = [(a, 1), (b, -1), (n, seat)]
                for i, xi in x:
                    g[i] += err * xi
                    for j, xj in x:
                        h[i][j] += v * xi * xj
            return g, h
        for _ in range(60):
            gradient, hessian = derivatives(theta)
            step = solve(hessian, gradient)
            # Bounded Newton step also handles complete separation.
            damping = min(1.0, 1.0 / max(1.0, max(abs(v) for v in step)))
            theta = [v + damping * d for v, d in zip(theta, step)]
            if max(abs(v) for v in step) < 1e-7:
                break
        _, hessian = derivatives(theta)
        self.theta, self.index = theta, index
        self.cov = inverse(hessian)

    def predict(self, a, b):
        difference = self.theta[self.index[a]] - self.theta[self.index[b]]
        return (sigmoid(difference + self.theta[-1]) + sigmoid(difference - self.theta[-1])) / 2

    def rating(self, hero):
        i, n = self.index[hero], len(self.roster)
        scale = 400 / math.log(10)
        mean = sum(self.theta[:-1]) / n
        variance = self.cov[i][i] - 2 * sum(self.cov[i][:n]) / n + sum(sum(r[:n]) for r in self.cov[:n]) / n**2
        value = 1500 + scale * (self.theta[i] - mean)
        radius = 1.96 * scale * math.sqrt(max(0, variance))
        return value, [value - radius, value + radius]


def estimate(model, pair, blocks):
    # Fixed shrinkage, identical for uniform/adaptive benchmark arms.
    n = len(blocks)
    return (sum((x + y) / 2 for x, y in blocks) + 8 * model.predict(*pair)) / (n + 8)
