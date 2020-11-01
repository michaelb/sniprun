WORK_DIR=$HOME/.cache/sniprun/python3_original
rm -f $WORK_DIR/*.pipe
# this cleans annoying eventual remnants

mkfifo $WORK_DIR/in_pipe

python -i -q <in_pipe >out_pipe &
sleep infinity >in_pipe &
