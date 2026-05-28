#!/bin/bash
# hindsight.bash — source this in your .bashrc
# Add to ~/.bashrc: source ~/.hindsight/hindsight.bash

[[ $- == *i* ]] || return 0

HINDSIGHT_BIN=${HINDSIGHT_BIN:-$(command -v hindsight 2>/dev/null)}
[[ -z "$HINDSIGHT_BIN" ]] && return 0

function _hindsight_preexec {
    export _HINDSIGHT_CMD="$1"
    export _HINDSIGHT_START=$SECONDS
}

function _hindsight_precmd {
    local exit_code=$?

    if [[ -n "$_HINDSIGHT_CMD" ]]; then
        local duration=$(( SECONDS - ${_HINDSIGHT_START:-$SECONDS} ))
        $HINDSIGHT_BIN add \
            --command "$_HINDSIGHT_CMD" \
            --cwd "$PWD" \
            --exit $exit_code \
            --duration $duration \
            2>/dev/null &
        unset _HINDSIGHT_CMD
        unset _HINDSIGHT_START
    fi

    return $exit_code
}

trap '_hindsight_preexec "$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="_hindsight_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
