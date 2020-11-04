#!/bin/sh

DIRECTORY=$1

echo "| Interpreter         | language    | comments"
echo "|---------------------|-------------|---------"
for file in $DIRECTORY/*.rs; do
  IFS= read -r line <$file
  if [[ ${line:0:14} == "//Interpreter:" ]]; then
    echo "${line:14}"

  fi
done
