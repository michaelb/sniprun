#!/bin/bash
working_dir="$1/fifo_repl"
echo "sync requested" >> $working_dir/log

pipe=pipe_in
out=out_file
err=err_file

echo "" > $working_dir/$pipe

echo "sync done" >> $working_dir/log
