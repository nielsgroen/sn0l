from collections import namedtuple
from typing import Any, Optional
from functools import cached_property

import pandas as pd
import matplotlib.pyplot as plt

from python.conspiracy_analysis import which_bucket

TimeStats = namedtuple("TimeStats", ["time_taken", "nodes_evaluated", "nodes_per_sec"])
StatsBase = namedtuple("StatsBase", ["time_taken", "nodes_evaluated", "nodes_per_sec", "num_mt_searches"])


class Stats(StatsBase):
    def to_time_stats(self):
        return TimeStats(self.time_taken, self.nodes_evaluated, self.nodes_per_sec)


def calc_time_stats(df: pd.DataFrame):
    time_taken = df["time_taken"].sum()
    total_nodes_evaluated = df["nodes_evaluated"].sum()
    nodes_per_sec = 1000 * total_nodes_evaluated / (time_taken + 1)  # time taken is in ms

    return TimeStats(time_taken, total_nodes_evaluated, nodes_per_sec)


def calc_stats(position_df: pd.DataFrame, mt_df: pd.DataFrame):
    time_stats = calc_time_stats(position_df)
    num_mt_rows = mt_df.shape[0]

    # calc num mt searches
    return Stats(time_stats.time_taken, time_stats.nodes_evaluated, time_stats.nodes_per_sec, num_mt_rows)

