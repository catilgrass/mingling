#!/bin/bash

cd "$(dirname "$0")" || exit 1

if [ $# -eq 0 ]; then
    echo "Available:"
    if [ -d "dev_tools/scripts" ]; then
        for file in dev_tools/scripts/*.sh; do
            if [ -f "$file" ]; then
                basename "$file" .sh
            fi
        done
        for file in dev_tools/scripts/*.py; do
            if [ -f "$file" ]; then
                basename "$file" .py
            fi
        done
    fi
    if [ -d "dev_tools/src/bin" ]; then
        for file in dev_tools/src/bin/*.rs; do
            if [ -f "$file" ]; then
                basename "$file" .rs
            fi
        done
    fi
    exit 1
fi

target_bin="$1"
target_script="dev_tools/scripts/${target_bin}.sh"
target_python="dev_tools/scripts/${target_bin}.py"
target_file="dev_tools/src/bin/${target_bin}.rs"

if [ -f "$target_script" ]; then
    chmod +x "$target_script"
    "$target_script"
elif [ -f "$target_python" ]; then
    python "$target_python"
elif [ -f "$target_file" ]; then
    cargo run --manifest-path dev_tools/Cargo.toml --bin "$1" --quiet
else
    echo "Error: target '$target_bin' does not exist"
    exit 1
fi
