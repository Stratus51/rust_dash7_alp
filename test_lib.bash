#!/bin/bash
set -eu


FEATURE_SETS=""
add() {
    FEATURE_SETS="${FEATURE_SETS} $1"
}

# Test decoding action one by one
add decode_nop
add decode_read_file_data
add decode_read_file_properties
add decode_write_file_data,alloc
add decode_action_query,decode_query_compare_with_value,alloc
add decode_action_query,decode_query_compare_with_range,alloc
add decode_status

# Test action builder
add all_actions,all_queries
add all_actions,all_queries,alloc

# Test all
add decode_all_actions,decode_all_queries
add decode_all_actions,decode_all_queries,alloc

FEATURE_SETS="$(echo "$FEATURE_SETS" | grep -v -e '^$')"

NB_FEATURE_SETS=$(echo "$FEATURE_SETS" | wc -w)
i=0
for feature_set in $FEATURE_SETS; do
    let i=i+1
    echo "Test $i/$NB_FEATURE_SETS"
    echo cargo test --no-default-features --features $feature_set
    cargo test --no-default-features --features $feature_set
done
