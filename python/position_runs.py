from functools import cached_property
from typing import Any, Optional, List

import pandas as pd
from scipy.optimize import minimize

from python.conspiracy_analysis import which_bucket, parse_conspiracy_string, \
    generate_conspiracy_training_data_up_buckets, generate_conspiracy_training_data_down_buckets
from python.mse import generate_mse_error_fn
from python.per_alg_analysis import calc_stats


class PositionRuns:
    """Represents the contents of an analysis on a list of runs on chess positions (not matches) with the same config"""

    def __init__(self, config_id: int, db: Any, depth: Optional[int] = None, first_n_rows: Optional[int] = None, offset: Optional[int] = None):
        if depth is None:
            depth_where_clause = ""
        else:
            depth_where_clause = "depth = {} AND".format(depth)

        if first_n_rows is None and offset is None:
            limit_clause = ""
        elif offset is None:
            limit_clause = "LIMIT {}".format(first_n_rows)
        else:
            limit_clause = "LIMIT {} OFFSET {}".format(99999999999 if first_n_rows is None else first_n_rows, offset)

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

        self.runs = pd.read_sql_query("SELECT * FROM run WHERE config_id = {} {}".format(config_id, limit_clause), db)

        self.positions = pd.read_sql_query("""SELECT * FROM position_search WHERE {} run_id IN (
            SELECT id FROM run WHERE config_id = {} {}
        )""".format(depth_where_clause, config_id, limit_clause), db)

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
                    SELECT id FROM run WHERE config_id = {} {}
                    )
                );
        """.format(depth_where_clause, config_id, limit_clause), db)

    @cached_property
    def total_stats(self):
        return calc_stats(self.positions, self.mt_searches)

    @cached_property
    def per_run_stats(self):
        separate_positions = self.positions.groupby("run_id")
        separate_mt_searches = self.mt_searches.groupby("run_id")

        sep_merged = map(lambda x: (x[0][0], x[0][1], x[1][1]), zip(separate_positions, separate_mt_searches))

        return list(map(lambda x: (x[0], calc_stats(x[1], x[2])), sep_merged))

    @cached_property
    def evaluations(self):
        return self.positions["evaluation"].tolist()

    @cached_property
    def evaluation_bucket_nums(self):
        return list(map(lambda x: which_bucket(x, self.config["bucket_size"], self.config["num_buckets"]), self.evaluations))

    def generate_training_data(self, target_bucket_nums: List[int]):
        up_result_list = []
        down_result_list = []

        for pos_index, pos in self.positions.iterrows():
            conspiracy_counter = parse_conspiracy_string(pos.loc["conspiracy_counter"])
            own_evaluation_bucket = which_bucket(pos.loc["evaluation"], self.config["bucket_size"], self.config["num_buckets"])

            down_result_list.extend(
                generate_conspiracy_training_data_down_buckets(
                    conspiracy_counter.down_buckets,
                    own_evaluation_bucket,
                    target_bucket_nums[pos_index],
                )
            )

            up_result_list.extend(
                generate_conspiracy_training_data_up_buckets(
                    conspiracy_counter.up_buckets,
                    own_evaluation_bucket,
                    target_bucket_nums[pos_index],
                )
            )

            # print(conspiracy_counter.down_buckets)
            # print(conspiracy_counter.up_buckets)

        up_result = pd.concat(up_result_list, ignore_index=True)
        down_result = pd.concat(down_result_list, ignore_index=True)

        return down_result, up_result

    def get_optimal_params(self, target_bucket_nums: List[int]):
        down_training_data, up_training_data = self.generate_training_data(target_bucket_nums)

        error_fn = generate_mse_error_fn(down_training_data, up_training_data)
        optimize_result = minimize(error_fn, (0.9, 0.5, 0.5, 0.001), bounds=((0, 1), (0, 1), (0, 1), (0, 1)))

        p = optimize_result.x[0]
        w_side_down = optimize_result.x[1]
        w_side_up = optimize_result.x[2]
        c = optimize_result.x[3]

        return p, w_side_down, w_side_up, c
