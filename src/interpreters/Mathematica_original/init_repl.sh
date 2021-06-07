#!/bin/bash
working_dir=$1
if [ -z "$working_dir" ]; then
    exit 1
fi



pipe=pipe_in
out=out_file


rm -rf $working_dir/
mkdir -p $working_dir

echo "WolframKernel process started at $(date +"%F %T")." >> $working_dir/log
mkfifo $working_dir/$pipe
touch $working_dir/$out
sleep 36000 > $working_dir/$pipe &
WolframKernel -noprompt < $working_dir/$pipe &> $working_dir/$out &

echo "done_logged" >> $working_dir/log





while pkill -0 nvim ;do
    sleep 1
done

pkill -P $$

echo "WolframKernel and other backoung process terminated at $(date +"%F %T")." >> $working_dir/log

