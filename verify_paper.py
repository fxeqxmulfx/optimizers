import pandas as pd
import numpy as np

df = pd.read_csv("benchmark_results.csv")

THRESHOLD = 0.01
discrepancies = []

def report(section, msg):
    discrepancies.append(f"[{section}] {msg}")

# Algo name mapping
ALGO_MAP = {'ans': 'ANS', 'ansr': 'ANSR', 'ansr_dpnm': 'DPNM', 'de': 'DE', 'shade': 'SHADE', 'zero_gradient': 'ZG'}

###############################################################################
# 1. Easy Table (Table 4)
###############################################################################
easy = df[df['test_set'] == 'easy'].copy()
easy_funcs = easy['function'].unique()

paper_easy = {
    64:   {'ANS': 1856, 'ANSR': 1856, 'DPNM': 3072, 'DE': 2528, 'SHADE': 1856, 'ZG': 1380},
    128:  {'ANS': 2832, 'ANSR': 2848, 'DPNM': 5632, 'DE': 3968, 'SHADE': 2176, 'ZG': 2669},
    256:  {'ANS': 4352, 'ANSR': 4352, 'DPNM': 10112, 'DE': 6272, 'SHADE': 3264, 'ZG': 5340},
    512:  {'ANS': 6976, 'ANSR': 6960, 'DPNM': 16864, 'DE': 10048, 'SHADE': 5376, 'ZG': 10689},
    1024: {'ANS': 12288, 'ANSR': 12288, 'DPNM': 23776, 'DE': 16832, 'SHADE': None, 'ZG': 21350},
}

for dim in [64, 128, 256, 512, 1024]:
    for algo_raw, algo_name in ALGO_MAP.items():
        expected = paper_easy[dim].get(algo_name)
        if expected is None:
            continue
        sub = easy[(easy['dim'] == dim) & (easy['algorithm'] == algo_raw)]
        converged = sub[sub['f_x'] <= THRESHOLD]
        # per-function median nfev
        per_func = converged.groupby('function')['nfev'].median()
        if len(per_func) == 0:
            actual = None
        else:
            actual = per_func.median()
        if actual is None or abs(actual - expected) > 1:
            report("Easy Table", f"{dim}D {algo_name}: paper={expected}, data={actual}")

###############################################################################
# 2. Shubert Table (Table 6)
###############################################################################
shub = df[(df['test_set'] == 'medium_periodic') & (df['function'] == 'shubert')]

paper_shubert_nfev = {
    16: {'ANS': 13800, 'ANSR': 9100, 'DPNM': 120000, 'DE': 16600, 'SHADE': 41100},
    32: {'ANS': 44700, 'ANSR': 28400, 'DPNM': 403000, 'DE': 37600, 'SHADE': 103000},
    64: {'ANS': 187000, 'ANSR': 60300, 'DPNM': None, 'DE': 81800, 'SHADE': 245000},
}
paper_shubert_rate = {
    16: {'ANS': 98, 'ANSR': 100, 'DPNM': 100, 'DE': 100, 'SHADE': 100},
    32: {'ANS': 87, 'ANSR': 100, 'DPNM': 100, 'DE': 100, 'SHADE': 100},
    64: {'ANS': 78, 'ANSR': 100, 'DPNM': 0, 'DE': 100, 'SHADE': 100},
}

for dim in [16, 32, 64]:
    for algo_raw, algo_name in ALGO_MAP.items():
        if algo_name == 'ZG':
            continue
        sub = shub[(shub['dim'] == dim) & (shub['algorithm'] == algo_raw)]
        if len(sub) == 0:
            continue
        converged = sub[sub['f_x'] <= THRESHOLD]
        total = len(sub)
        rate = len(converged) / total * 100
        expected_rate = paper_shubert_rate[dim].get(algo_name)
        if expected_rate is not None and abs(rate - expected_rate) > 0.5:
            report("Shubert Rate", f"{dim}D {algo_name}: paper={expected_rate}%, data={rate:.1f}%")

        expected_nfev = paper_shubert_nfev[dim].get(algo_name)
        if expected_nfev is not None and len(converged) > 0:
            med = converged['nfev'].median()
            # Allow 5% tolerance for rounding (paper uses "k" notation)
            if abs(med - expected_nfev) / expected_nfev > 0.05:
                report("Shubert Nfev", f"{dim}D {algo_name}: paper={expected_nfev}, data={med:.0f}")

