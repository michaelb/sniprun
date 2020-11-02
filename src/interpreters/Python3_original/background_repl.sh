#!/bin/bash
WORK_DIR=$HOME/.cache/sniprun/python3_original
rm -f $WORK_DIR/in_pipe
# this cleans annoying eventual remnants

INPUT=$WORK_DIR/in_pipe
OUTPUT=$WORK_DIR/out_pipe
touch $OUTPUT
mkfifo $INPUT

python -i -q <$INPUT >>$OUTPUT &
sleep infinity >"$INPUT" &

# echo "test = lambda a : print(a)" >>$INPUT &
MAIN_FILE=$WORK_DIR/main.py
echo "sniprunexecmain = lambda: exec(open(\"$MAIN_FILE\").read())" >>$INPUT &
