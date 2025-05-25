#!/usr/bin/env python3

import json

# Test the correct RangeFilter format based on Rust enum serialization
# Rust enums with serde serialize as {"VariantName": value} for simple variants

test_filters = [
    {"KeyPrefix": "warehouse:"},
    {"Key": "brand"},
    {"KeyPattern": "store:*"},
    {"KeyRange": {"start": "a", "end": "z"}},
    {"Keys": ["key1", "key2"]},
    {"Value": "some_value"}
]

print("Testing RangeFilter serialization formats:")
for i, filter_obj in enumerate(test_filters):
    print(f"{i+1}. {json.dumps(filter_obj, indent=2)}")