###############################################################################
# 3. Megacity Table (Table 7)
###############################################################################
mega = df[(df['test_set'] == 'hard_discrete') & (df['function'] == 'megacity')]

paper_mega_nfev = {
    16: {'ANS': 126000, 'ANSR': None, 'DPNM': None, 'DE': 36800, 'SHADE': 36300},
    32: {'ANS': None, 'ANSR': None, 'DPNM': None, 'DE': 91800, 'SHADE': 115000},
    64: {'ANS': None, 'ANSR': None, 'DPNM': None, 'DE': 217000, 'SHADE': 364000},
}
paper_mega_rate = {
    16: {'ANS': 73, 'ANSR': 0, 'DPNM': 0, 'DE': 80, 'SHADE': 75},
    32: {'ANS': 0, 'ANSR': 0, 'DPNM': 0, 'DE': 80, 'SHADE': 73.5},
    64: {'ANS': 0, 'ANSR': 0, 'DPNM': 0, 'DE': 90.5, 'SHADE': 7},
}

for dim in [16, 32, 64]:
    for algo_raw, algo_name in ALGO_MAP.items():
        if algo_name == 'ZG':
            continue
        sub = mega[(mega['dim'] == dim) & (mega['algorithm'] == algo_raw)]
        if len(sub) == 0:
            continue
        converged = sub[sub['f_x'] <= THRESHOLD]
        total = len(sub)
        rate = len(converged) / total * 100
        expected_rate = paper_mega_rate[dim].get(algo_name)
        if expected_rate is not None and abs(rate - expected_rate) > 0.5:
            report("Megacity Rate", f"{dim}D {algo_name}: paper={expected_rate}%, data={rate:.1f}%")

        expected_nfev = paper_mega_nfev[dim].get(algo_name)
        if expected_nfev is not None and len(converged) > 0:
            med = converged['nfev'].median()
            if abs(med - expected_nfev) / expected_nfev > 0.05:
                report("Megacity Nfev", f"{dim}D {algo_name}: paper={expected_nfev}, data={med:.0f}")

###############################################################################
# 4. Terrain Table (Table 5)
###############################################################################
terrain = df[df['test_set'] == 'medium_terrain']

paper_terrain_nfev = {
    (64, 'hilly'):   {'ANS': 70700, 'ANSR': 70700, 'DPNM': 61800, 'DE': 42400, 'SHADE': 286000},
    (64, 'forest'):  {'ANS': 47300, 'ANSR': 47300, 'DPNM': 47100, 'DE': 29500, 'SHADE': 86000},
    (128, 'hilly'):  {'ANS': 159000, 'ANSR': 111000, 'DPNM': 101000, 'DE': 137000, 'SHADE': None},
    (128, 'forest'): {'ANS': 101000, 'ANSR': 73100, 'DPNM': 69600, 'DE': 85600, 'SHADE': 196000},
    (256, 'hilly'):  {'ANS': 274000, 'ANSR': 334000, 'DPNM': None, 'DE': 202000, 'SHADE': None},
    (256, 'forest'): {'ANS': 175000, 'ANSR': 209000, 'DPNM': None, 'DE': 128000, 'SHADE': 450000},
}
paper_terrain_rate = {
    (64, 'hilly'):   {'ANS': 98, 'ANSR': 97, 'DPNM': 100, 'DE': 72, 'SHADE': 96},
    (64, 'forest'):  {'ANS': 100, 'ANSR': 100, 'DPNM': 100, 'DE': 96, 'SHADE': 92},
    (128, 'hilly'):  {'ANS': 96, 'ANSR': 84, 'DPNM': 98, 'DE': 99, 'SHADE': 0},
    (128, 'forest'): {'ANS': 100, 'ANSR': 100, 'DPNM': 100, 'DE': 100, 'SHADE': 100},
    (256, 'hilly'):  {'ANS': 94, 'ANSR': 98, 'DPNM': 0, 'DE': 95, 'SHADE': 0},
    (256, 'forest'): {'ANS': 100, 'ANSR': 100, 'DPNM': 0, 'DE': 100, 'SHADE': 94},
}

