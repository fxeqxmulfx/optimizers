#!/usr/bin/env python3
"""Statistical analysis of benchmark results for scientific paper."""

import sys
import os
import pandas as pd
import numpy as np
from scipy import stats
from itertools import combinations


def load_data(path="benchmark_results.csv"):
    df = pd.read_csv(path)
    df["converged"] = df["f_x"] <= 0.01
    return df


def summary_table(df):
    """Median ± IQR of nfev per algorithm per function, inf if not all seeds converged."""
    print("\n=== Summary: median nfev (IQR) ===\n")
    for (test_set, dim), group in df.groupby(["test_set", "dim"]):
        print(f"--- {test_set} {dim}D ---")
        algos = group["algorithm"].unique()
        funcs = group["function"].unique()
        seed_count = group.groupby(["algorithm", "function"])["seed"].count().iloc[0]

        header = f"{'algorithm':<16}" + "".join(f"{f:>24}" for f in funcs) + f"{'mean':>24}"
        print(header)
        print("-" * len(header))

        for algo in sorted(algos):
            ag = group[group["algorithm"] == algo]
            parts = []
            means = []
            for func in funcs:
                fg = ag[ag["function"] == func]
                converged = fg[fg["converged"]]
                if len(converged) == seed_count:
                    med = converged["nfev"].median()
                    q1 = converged["nfev"].quantile(0.25)
                    q3 = converged["nfev"].quantile(0.75)
                    parts.append(f"{med:.0f} ({q1:.0f}-{q3:.0f})")
                    means.append(med)
                else:
                    parts.append("inf")
                    means.append(float("inf"))
            if all(m != float("inf") for m in means):
                mean_str = f"{np.mean(means):.0f}"
            else:
                mean_str = "inf"
            print(f"{algo:<16}" + "".join(f"{p:>24}" for p in parts) + f"{mean_str:>24}")
        print()


def convergence_rate(df):
    """Success rate per algorithm per function."""
    print("\n=== Convergence Rate (%) ===\n")
    for (test_set, dim), group in df.groupby(["test_set", "dim"]):
        print(f"--- {test_set} {dim}D ---")
        pivot = group.groupby(["algorithm", "function"])["converged"].mean() * 100
        table = pivot.unstack(fill_value=0)
        print(table.round(1).to_string())
        print()


def vargha_delaney_a(x, y):
    """Vargha-Delaney A effect size measure.
    A = P(X > Y) + 0.5 * P(X == Y)
    A = 0.5 means no effect, A > 0.5 means X tends to be larger.
    Returns A from perspective of x (A > 0.5 means y is better / smaller).
    """
    nx, ny = len(x), len(y)
    r = stats.rankdata(np.concatenate([x, y]))
    r1 = r[:nx].sum()
    a = (r1 / nx - (nx + 1) / 2) / ny
    return a


def effect_size_label(a):
    """Interpret Vargha-Delaney A."""
    d = abs(a - 0.5)
    if d < 0.06:
        return "negligible"
    elif d < 0.14:
        return "small"
    elif d < 0.21:
        return "medium"
    else:
        return "large"


def wilcoxon_pairwise(df):
    """Wilcoxon rank-sum test with Vargha-Delaney effect size between all algorithm pairs."""
    print("\n=== Wilcoxon Rank-Sum Tests + Effect Size ===\n")
    for (test_set, dim), group in df.groupby(["test_set", "dim"]):
        print(f"--- {test_set} {dim}D ---")
        algos = sorted(group["algorithm"].unique())
        funcs = sorted(group["function"].unique())
        seed_count = group.groupby(["algorithm", "function"])["seed"].count().iloc[0]

        for func in funcs:
            fg = group[group["function"] == func]
            print(f"\n  {func}:")
            for a1, a2 in combinations(algos, 2):
                d1 = fg[fg["algorithm"] == a1]
                d2 = fg[fg["algorithm"] == a2]
                c1 = d1[d1["converged"]]
                c2 = d2[d2["converged"]]
                if len(c1) == seed_count and len(c2) == seed_count:
                    stat, p = stats.mannwhitneyu(
                        c1["nfev"].values, c2["nfev"].values, alternative="two-sided"
                    )
                    a = vargha_delaney_a(c1["nfev"].values, c2["nfev"].values)
                    sig = "***" if p < 0.001 else "**" if p < 0.01 else "*" if p < 0.05 else ""
                    med1 = c1["nfev"].median()
                    med2 = c2["nfev"].median()
                    winner = a1 if med1 < med2 else a2
                    elabel = effect_size_label(a)
                    print(
                        f"    {a1:>16} vs {a2:<16} p={p:.4e} {sig:>4}"
                        f"  A={a:.3f} ({elabel:<10})  winner={winner}"
                    )
                else:
                    r1 = f"{len(c1)}/{seed_count}"
                    r2 = f"{len(c2)}/{seed_count}"
                    print(f"    {a1:>16} vs {a2:<16} skip (converged: {r1}, {r2})")
        print()


def holm_correction(p_values):
    """Holm-Bonferroni step-down correction for multiple comparisons.
    Returns adjusted p-values.
    """
    n = len(p_values)
    sorted_idx = np.argsort(p_values)
    adjusted = np.zeros(n)
    for rank, idx in enumerate(sorted_idx):
        adjusted[idx] = min(1.0, p_values[idx] * (n - rank))
    # Enforce monotonicity
    cummax = 0.0
    for idx in sorted_idx:
        cummax = max(cummax, adjusted[idx])
        adjusted[idx] = cummax
    return adjusted


