import sys
import argparse
import sqlite3

import pandas as pd
import matplotlib.pyplot as plt

from python.per_alg_analysis import calc_time_stats, calc_stats, PositionRuns

if __name__ == "__main__":
    print(sys.argv)

    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument(
        "-o",
        "--output-path",
        help="Folder to write to",
        default="temp.txt",
    )
    arg_parser.add_argument(
        "-d",
        "--db-path",
        help="The path of the DB to read from",
        default="../sqlite.db",
    )
    args = arg_parser.parse_args()

    db = sqlite3.connect(args.db_path)

    a = PositionRuns(4, db, depth=7)
    print(a.total_stats)
    print(a.per_run_stats)
    print(len(a.per_run_stats))

    a = PositionRuns(4, db, depth=8)
    print(a.total_stats)
    print(a.per_run_stats)
    print(len(a.per_run_stats))

    a = PositionRuns(4, db)
    print(a.total_stats)
    print(a.per_run_stats)
    print(len(a.per_run_stats))
