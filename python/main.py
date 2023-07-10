import os
import sys
import argparse
import sqlite3

import pandas as pd
import matplotlib.pyplot as plt
from scipy.optimize import minimize


from python.mse import generate_mse_error_fn
from python.position_runs import PositionRuns

if __name__ == "__main__":
    print(sys.argv)

    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument(
        "-o",
        "--output-path",
        help="Folder to write to",
        default="analysis_output",
    )
    arg_parser.add_argument(
        "-d",
        "--db-path",
        help="The path of the DB to read from",
        default="../sqlite.db",
    )
    args = arg_parser.parse_args()
    args.optimal_params_path = os.path.join(args.output_path, "optimal_params.csv")

    if not os.path.exists(args.output_path):
        os.mkdir(args.output_path)

    db = sqlite3.connect(args.db_path)

    config_df = pd.read_sql_query("""
        SELECT * FROM config
    """, db)

    print(config_df.to_string())

    mtdbi_row = config_df[
        (config_df["algorithm_used"] == "MtdBi") &
        (config_df["bucket_size"] == 20.0) &
        (config_df["num_buckets"] == 101.0) &
        (config_df["transposition_table_used"] == 1) &
        (config_df["minimum_transposition_depth"] == 2)
    ]
    training_run = mtdbi_row.iloc[0]
    training_run_id = training_run["id"]
    training_run_depth = training_run["max_search_depth"]

    def predict_down(row: dict) -> float:
        bucket_num = row["bucket_index"]
        marginal = row["marginal"]
        cumulative = row["cumulative"]
        is_target = int(row["is_target_bucket"])

        # if is last bucket: impassable
        if bucket_num == 0:
            marginal = -1

        if marginal == -1:
            return w_side_down * (p ** cumulative) + c

        return w_side_down * (1 - p ** marginal) * (p ** cumulative) + c

    def predict_up(row: dict, num_buckets) -> float:
        bucket_num = row["bucket_index"]
        marginal = row["marginal"]
        cumulative = row["cumulative"]
        is_target = int(row["is_target_bucket"])

        # if is last bucket: impassable
        if bucket_num == num_buckets - 1:
            marginal = -1

        if marginal == -1:
            return w_side_up * (p ** cumulative) + c

        return w_side_up * (1 - p ** marginal) * (p ** cumulative) + c

    position_runs_list = []
    for depth in range(1, 9):
        position_runs_list.append(PositionRuns(training_run_id, db, depth=depth))
        print("loading depth", depth, "done")

    result_list = []
    for target_depth in range(3, 9):
        p, w_side_down, w_side_up, c = position_runs_list[target_depth - 3]\
            .get_optimal_params(position_runs_list[target_depth - 1].evaluation_bucket_nums)

        result_list.append(pd.DataFrame({
            "training_depth": [target_depth - 2],
            "target_depth": [target_depth],
            "p": [p],
            "w_side_down": [w_side_down],
            "w_side_up": [w_side_up],
            "c": [c],
        }))
        print("p", p)
        print("w_side_down", w_side_down)
        print("w_side_up", w_side_up)
        print("c", c)

        p, w_side_down, w_side_up, c = position_runs_list[target_depth - 2] \
            .get_optimal_params(position_runs_list[target_depth - 1].evaluation_bucket_nums)

        result_list.append(pd.DataFrame({
            "training_depth": [target_depth - 1],
            "target_depth": [target_depth],
            "p": [p],
            "w_side_down": [w_side_down],
            "w_side_up": [w_side_up],
            "c": [c],
        }))
        print("p", p)
        print("w_side_down", w_side_down)
        print("w_side_up", w_side_up)
        print("c", c)

        print("training for target depth", target_depth, "done")

    training_params = pd.concat(result_list)
    print(training_params.to_string())

    training_params.to_csv(args.optimal_params_path, index=False)

    # print(down_training_data.head(50).to_string())
    #
    # rows = list(map(lambda x: x[1].to_dict(), down_training_data.head(50).iterrows()))
    # predictions = list(map(lambda x: predict_down(x), rows))
    #
    # print(predictions)

    # a = PositionRuns(4, db, depth=7)
    # print(a.total_stats)
    # print(a.per_run_stats)
    # print(len(a.per_run_stats))
    #
    # a = PositionRuns(4, db, depth=8)
    # print(a.total_stats)
    # print(a.per_run_stats)
    # print(len(a.per_run_stats))
    #
    # a = PositionRuns(4, db)
    # print(a.total_stats)
    # print(a.per_run_stats)
    # print(len(a.per_run_stats))
    # print(a.positions.head().to_string())
    #
    # # Conspiracy_numbers
    # bucket_size, down_bucket, up_bucket = parse_conspiracy_string("20, [U, 1, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 2, 3, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 4, 7, 19, 4, 3, 2, 0, 0, 2, 0, 3, 9, 1, 2, 1, 0, 3, 2, 1, 8, 10, 61, 60, 14, 213, 69, 620, 292, 233, 172, 64, 19, 13, 4, 10, 15, 37, 4, 8, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]")
    #
    # print(generate_conspiracy_training_data_up_buckets(up_bucket))
    # print(generate_conspiracy_training_data_down_buckets(down_bucket))
