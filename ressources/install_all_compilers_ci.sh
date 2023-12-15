sudo add-apt-repository ppa:neovim-ppa/stable -y
sudo apt-get update
sudo apt-get install neovim # install neovim 0.5+

if ! command -v ocaml &> /dev/null
then
    sudo apt-get install ocaml
fi

if ! command -v ghc &> /dev/null
then
    sudo apt-get install haskell-platform -y
fi

if ! command -v node &> /dev/null
then
    sudo apt-get install -y nodejs
fi

if ! command -v npm &> /dev/null
then
    sudo apt-get install -y npm 
fi

if ! command -v coffee &> /dev/null
then
    npm install -g coffee-script 
fi

if ! command -v ts-node &> /dev/null
then
    npm install -g typescript
    npm install -g ts-node
fi

if ! command -v Rscript &> /dev/null
then
    sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E298A3A825C0D65DFD57CBB651716619E084DAB9  
    sudo add-apt-repository 'deb https://cloud.r-project.org/bin/linux/ubuntu focal-cran40/' 
    sudo apt-get install r-base 
fi

#ADA
if ! command -v gnatmake &> /dev/null
then
    sudo apt-get install gnat 
fi

if ! command -v scalac &> /dev/null
then
    sudo apt-get install scala
fi
pip3 install jupyter 

if ! command -v lua &> /dev/null
then
    sudo apt-get install lua5.3 
fi

if ! command -v sage &> /dev/null
then
    sudo apt-get install sagemath
fi

# sudo apt-get install gprolog

if ! command -v dotnet &> /dev/null
then
    sudo apt-get install dotnet
fi

if ! command -v clojure &> /dev/null
then
    sudo apt-get install clojure
fi



if ! command -v go &> /dev/null
then
    ./ressources/go_install.sh 
    export PATH=$PATH:$HOME/golang/go/bin/
fi

if ! command -v mono &> /dev/null 
then
    sudo apt-get install dirmngr gnupg apt-transport-https ca-certificates -y
    sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF
    sudo apt-add-repository 'deb <a href="https://download.mono-project.com/repo/ubuntu">https://download.mono-project.com/repo/ubuntu</a> stable-bionic main'
    sudo apt-get update
    sudo apt-get install mono-complete -y
fi

# deno for typescript and javascript
# cargo install deno --locked # too long, takes 20 min!
curl -fsSL https://deno.land/x/install/install.sh | sh
cp $HOME/.deno/bin/* $HOME/.cargo/bin
