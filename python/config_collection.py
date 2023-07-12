import sqlite3
import matplotlib.pyplot as plt
import pandas as pd

from python.position_runs import PositionRuns


class ConfigCollection:

    def __init__(self, config: dict, db: sqlite3.Connection, line_style="-"):
        self.config_id = config["id"]
        self.config = config

        self.line_style = line_style
        if self.config["algorithm_used"] == "MtdH":
            self.line_style = ":"

        self.position_list = []
        for depth in range(1, self.config["max_search_depth"] + 1):
            self.position_list.append(PositionRuns(self.config_id, db, depth=depth))

    def get_position(self, depth) -> PositionRuns:
        return self.position_list[depth - 1]

    @property
    def time_taken_per_depth(self):
        result = list(map(lambda x: x.total_stats.time_taken, self.position_list))

        return result

    @property
    def nodes_visited_per_depth(self):
        result = list(map(lambda x: x.total_stats.nodes_evaluated, self.position_list))

        return result

    @property
    def nodes_per_sec_per_depth(self):
        result = list(map(lambda x: x.total_stats.nodes_per_sec, self.position_list))

        return result

    @property
    def mt_searches_per_depth(self):
        result = list(map(lambda x: x.total_stats.num_mt_searches, self.position_list))

        return result

    @property
    def label(self):
        if pd.isnull(self.config["num_buckets"] or pd.isnull(self.config["bucket_size"])):
            result = self.config["algorithm_used"]
        else:
            result = "{}, {} buckets, {} bucket size".format(
                self.config["algorithm_used"],
                self.config["num_buckets"],
                self.config["bucket_size"],
            )

        return result

    def plot_time_taken_per_depth(self, axis: plt.Axes):
        axis.plot(range(1, self.config["max_search_depth"] + 1), self.time_taken_per_depth, label=self.config["algorithm_used"], ls=self.line_style)

    def plot_nodes_visited_per_depth(self, axis: plt.Axes):
        axis.plot(range(1, self.config["max_search_depth"] + 1), self.nodes_visited_per_depth, label=self.config["algorithm_used"], ls=self.line_style)

    def plot_nodes_per_sec_per_depth(self, axis: plt.Axes):
        axis.plot(range(1, self.config["max_search_depth"] + 1), self.nodes_per_sec_per_depth, label=self.config["algorithm_used"], ls=self.line_style)

    def plot_mt_searches_per_depth(self, axis: plt.Axes):
        axis.plot(range(1, self.config["max_search_depth"] + 1), self.mt_searches_per_depth, label=self.config["algorithm_used"], ls=self.line_style)


