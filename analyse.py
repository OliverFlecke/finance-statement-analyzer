#!/usr/bin/env python3

import argparse
import csv
from itertools import groupby
from types import FunctionType
from typing import Dict

parser = argparse.ArgumentParser()
parser.add_argument('filename', help='Name of the file to analyse')

args = parser.parse_args()
filename = args.filename

with open(filename, 'r') as file:
    rows = list(csv.DictReader(file))

get_value = lambda x: float(x["Debit Amount"] if x["Debit Amount"] != "" else x["Credit Amount"])

def toGroups(rows):
    d = {}
    for row in rows:
        categories = row['Category'].split('/')

        current = d
        for category in categories:
            if category not in current: current[category] = {}
            current = current[category]

        if 'items' not in current:
            current['items'] = []
        current['items'].append(get_value(row))

    return d
    # groups = groupby(rows, lambda x: x['Category'].split('/')[depth])

    # return map(lambda k, rs: (k, toGroups(rs, depth + 1)), groups)

def mapTree(f: FunctionType, t: Dict) -> Dict:
    return {
        category: f(items)
        for category, items in t.items()
    }

groups = toGroups(rows)

# print(mapTree(lambda x: , groups))

def output(groups: Dict, indent = 0):
    for category, items in filter(lambda name: name[0] != 'items', groups.items()):
        amount = sum(items['items']) if 'items' in items else 0

        print('\t' * indent + f'{category}: {amount}')
        output(items, indent + 1)

# output(groups)

# for category, rows in groups:
#     amount = ["Debit Amount"]
#     print(f'{category}: {sum(map(get_value, rows))}')

