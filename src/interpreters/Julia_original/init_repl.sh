#!/bin/bash

# this script takes 3 (or more) args.
# $1 is a path to a working directory
# $2 is the PID of the parent neovim session. All processes created here will be killed when the parent exits
# $3, $4 ... are the repl command to launch (ex: "deno", "repl" "-q")


working_dir="$1/fifo_repl"
log=$working_dir/../background_repl_process.log
if test -e "$working_dir" ; then
    echo "process already present" >> $log
    exit 1
fi




pipe=pipe_in
out=out_file
err=err_file

nvim_pid=$2

shift 2
repl="$@" # the rest


rm -rf $working_dir/
mkdir -p $working_dir
echo "pid of parent neovim session is $nvim_pid" >> $log
echo "setting up things" >> $log

mkfifo $working_dir/$pipe
touch $working_dir/$out
sleep 36000 > $working_dir/$pipe &

echo "/bin/cat " $working_dir/$pipe " | " $repl  > $working_dir/real_launcher.sh
chmod +x $working_dir/real_launcher.sh

echo $repl " process started at $(date +"%F %T")." >> $log
bash $working_dir/real_launcher.sh | tee $working_dir/$out 2| tee $working_dir/$err &

echo "done" >> $log


# wait for the parent nvim process to exit, then kill the sniprun process that launched this script

while ps -p $nvim_pid ;do
    sleep 1
done


pkill -P $$

echo $repl " and other background processes terminated at $(date +"%F %T")." >> $log

rm -rf $working_dir
