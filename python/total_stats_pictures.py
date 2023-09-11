import argparse
import os
import sqlite3
from typing import List, Optional

import matplotlib.pyplot as plt
import pandas as pd

from python.config_collection import ConfigCollection
from python.position_runs import PositionRuns


def plot_line_stats(axis: plt.Axes, stats: List[List[float]], labels: List[str], baseline: Optional[List[float]] = None, x_numbers: Optional[List[float]] = None, line_styles: Optional[List[str]] = None):
    if baseline is not None:
        updated_stats = []
        for stat_list in stats:
            new_stat_list = []
            for stat, baseline_stat in zip(stat_list, baseline):
                new_stat_list.append(stat / baseline_stat)

            updated_stats.append(new_stat_list)
    else:
        updated_stats = stats

    if line_styles is None:
        line_styles = ["-"] * len(labels)

    if x_numbers is not None:
        for stat_list, label, ls in zip(updated_stats, labels, line_styles):
            axis.plot(x_numbers, stat_list, label=label, ls=ls)
    else:
        for stat_list, label, ls in zip(updated_stats, labels, line_styles):
            axis.plot(stat_list, label=label, ls=ls)


if __name__ == "__main__":
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
    # arg_parser.add_argument(
    #     "--configs",
    #     help="The list of configs to use for analysis",
    #     default=[1, 2, 3, 4, 5, 6, 7, 8],
    #     nargs="+",
    # )
    args = arg_parser.parse_args()
    # args.optimal_params_path = os.path.join(args.output_path, "optimal_params.csv")
    args.total_nodes_visited_plot_path = os.path.join(args.output_path, "total_nodes_visited.png")
    args.mt_searches_plot_path = os.path.join(args.output_path, "mt_searches.png")
    args.nodes_per_second_plot_path = os.path.join(args.output_path, "nodes_per_second.png")
    args.total_time_taken_plot_path = os.path.join(args.output_path, "total_time_taken.png")

    if not os.path.exists(args.output_path):
        os.mkdir(args.output_path)

    db = sqlite3.connect(args.db_path)

    config_df = pd.read_sql_query("""
        SELECT * FROM config
    """, db)

    config_collections = []
    for _, config in config_df.iterrows():
        config_collection = ConfigCollection(config.to_dict(), db)
        print("config id", config_collection.config_id)
        print("algorithm_used", config_collection.config["algorithm_used"])
        print("time_taken_per_depth", config_collection.time_taken_per_depth)
        print("nodes_visited_per_depth", config_collection.nodes_visited_per_depth)
        print("nodes_per_sec_per_depth", config_collection.nodes_per_sec_per_depth)
        print("mt_searches_per_depth", config_collection.mt_searches_per_depth)

        config_collections.append(config_collection)

    baseline_config_collection = config_collections[0]

    # fig, ax = plt.subplots(1)
    # fig: plt.Figure
    # ax: plt.Axes
    # ax.set_yticklabels(classes or model.classes_)
    # ax.set_title("nodes_visited_per_depth")
    # ax.set_xticklabels([])

    # for config_col in config_collections:
    #     if config_col.config["algorithm_used"] == "MtdH":
    #         config_col.plot_nodes_visited_per_depth(ax, line_style=":")
    #     else:
    #         config_col.plot_nodes_visited_per_depth(ax)

    fig, ax = plt.subplots(1)
    fig: plt.Figure
    ax: plt.Axes
    plot_line_stats(
        ax,
        list(map(lambda x: x.nodes_visited_per_depth, config_collections)),
        list(map(lambda x: x.label, config_collections)),
        baseline=baseline_config_collection.nodes_visited_per_depth,
        x_numbers=list(range(1, len(config_collections) + 1)),
        line_styles=list(map(lambda x: x.line_style, config_collections))
    )
    ax.legend()
    ax.set_title("Nodes visited per depth as a multiple of Alpha Beta search")
    ax.set_ylabel("nodes visited")
    ax.set_xlabel("depth")
    # plt.show()
    plt.savefig(args.total_nodes_visited_plot_path)

    fig, ax = plt.subplots(1)
    fig: plt.Figure
    ax: plt.Axes
    plot_line_stats(
        ax,
        list(map(lambda x: x.mt_searches_per_depth, config_collections)),
        list(map(lambda x: x.label, config_collections)),
        x_numbers=list(range(1, len(config_collections) + 1)),
        line_styles=list(map(lambda x: x.line_style, config_collections))
    )
    ax.legend()
    ax.set_title("Number of MT searches per depth")
    ax.set_ylabel("number of MT searches")
    ax.set_xlabel("depth")
    # plt.show()
    plt.savefig(args.mt_searches_plot_path)

    fig, ax = plt.subplots(1)
    fig: plt.Figure
    ax: plt.Axes
    plot_line_stats(
        ax,
        list(map(lambda x: x.nodes_per_sec_per_depth[2:], config_collections)),
        list(map(lambda x: x.label, config_collections)),
        x_numbers=list(range(3, len(config_collections) + 1)),
        line_styles=list(map(lambda x: x.line_style, config_collections))
    )
    ax.legend()
    ax.set_title("nodes per second per depth")
    ax.set_ylabel("nodes per second")
    ax.set_xlabel("depth")
    # plt.show()
    plt.savefig(args.nodes_per_second_plot_path)

    fig, ax = plt.subplots(1)
    fig: plt.Figure
    ax: plt.Axes
    plot_line_stats(
        ax,
        list(map(lambda x: x.time_taken_per_depth[2:], config_collections)),
        list(map(lambda x: x.label, config_collections)),
        baseline=baseline_config_collection.time_taken_per_depth[2:],
        x_numbers=list(range(3, len(config_collections) + 1)),
        line_styles=list(map(lambda x: x.line_style, config_collections))
    )
    ax.legend()
    ax.set_title("time taken per depth as a multiple of Alpha Beta search")
    ax.set_ylabel("time taken")
    ax.set_xlabel("depth")
    # plt.show()
    plt.savefig(args.total_time_taken_plot_path)
