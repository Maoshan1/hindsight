#!/bin/zsh
# hindsight.zsh — source this in your .zshrc
# Add to ~/.zshrc: source ~/.hindsight/hindsight.zsh

[[ -o interactive ]] || return 0

HINDSIGHT_BIN=${HINDSIGHT_BIN:-$(command which hindsight 2>/dev/null)}
if [[ -z "$HINDSIGHT_BIN" || "$HINDSIGHT_BIN" == "hindsight not found" ]]; then
    return 0
fi

# Preexec: record command start time
_hindsight_preexec() {
    _HINDSIGHT_CMD="$1"
    _HINDSIGHT_START=$SECONDS
}

# Precmd: record command with exit code and duration
_hindsight_precmd() {
    local _exit=$?
    local _dur=$(( SECONDS - ${_HINDSIGHT_START:-$SECONDS} ))

    if [[ -n "$_HINDSIGHT_CMD" ]]; then
        $HINDSIGHT_BIN add --command "$_HINDSIGHT_CMD" --cwd "$PWD" --exit $_exit --duration $_dur &>/dev/null &
        disown 2>/dev/null
    fi

    _HINDSIGHT_CMD=""
    _HINDSIGHT_START=""
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _hindsight_preexec
add-zsh-hook precmd _hindsight_precmd
