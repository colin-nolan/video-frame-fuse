#!/usr/bin/env python3

import json
import sys

offset = float(sys.argv[1])

for raw_line in sys.stdin:
    line = json.loads(raw_line)
    if isinstance(line, dict):
        print(raw_line, end="")
        continue
    updated_timestamp = line[0] - offset
    if updated_timestamp > 0:
        line[0] = updated_timestamp
    print(json.dumps(line))
