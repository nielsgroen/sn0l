from collections import namedtuple
from typing import Any, Optional
from functools import cached_property

import pandas as pd
import matplotlib.pyplot as plt

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

class PositionRuns:
    """Represents the contents of an analysis on a list of runs on chess positions (not matches) with the same config"""

    def __init__(self, config_id: int, db: Any, depth: Optional[int] = None):
        if depth is None:
            depth_where_clause = ""
        else:
            depth_where_clause = "depth = {} AND".format(depth)

        pandas_config = pd.read_sql_query("SELECT * FROM config WHERE id = {}".format(config_id), db)
        row = pandas_config.iloc[0]

        self.config = {
            "id": config_id,
            "max_search_depth": row["max_search_depth"],
            "algorithm_used": row["algorithm_used"],
            "conspiracy_search_used": row["conspiracy_search_used"],
            "bucket_size": row["bucket_size"],
            "num_buckets": row["num_buckets"],
            "conspiracy_merge_fn": row["conspiracy_merge_fn"],
            "transposition_table_used": row["transposition_table_used"],
            "minimum_transposition_depth": row["minimum_transposition_depth"],
            "timestamp": row["timestamp"],
        }

        self.runs = pd.read_sql_query("SELECT * FROM run WHERE config_id = {}".format(config_id), db)

        self.positions = pd.read_sql_query("""SELECT * FROM position_search WHERE {} run_id IN (
            SELECT id FROM run WHERE config_id = {}
        )""".format(depth_where_clause, config_id), db)

        self.mt_searches = pd.read_sql_query("""
            SELECT
                id,
                position_search_id,
                test_value,
                time_taken,
                nodes_evaluated,
                eval_bound,
                conspiracy_counter,
                search_num,
                timestamp,
                run_id
            FROM mt_search JOIN (SELECT run_id, id AS position_id FROM position_search) AS ps ON ps.position_id = mt_search.position_search_id
            WHERE position_search_id IN (
                SELECT id FROM position_search WHERE {} run_id IN (
                    SELECT id FROM run WHERE config_id = {}
                    )
                );
        """.format(depth_where_clause, config_id), db)

    @cached_property
    def total_stats(self):
        return calc_stats(self.positions, self.mt_searches)

    @cached_property
    def per_run_stats(self):
        separate_positions = self.positions.groupby("run_id")
        separate_mt_searches = self.mt_searches.groupby("run_id")

        sep_merged = map(lambda x: (x[0][0], x[0][1], x[1][1]), zip(separate_positions, separate_mt_searches))

        return list(map(lambda x: (x[0], calc_stats(x[1], x[2])), sep_merged))