def friedman_ranking(df):
    """Friedman test with Holm post-hoc for ranking algorithms across functions."""
    print("\n=== Friedman Test + Holm Post-Hoc ===\n")
    for (test_set, dim), group in df.groupby(["test_set", "dim"]):
        algos = sorted(group["algorithm"].unique())
        funcs = sorted(group["function"].unique())
        seed_count = group.groupby(["algorithm", "function"])["seed"].count().iloc[0]

        matrix = []
        valid_funcs = []
        for func in funcs:
            fg = group[group["function"] == func]
            row = []
            all_converged = True
            for algo in algos:
                ag = fg[fg["algorithm"] == algo]
                converged = ag[ag["converged"]]
                if len(converged) == seed_count:
                    row.append(converged["nfev"].median())
                else:
                    all_converged = False
                    break
            if all_converged:
                matrix.append(row)
                valid_funcs.append(func)

        if len(matrix) < 2:
            print(f"--- {test_set} {dim}D --- skipped (need >=2 functions where all algos converge)")
            continue

        matrix = np.array(matrix)
        stat, p = stats.friedmanchisquare(*[matrix[:, i] for i in range(matrix.shape[1])])
        print(f"--- {test_set} {dim}D ---")
        print(f"  Functions used: {valid_funcs}")
        print(f"  Friedman chi2={stat:.2f}, p={p:.4e}")

        # Average ranks
        ranks = np.zeros_like(matrix, dtype=float)
        for i in range(len(matrix)):
            ranks[i] = stats.rankdata(matrix[i])
        avg_ranks = ranks.mean(axis=0)
        ranking = sorted(zip(algos, avg_ranks), key=lambda x: x[1])
        print("  Ranking:")
        for i, (algo, rank) in enumerate(ranking, 1):
            print(f"    {i}. {algo:<16} avg_rank={rank:.2f}")

        # Holm post-hoc: compare all pairs using Nemenyi-like z-test
        if p < 0.05 and len(algos) > 2:
            k = len(algos)
            n = len(valid_funcs)
            pairs = list(combinations(range(k), 2))
            p_values = []
            pair_names = []
            for i, j in pairs:
                diff = abs(avg_ranks[i] - avg_ranks[j])
                se = np.sqrt(k * (k + 1) / (6.0 * n))
                z = diff / se
                pval = 2.0 * (1.0 - stats.norm.cdf(z))
                p_values.append(pval)
                pair_names.append((algos[i], algos[j]))

            adjusted = holm_correction(np.array(p_values))
            print("\n  Holm post-hoc pairwise comparisons:")
            sorted_pairs = sorted(zip(pair_names, p_values, adjusted), key=lambda x: x[1])
            for (a1, a2), raw_p, adj_p in sorted_pairs:
                sig = "***" if adj_p < 0.001 else "**" if adj_p < 0.01 else "*" if adj_p < 0.05 else ""
                print(f"    {a1:>16} vs {a2:<16} raw_p={raw_p:.4e}  adj_p={adj_p:.4e} {sig}")
        print()


def latex_tables(df, outdir="latex_tables"):
    """Generate LaTeX tables for paper."""
    os.makedirs(outdir, exist_ok=True)

    for (test_set, dim), group in df.groupby(["test_set", "dim"]):
        algos = sorted(group["algorithm"].unique())
        funcs = sorted(group["function"].unique())
        seed_count = group.groupby(["algorithm", "function"])["seed"].count().iloc[0]

        rows = []
        for algo in algos:
            ag = group[group["algorithm"] == algo]
            row = {"algorithm": algo}
            medians = []
            for func in funcs:
                fg = ag[ag["function"] == func]
                converged = fg[fg["converged"]]
                if len(converged) == seed_count:
                    med = converged["nfev"].median()
                    iqr = converged["nfev"].quantile(0.75) - converged["nfev"].quantile(0.25)
                    row[func] = f"${med:.0f} \\pm {iqr:.0f}$"
                    medians.append(med)
                else:
                    rate = len(converged) / seed_count * 100
                    row[func] = f"$\\infty$ ({rate:.0f}\\%)"
                    medians.append(float("inf"))
            if all(m != float("inf") for m in medians):
                row["mean"] = f"${np.mean(medians):.0f}$"
            else:
                row["mean"] = "$\\infty$"
            rows.append(row)

        # Sort by mean (finite first)
        rows.sort(key=lambda r: float("inf") if "infty" in r["mean"] else float(r["mean"].strip("$")))

        cols = ["algorithm"] + funcs + ["mean"]
        fname = f"{outdir}/{test_set}_{dim}d.tex"
        with open(fname, "w") as f:
            col_spec = "l" + "r" * (len(funcs) + 1)
            f.write(f"\\begin{{tabular}}{{{col_spec}}}\n")
            f.write("\\toprule\n")
            header = " & ".join(["Algorithm"] + [fn.replace("_", "\\_") for fn in funcs] + ["Mean"])
            f.write(header + " \\\\\n")
            f.write("\\midrule\n")
            for row in rows:
                vals = [row.get(c, "") for c in cols]
                vals[0] = vals[0].replace("_", "\\_")
                f.write(" & ".join(vals) + " \\\\\n")
            f.write("\\bottomrule\n")
            f.write("\\end{tabular}\n")

        print(f"  Wrote {fname}")


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "benchmark_results.csv"
    df = load_data(path)
    print(f"Loaded {len(df)} rows from {path}")
    print(f"Test sets: {df['test_set'].unique()}")
    print(f"Algorithms: {df['algorithm'].unique()}")

    summary_table(df)
    convergence_rate(df)
    wilcoxon_pairwise(df)
    friedman_ranking(df)
    latex_tables(df)


if __name__ == "__main__":
    main()
