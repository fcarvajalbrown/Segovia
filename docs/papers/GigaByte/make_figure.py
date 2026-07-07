import os
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "template", "fig_latency_memory")

DEADLINE_MS = 300.0

lat_labels = ["Mean", "p99", "Max"]
segovia_lat = [179.2, 277.0, 334.5]
si_lat = [205.3, 355.0, 932.0]

segovia_rss = 0.21
si_rss = 0.41

seg_color = "#1f77b4"
si_color = "#d62728"

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.2, 3.1), gridspec_kw={"width_ratios": [2.4, 1.0]})

x = np.arange(len(lat_labels))
w = 0.38
ax1.bar(x - w / 2, segovia_lat, w, label="Segovia", color=seg_color)
ax1.bar(x + w / 2, si_lat, w, label="SpikeInterface online", color=si_color)
ax1.axhline(DEADLINE_MS, ls="--", lw=1.2, color="0.35")
ax1.text(-0.45, DEADLINE_MS + 12, "300 ms deadline", ha="left", va="bottom", fontsize=8, color="0.35")
ax1.set_xticks(x)
ax1.set_xticklabels(lat_labels)
ax1.set_ylabel("Per-chunk latency (ms)")
ax1.set_title("(a) Latency, 300 ms budget", fontsize=10)
ax1.legend(fontsize=8, frameon=False, loc="upper left")
for xi, v in zip(x - w / 2, segovia_lat):
    ax1.text(xi, v + 8, f"{v:.0f}", ha="center", va="bottom", fontsize=7, color=seg_color)
for xi, v in zip(x + w / 2, si_lat):
    ax1.text(xi, v + 8, f"{v:.0f}", ha="center", va="bottom", fontsize=7, color=si_color)
ax1.set_ylim(0, 1000)

xr = np.arange(1)
ax2.bar(xr - w / 2, [segovia_rss], w, color=seg_color)
ax2.bar(xr + w / 2, [si_rss], w, color=si_color)
ax2.set_xticks([xr[0] - w / 2, xr[0] + w / 2])
ax2.set_xticklabels(["Segovia", "SI"], fontsize=8)
ax2.set_ylabel("Peak resident memory (GB)")
ax2.set_title("(b) Memory", fontsize=10)
ax2.text(xr[0] - w / 2, segovia_rss + 0.01, f"{segovia_rss:.2f}", ha="center", va="bottom", fontsize=8, color=seg_color)
ax2.text(xr[0] + w / 2, si_rss + 0.01, f"{si_rss:.2f}", ha="center", va="bottom", fontsize=8, color=si_color)
ax2.set_ylim(0, 0.5)

fig.tight_layout()
fig.savefig(OUT + ".pdf")
fig.savefig(OUT + ".png", dpi=200)
print("wrote", OUT + ".pdf", "and", OUT + ".png")
