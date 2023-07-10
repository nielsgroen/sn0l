from typing import Callable, Tuple

import pandas as pd

# p, w_side_down, w_side_up, c
MSE_Function = Callable[[Tuple[float, float, float, float]], float]

# p, w_side, c
MSE_Function_Side = Callable[[Tuple[float, float, float]], float]


# Generate error function
def generate_mse_error_fn(down_data: pd.DataFrame, up_data: pd.DataFrame) -> MSE_Function:
    # sum down_bucket terms

    down_terms = list(map(lambda x: training_row_to_mse_term(x[1].to_dict()), down_data.iterrows()))
    up_terms = list(map(lambda x: training_row_to_mse_term(x[1].to_dict()), up_data.iterrows()))

    def mse_function(x):
        p = x[0]
        w_side_down = x[1]
        w_side_up = x[2]
        c = x[3]

        return sum(
            map(lambda fn: fn(p, w_side_down, c), down_terms)
        ) / len(down_terms) + sum(
            map(lambda fn: fn(p, w_side_up, c), up_terms)
        ) / len(up_terms)

    return mse_function


def training_row_to_mse_term(row: dict) -> MSE_Function_Side:
    marginal = row["marginal"]
    cumulative = row["cumulative"]
    is_target = int(row["is_target_bucket"])

    if marginal == -1:
        return lambda p, w_side, c: ((w_side * (p ** cumulative) + c) - is_target) ** 2

    return lambda p, w_side, c: ((w_side * (1 - p ** marginal) * (p ** cumulative) + c) - is_target) ** 2
