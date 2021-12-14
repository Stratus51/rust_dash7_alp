#!/bin/bash
set -eu

# ================================================================
# Defines
# ================================================================
red=`tput setaf 1`
green=`tput setaf 2`
reset=`tput sgr0`

# ================================================================
# Helpers
# ================================================================
FEATURE_SETS=""
add() {
    FEATURE_SETS="${FEATURE_SETS} $1"
}

name_cmd() {
    local feature_set="$1"
    echo cargo test --no-default-features --features "$feature_set"
}

cmd() {
    local feature_set="$1"
    cargo test --no-default-features --features "$feature_set"
}

run_test() {
    local feature_set="$1"
    name_cmd "$feature_set"
    cmd "$feature_set"
}

syntax() {
    echo "Syntax: $0 [specific_test_number]"
}

crash() {
    echo "${red}$@${reset}"
    exit -1
}

# ================================================================
# Arguments
# ================================================================
SPECIFIC_TEST=""
if (( $# >= 1 )); then
    SPECIFIC_TEST=$1
    if [[ -z "$(echo "$SPECIFIC_TEST" | grep "^[0-9]\+$")" ]]; then
        syntax
        crash "Specific test should be an integer, not '$SPECIFIC_TEST'"
    fi
fi

# ================================================================
# Tests definition
# ================================================================
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

# ================================================================
# Run tests
# ================================================================
# Specific test case
if [[ -n "$SPECIFIC_TEST" ]]; then
    if (( $SPECIFIC_TEST < 1 )) || (( $SPECIFIC_TEST > $NB_FEATURE_SETS )); then
        crash "Bad specific test: $SPECIFIC_TEST does not belong to [0;$NB_FEATURE_SETS]."
    fi
    echo -n "$SPECIFIC_TEST/$NB_FEATURE_SETS "
    i=0
    for feature_set in $FEATURE_SETS; do
        let i=i+1
        if [[ $i == $SPECIFIC_TEST ]]; then
            run_test "$feature_set"
            exit 0
        fi
    done
fi

# All tests
i=0
for feature_set in $FEATURE_SETS; do
    let i=i+1
    echo -n "$i/$NB_FEATURE_SETS "
    LOGS="$(run_test "$feature_set" 2>&1 || true)"
    FAILS="$(echo "$LOGS" | grep "passed; [^ ]\+ failed" | sed 's/.*\([0-9]\+\) failed.*/\1/')"
    FAILED=false
    for line in $FAILS; do
        if [[ $line != 0 ]]; then
            FAILED=true
            break
        fi
    done
    if [[ $FAILED != false ]]; then
        echo "$LOGS"
        echo "${red}Failed!${reset}"
        exit -1
    else
        echo "${green}OK${reset}"
    fi
done
