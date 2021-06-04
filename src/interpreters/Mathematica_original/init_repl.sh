#!/bin/bash
working_dir=$1
if [ -z "$working_dir" ]; then
    exit 1
fi
echo "starting"


pipe=pipe_in
out=out_file


rm -rf $working_dir/
mkdir -p $working_dir

mkfifo $working_dir/$pipe
touch $working_dir/$out
sleep 36000 > $working_dir/$pipe &
WolframKernel -noprompt < $working_dir/$pipe > $working_dir/$out &

echo "done"
echo "done_logged" > $working_dir/log-$id

sleep 36000
