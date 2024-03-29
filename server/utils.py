from datetime import datetime

import pandas as pd
from dfply import dfpipe


def sorter(a):
    if a.name == "params":
        # return a.map(lambda x: x.replace("_", "x").split("x")).map(
        #     lambda l: reduce(lambda x, y: x * y, map(int, l))
        # )
        def per_key(el):
            if "_" in el:
                left, right = el.split("_")
                a, b = left.split("x")
                c, d = right.split("x")
                a, b, c, d = int(a), int(b), int(c), int(d)
                return ((a * b) ** 2) + (c * d)
            else:
                try:
                    return int(el)
                except ValueError:
                    return -1

        return a.map(per_key)
    elif a.name == "date":
        formats = ["%b%d-%H%M", "%b%d"]
        for f in formats:
            try:
                return a.map(lambda r: datetime.strptime(r, f))
            except ValueError:
                pass
        # if we couldn't format the date,
        return a
    else:
        return a


@dfpipe
def pivot_table(df, **kwargs):
    df = pd.pivot_table(df, **kwargs)
    return df.reset_index(drop=True, names=["index"])


@dfpipe
def reset_index(df, **kwargs):
    return df.reset_index(**kwargs)


@dfpipe
def sort_values(df, **kwargs):
    return df.sort_values(**kwargs)


@dfpipe
def to_csv(df, path, **kwargs):
    df.to_csv(path, **kwargs)
    print(f"Wrote {path}")
    return df


@dfpipe
def display(df):
    print(df.to_string())
    return df


@dfpipe
def agg(df, list):
    return df.agg(list)


@dfpipe
def replace(df, *args, **kwargs):
    return df.replace(*args, **kwargs)


@dfpipe
def iloc(df, *args):
    if df.empty:
        return df
    else:
        return df.iloc[*args]
