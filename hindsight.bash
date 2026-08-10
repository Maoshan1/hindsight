#!/bin/bash
# hindsight.bash — source this in your .bashrc
# Add to ~/.bashrc: source ~/.hindsight/hindsight.bash

[[ $- == *i* ]] || return 0

HINDSIGHT_BIN=${HINDSIGHT_BIN:-$(command -v hindsight 2>/dev/null)}
[[ -z "$HINDSIGHT_BIN" ]] && return 0

function _hindsight_preexec {
    export _HINDSIGHT_CMD="$1"
    export _HINDSIGHT_START_MS=$(_hindsight_now_ms)
}

function _hindsight_now_ms {
    local now
    now=$(date +%s%3N 2>/dev/null)
    if [[ "$now" =~ ^[0-9]+$ ]]; then
        printf '%s' "$now"
    else
        printf '%s000' "$(date +%s)"
    fi
}

function _hindsight_precmd {
    local exit_code=$?
    local now_ms=$(_hindsight_now_ms)

    if [[ -n "$_HINDSIGHT_CMD" ]]; then
        local duration=$(( now_ms - ${_HINDSIGHT_START_MS:-now_ms} ))
        $HINDSIGHT_BIN add \
            --command "$_HINDSIGHT_CMD" \
            --cwd "$PWD" \
            --exit $exit_code \
            --duration $duration \
            2>/dev/null &
        unset _HINDSIGHT_CMD
        unset _HINDSIGHT_START_MS
    fi

    return $exit_code
}

trap '_hindsight_preexec "$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="_hindsight_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
