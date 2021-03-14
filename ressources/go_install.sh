export CURRENT_BUILD_PATH=$(pwd)
echo $PATH 
rm -rf $HOME/golang 
rm -rf $HOME/gopath 
mkdir -p $HOME/golang  
mkdir -p $HOME/gopath 
curl http://storage.googleapis.com/golang/go1.5.2.linux-amd64.tar.gz 2>/dev/null > go1.5.2.linux-amd64.tar.gz 
tar -C $HOME/golang -xzf go1.5.2.linux-amd64.tar.gz 
export GOROOT=$HOME/golang/go 
export GOPATH=$HOME/gopath 
export PATH=$PATH:$GOROOT/bin 
export PATH=$PATH:$GOPATH/bin 
(if [[ "$(go version)" == *"go version go1.5"* ]]; then echo "✓ Go binary installed!"; else echo "Go binary not installed"; exit -1; fi); 
go version 
echo $PATH 
go env 
which go
alias go=$HOME/golang/go/bin/go