for (dim, func), algos in paper_terrain_nfev.items():
    for algo_raw, algo_name in ALGO_MAP.items():
        if algo_name == 'ZG':
            continue
        sub = terrain[(terrain['dim'] == dim) & (terrain['function'] == func) & (terrain['algorithm'] == algo_raw)]
        if len(sub) == 0:
            continue
        converged = sub[sub['f_x'] <= THRESHOLD]
        total = len(sub)
        rate = len(converged) / total * 100

        expected_rate = paper_terrain_rate.get((dim, func), {}).get(algo_name)
        if expected_rate is not None and abs(rate - expected_rate) > 0.5:
            report("Terrain Rate", f"{dim}D {func} {algo_name}: paper={expected_rate}%, data={rate:.1f}%")

        expected_nfev = algos.get(algo_name)
        if expected_nfev is not None and len(converged) > 0:
            med = converged['nfev'].median()
            if abs(med - expected_nfev) / expected_nfev > 0.05:
                report("Terrain Nfev", f"{dim}D {func} {algo_name}: paper={expected_nfev}, data={med:.0f}")

###############################################################################
# 5. Abstract claim: ANS 78% (155/200) on Shubert 64D
###############################################################################
ans_shub_64 = shub[(shub['dim'] == 64) & (shub['algorithm'] == 'ans')]
conv = ans_shub_64[ans_shub_64['f_x'] <= THRESHOLD]
actual_count = len(conv)
actual_total = len(ans_shub_64)
actual_pct = actual_count / actual_total * 100 if actual_total > 0 else 0
if actual_count != 155 or actual_total != 200:
    report("Abstract 155/200", f"ANS Shubert 64D: paper=155/200 (78%), data={actual_count}/{actual_total} ({actual_pct:.1f}%)")

# Also check ANSR is 100% at 64D
ansr_shub_64 = shub[(shub['dim'] == 64) & (shub['algorithm'] == 'ansr')]
conv_ansr = ansr_shub_64[ansr_shub_64['f_x'] <= THRESHOLD]
ansr_rate = len(conv_ansr) / len(ansr_shub_64) * 100 if len(ansr_shub_64) > 0 else 0
if ansr_rate != 100:
    report("Abstract ANSR 100%", f"ANSR Shubert 64D: paper=100%, data={ansr_rate:.1f}%")

###############################################################################
# 6. SHADE fails on shifted_sphere 1024D, hilly 128D/256D
###############################################################################
for func, dim, label in [('shifted_sphere', 1024, 'shifted_sphere 1024D'),
                          ('hilly', 128, 'hilly 128D'), ('hilly', 256, 'hilly 256D')]:
    ts = 'easy' if func == 'shifted_sphere' else 'medium_terrain'
    sub = df[(df['test_set'] == ts) & (df['dim'] == dim) & (df['algorithm'] == 'shade') & (df['function'] == func)]
    conv = sub[sub['f_x'] <= THRESHOLD]
    rate = len(conv) / len(sub) * 100 if len(sub) > 0 else -1
    if rate != 0:
        report("SHADE fails", f"SHADE {label}: paper=0%, data={rate:.1f}%")

###############################################################################
# 7. ZG fails on discus 128D (99% success rate claimed)
###############################################################################
zg_discus = df[(df['test_set'] == 'easy') & (df['dim'] == 128) & (df['algorithm'] == 'zero_gradient') & (df['function'] == 'discus')]
conv = zg_discus[zg_discus['f_x'] <= THRESHOLD]
rate = len(conv) / len(zg_discus) * 100 if len(zg_discus) > 0 else -1
if abs(rate - 99) > 0.5:
    report("ZG discus 128D", f"paper=99%, data={rate:.1f}%")

