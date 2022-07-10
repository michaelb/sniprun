add-apt-repository ppa:neovim-ppa/stable -y
apt-get update
apt-get install neovim # install neovim 0.5+

apt-get install haskell-platform -y
apt-get install -y nodejs npm 
npm install -g coffee-script 
npm install -g typescript
npm install -g ts-node
apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E298A3A825C0D65DFD57CBB651716619E084DAB9  
add-apt-repository 'deb https://cloud.r-project.org/bin/linux/ubuntu focal-cran40/' 
apt-get install r-base 
apt-get install gnat 
apt-get install scala 
pip3 install jupyter 
apt-get install lua5.3 
apt-get install sagemath
apt-get install gprolog
apt-get install dotnet
./ressources/go_install.sh 
export PATH=$PATH:$HOME/golang/go/bin/

# deno for typescript and javascript
# cargo install deno --locked # too long, takes 20 min!
curl -fsSL https://deno.land/x/install/install.sh | sh
cp $HOME/.deno/bin/* $HOME/.cargo/bin
