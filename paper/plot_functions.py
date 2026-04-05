import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib import cm
from mpl_toolkits.mplot3d import Axes3D

PI = np.pi

def scale(v, in_min, in_max, out_min, out_max):
    return (v - in_min) / (in_max - in_min) * (out_max - out_min) + out_min

def sphere(x, y):
    r = x**2 + y**2
    return scale(r, 0.0, 50.0, 0.0, 1.0)

def shifted_sphere(x, y):
    xp, yp = x + PI, y + PI
    r = xp**2 + yp**2
    return scale(r, 0.0, 345.402914946, 0.0, 1.0)

def ellipsoid(x, y):
    r = x**2 + 1_000_000.0 * y**2
    return scale(r, 0.0, 25_000_025.0, 0.0, 1.0)

def discus(x, y):
    r = 1_000_000.0 * x**2 + y**2
    return scale(r, 0.0, 25_000_025.0, 0.0, 1.0)

def different_powers(x, y):
    r = x**2 + y**6
    return scale(r, 0.0, 15_650.0, 0.0, 1.0)

def rosenbrock(x, y):
    r = 100.0 * (x**2 - y)**2 + (x - 1.0)**2
    return scale(r, 0.0, 90_036.0, 0.0, 1.0)

def hilly(x, y):
    r = (20.0 + x**2 + y**2
         - 10.0 * np.cos(2*PI*x) - 10.0 * np.cos(2*PI*y)
         - 30.0 * np.exp(-((x-1.0)**2 + y**2) / 0.1)
         + 200.0 * np.exp(-((x+PI*0.47)**2 + (y-PI*0.2)**2) / 0.1)
         + 100.0 * np.exp(-((x-0.5)**2 + (y+0.5)**2) / 0.01)
         - 60.0 * np.exp(-((x-1.33)**2 + (y-2.0)**2) / 0.02)
         - 40.0 * np.exp(-((x+1.3)**2 + (y+0.2)**2) / 0.5)
         + 60.0 * np.exp(-((x-1.5)**2 + (y+1.5)**2) / 0.1))
    r = -r
    return scale(r, -229.91931214214105, 39.701816104859866, 0.0, 1.0)

def forest(x, y):
    a = np.sin(np.sqrt(np.abs(x - 1.13) + np.abs(y - 2.0)))
    b = np.cos(np.sqrt(np.abs(np.sin(x))) + np.sqrt(np.abs(np.sin(y - 2.0))))
    f = (a + b
         + 1.01 * np.exp(-(((x+42.0)**2 + (y+43.5)**2) / 0.9))
         + 1.0  * np.exp(-(((x+40.2)**2 + (y+46.0)**2) / 0.3)))
    r = f**4 - 0.3 * np.exp(-(((x+42.3)**2 + (y+46.0)**2) / 0.02))
    r = -r
    return scale(r, -1.8779867959790217, 0.26489289358875895, 0.0, 1.0)

def shubert(x, y):
    sx = sum(i * np.cos((i+1)*x + i) for i in range(1, 6))
    sy = sum(i * np.cos((i+1)*y + i) for i in range(1, 6))
    r = sx * sy
    return scale(r, -186.7309, 210.0, 0.0, 1.0)

def megacity(x, y):
    a = np.sin(np.sqrt(np.abs(x - 1.13) + np.abs(y - 2.0)))
    b = np.cos(np.sqrt(np.abs(np.sin(x))) + np.sqrt(np.abs(np.sin(y - 2.0))))
    f = a + b
    term1 = np.floor(f**4)
    term2 = np.floor(2.0 * np.exp(-(((x+9.5)**2 + (y+7.5)**2) / 0.4)))
    r = -(term1 - term2)
    return scale(r, -12.0, 2.0, 0.0, 1.0)

FUNCTIONS = [
    ("sphere",           sphere,         (-5, 5),       (-5, 5),       "Sphere"),
    ("shifted_sphere",   shifted_sphere,  (-10, 10),     (-10, 10),     "Shifted Sphere"),
    ("ellipsoid",        ellipsoid,       (-5, 5),       (-5, 5),       "Ellipsoid"),
    ("discus",           discus,          (-5, 5),       (-5, 5),       "Discus"),
    ("different_powers", different_powers,(-5, 5),       (-5, 5),       "Different Powers"),
    ("rosenbrock",       rosenbrock,      (-5, 5),       (-5, 5),       "Rosenbrock"),
    ("hilly",            hilly,           (-3, 3),       (-3, 3),       "Hilly"),
    ("forest",           forest,          (-43.5, -39),  (-47.35, -40), "Forest"),
    ("shubert",          shubert,         (-10, 10),     (-10, 10),     "Shubert"),
    ("megacity",         megacity,        (-10, -2),     (-10.5, 10),   "Megacity"),
]

N = 200

for name, func, (x0, x1), (y0, y1), title in FUNCTIONS:
    x = np.linspace(x0, x1, N)
    y = np.linspace(y0, y1, N)
    X, Y = np.meshgrid(x, y)
    Z = func(X, Y)
    Z = np.clip(Z, 0.0, 1.0)

    fig = plt.figure(figsize=(4, 3))
    ax = fig.add_subplot(111, projection='3d')
    ax.plot_surface(X, Y, Z, cmap='viridis', linewidth=0, antialiased=True, rcount=80, ccount=80)
    ax.set_title(title, fontsize=11, pad=4)
    ax.set_xlabel('$x_1$', fontsize=8, labelpad=1)
    ax.set_ylabel('$x_2$', fontsize=8, labelpad=1)
    ax.set_zlabel('$f$', fontsize=8, labelpad=1)
    ax.tick_params(labelsize=6, pad=0)
    ax.set_zlim(0, 1)
    ax.view_init(elev=30, azim=-60)
    fig.tight_layout(pad=0.5)
    out = f"paper/figures/{name}.pdf"
    fig.savefig(out, bbox_inches='tight', dpi=150)
    plt.close(fig)
    print(f"Saved {out}")

print("Done.")
