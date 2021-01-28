#!/bin/bash
set -eu
cargo hack test --each-feature --skip decode_action --features decode_query_compare_with_value,alloc
cargo test --no-default-features --features decode_all_actions,decode_all_queries
cargo test --no-default-features --features decode_all_actions,decode_all_queries,alloc
