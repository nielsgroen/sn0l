from collections import namedtuple
from typing import List, Tuple
import re

import pandas as pd



# from a list of positions with conspiracy_buckets,
# generate a list of pairs of:
# X = (marginal, cumulative, marginal/cumulative)
# y = solution_is_in_bucket [= 0 or 1]
#
# `marginal` meaning the marginal number of conspiracy numbers in that bucket
# `cumulative` meaning the cumulative conspiracy number of all the buckets before it

ConspiracyBucket = namedtuple("ConspiracyBucket", ["bucket_size", "down_buckets", "up_buckets"])


def parse_conspiracy_string(input: str) -> ConspiracyBucket:
    first_split = input.split(", [")
    bucket_size, down_buckets_str, up_buckets_str = first_split[0], first_split[1], first_split[2]

    down_buckets_str, up_buckets_str = re.sub(" |\[|\]", "", down_buckets_str), re.sub(" |\[|\]", "", up_buckets_str)
    down_buckets_split, up_buckets_split = re.split(",", down_buckets_str), re.split(",", up_buckets_str)

    def parse_conspiracy_value(value: str) -> int:
        if value == "U":
            return -1
        else:
            return int(value)

    down_buckets = list(map(lambda x: parse_conspiracy_value(x), down_buckets_split))
    up_buckets = list(map(lambda x: parse_conspiracy_value(x), up_buckets_split))

    return ConspiracyBucket(bucket_size, down_buckets, up_buckets)


def generate_conspiracy_training_data_up_buckets(buckets, own_evaluation_bucket_num, target_evaluation_bucket_num) -> pd.DataFrame:
    result_list = []

    cumulative_count = 0
    for bucket_index, marginal_count in enumerate(buckets):
        # if last bucket: impassable
        if bucket_index == len(buckets) - 1:
            marginal_count = -1

        # if (cumulative_count == 0 and marginal_count == 0) or cumulative_count == -1:
        if bucket_index < own_evaluation_bucket_num or cumulative_count == -1:
            continue

        # ignore the checkmates
        if marginal_count == -1 and cumulative_count == 0 and (bucket_index == 0 or bucket_index == len(buckets) - 1):
            break

        result_list.append(pd.DataFrame({
            "bucket_index": [bucket_index],
            "marginal": [marginal_count],
            "cumulative": [cumulative_count],
            "is_target_bucket": [bucket_index == target_evaluation_bucket_num],
        }))

        if marginal_count == -1:
            cumulative_count = -1
        else:
            cumulative_count += marginal_count

    # result = pd.concat(result_list, ignore_index=True)

    return result_list


def generate_conspiracy_training_data_down_buckets(buckets, own_evaluation_bucket_num, target_evaluation_bucket_num) -> pd.DataFrame:
    result_list = []

    cumulative_count = 0
    for bucket_index, marginal_count in reversed(list(enumerate(buckets))):
        # if last bucket: impassable
        # bucket 0 is last bucket when going down
        if bucket_index == 0:
            marginal_count = -1

        # if (cumulative_count == 0 and marginal_count == 0) or cumulative_count == -1:
        if bucket_index > own_evaluation_bucket_num or cumulative_count == -1:
            continue

        # ignore the checkmates
        if marginal_count == -1 and cumulative_count == 0 and (bucket_index == 0 or bucket_index == len(buckets) - 1):
            break

        result_list.append(pd.DataFrame({
            "bucket_index": [bucket_index],
            "marginal": [marginal_count],
            "cumulative": [cumulative_count],
            "is_target_bucket": [bucket_index == target_evaluation_bucket_num],
        }))

        if marginal_count == -1:
            cumulative_count = -1
        else:
            cumulative_count += marginal_count

    # result = pd.concat(result_list, ignore_index=True)

    return result_list


def which_bucket(evaluation: str, bucket_size: int, num_buckets: int) -> int:
    is_white_mate = re.fullmatch("\+M.+", evaluation) is not None
    is_black_mate = re.fullmatch("-M.+", evaluation) is not None

    if is_white_mate:
        return int(num_buckets - 1)
    elif is_black_mate:
        return int(0)
    else:
        value = int(evaluation)

        middle_bucket = num_buckets // 2
        bucket_offset = (value + bucket_size // 2) // bucket_size
        bucket_index = middle_bucket + bucket_offset

        return int(max(0, min(bucket_index, num_buckets - 1)))
