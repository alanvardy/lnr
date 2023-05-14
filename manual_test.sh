#!/bin/sh
# Does not cover the complete function
# Run it manually to ensure that we didn't break clap

commands=(
"cargo run -- -h"
)

for cmd in "${commands[@]}"
do
  echo "Executing command: $cmd"
  eval "$cmd"
  if [ $? -ne 0 ]; then
    echo "Command failed: $cmd"
    exit 1
  fi
done