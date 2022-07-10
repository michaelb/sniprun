sudo add-apt-repository ppa:neovim-ppa/stable -y
sudo apt-get update
sudo apt-get install neovim # install neovim 0.5+

sudo apt-get install haskell-platform -y
sudo apt-get install -y nodejs npm 
npm install -g coffee-script 
npm install -g typescript
npm install -g ts-node
sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E298A3A825C0D65DFD57CBB651716619E084DAB9  
sudo add-apt-repository 'deb https://cloud.r-project.org/bin/linux/ubuntu focal-cran40/' 
sudo apt-get install r-base 
sudo apt-get install gnat 
sudo apt-get install scala 
pip3 install jupyter 
sudo apt-get install lua5.3 
sudo apt-get install sagemath
sudo apt-get install gprolog
sudo apt-get install dotnet
./ressources/go_install.sh 
export PATH=$PATH:$HOME/golang/go/bin/

# deno for typescript and javascript
# cargo install deno --locked # too long, takes 20 min!
curl -fsSL https://deno.land/x/install/install.sh | sh
cp $HOME/.deno/bin/* $HOME/.cargo/bin
