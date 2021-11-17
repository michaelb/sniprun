#!/bin/bash
working_dir="$1/fifo_repl"
if test -e "$working_dir" ; then
    echo "process already present" >> $working_dir/log
    exit 1
fi



pipe=pipe_in
out=out_file
err=err_file


repl="$2"


rm -rf $working_dir/
mkdir -p $working_dir
echo "setting up things" > $working_dir/log

mkfifo $working_dir/$pipe
touch $working_dir/$out
sleep 36000 > $working_dir/$pipe &

echo "/bin/cat " $working_dir/$pipe " | " $repl  > $working_dir/real_launcher.sh
chmod +x $working_dir/real_launcher.sh

echo $repl " process started at $(date +"%F %T")." >> $working_dir/log
bash $working_dir/real_launcher.sh > $working_dir/$out 2> $working_dir/$err &

echo "done_logged" >> $working_dir/log





while pkill -0 nvim ;do
    sleep 1
done

rm -rf $working_dir

pkill -P $$

echo $repl " and other backoung process terminated at $(date +"%F %T")." >> $working_dir/log