###############################################################################
# 8. ANS success rate on Shubert: 98/87/78%
###############################################################################
for dim, expected in [(16, 98), (32, 87), (64, 78)]:
    sub = shub[(shub['dim'] == dim) & (shub['algorithm'] == 'ans')]
    conv = sub[sub['f_x'] <= THRESHOLD]
    rate = len(conv) / len(sub) * 100 if len(sub) > 0 else 0
    if abs(rate - expected) > 0.5:
        report("ANS Shubert rate", f"{dim}D: paper={expected}%, data={rate:.1f}%")

###############################################################################
# 9. Convergence rates for all algos on all test sets
###############################################################################
print("\n=== FULL CONVERGENCE RATE TABLE ===")
for ts in df['test_set'].unique():
    sub_ts = df[df['test_set'] == ts]
    for dim in sorted(sub_ts['dim'].unique()):
        for func in sorted(sub_ts[sub_ts['dim'] == dim]['function'].unique()):
            rates = []
            for algo_raw in ['ans', 'ansr', 'ansr_dpnm', 'de', 'shade', 'zero_gradient']:
                sub = sub_ts[(sub_ts['dim'] == dim) & (sub_ts['function'] == func) & (sub_ts['algorithm'] == algo_raw)]
                if len(sub) == 0:
                    rates.append(f"{ALGO_MAP[algo_raw]}=N/A")
                else:
                    conv = sub[sub['f_x'] <= THRESHOLD]
                    r = len(conv) / len(sub) * 100
                    if r < 100:
                        rates.append(f"{ALGO_MAP[algo_raw]}={r:.1f}%")
            if rates:
                non_full = [r for r in rates if 'N/A' not in r]
                if non_full:
                    print(f"  {ts} {dim}D {func}: {', '.join(non_full)}")

###############################################################################
# 10. Tune results verification
###############################################################################
print("\n=== TUNE RESULTS ===")
for fname in ['tune_results/easy_64d_ans.csv', 'tune_results/medium_periodic_64d_ans.csv', 'tune_results/medium_periodic_64d_ansr.csv']:
    t = pd.read_csv(fname)
    print(f"\n{fname}:")
    print(f"  Columns: {list(t.columns)}")
    print(f"  Best row (lowest mean):")
    best = t.loc[t['mean'].idxmin()]
    print(f"  {best.to_dict()}")

###############################################################################
# DPNM fails at 256D terrain claim
###############################################################################
for func in ['hilly', 'forest']:
    sub = terrain[(terrain['dim'] == 256) & (terrain['algorithm'] == 'ansr_dpnm') & (terrain['function'] == func)]
    conv = sub[sub['f_x'] <= THRESHOLD]
    rate = len(conv) / len(sub) * 100 if len(sub) > 0 else -1
    if rate != 0:
        report("DPNM 256D terrain", f"DPNM {func} 256D: paper=0%, data={rate:.1f}%")

###############################################################################
# ZG 0% on all shubert dims
###############################################################################
for dim in [16, 32, 64]:
    sub = shub[(shub['dim'] == dim) & (shub['algorithm'] == 'zero_gradient')]
    if len(sub) > 0:
        conv = sub[sub['f_x'] <= THRESHOLD]
        rate = len(conv) / len(sub) * 100
        if rate > 0:
            report("ZG Shubert", f"{dim}D: paper=0%, data={rate:.1f}%")

# ZG 0% on terrain
for dim in [64, 128, 256]:
    for func in ['hilly', 'forest']:
        sub = terrain[(terrain['dim'] == dim) & (terrain['algorithm'] == 'zero_gradient') & (terrain['function'] == func)]
        if len(sub) > 0:
            conv = sub[sub['f_x'] <= THRESHOLD]
            rate = len(conv) / len(sub) * 100
            if rate > 0:
                report("ZG Terrain", f"{dim}D {func}: paper=0%, data={rate:.1f}%")

###############################################################################
# REPORT
###############################################################################
print("\n" + "="*60)
if discrepancies:
    print(f"DISCREPANCIES FOUND: {len(discrepancies)}")
    for d in discrepancies:
        print(f"  !! {d}")
else:
    print("NO DISCREPANCIES FOUND - all paper claims match data")
print("="*60)
