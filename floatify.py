import json
import sys

def floatify(obj):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k in ['ip', 'op', 'st', 'fr', 'w', 'h'] and isinstance(v, int):
                obj[k] = float(v)
            else:
                floatify(v)
    elif isinstance(obj, list):
        for i in obj:
            floatify(i)

try:
    with open(sys.argv[1], 'r') as f:
        data = json.load(f)

    floatify(data)

    with open(sys.argv[1], 'w') as f:
        json.dump(data, f, indent=4)
    print(f"Processed {sys.argv[1]}")
except Exception as e:
    print(e)
