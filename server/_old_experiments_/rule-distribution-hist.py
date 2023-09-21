#!/usr/bin/env python3

import pandas
import seaborn as sns
from matplotlib import pyplot as plt
import sys
from pathlib import Path

if __name__ == "__main__":
    input = Path(sys.argv[1])
    df = pandas.read_csv(input)
    sns.set_style("whitegrid")
    sns.set_context("paper")
    sns.set_palette("colorblind")
    # sns.set(font_scale=1.5)
    # sns.set_style({"font.family": "serif", "font.serif": ["Times"]})
    # sns.set_style({"text.usetex": True})

    df = df.pivot("rule", "name", "value")
    sns.jointplot(df, x="average", y="differential")

    plt.savefig(f"{input.stem}.png")
    
