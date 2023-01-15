Juno staking Envirnment Setup

sudo apt-get update && sudo apt upgrade -y
sudo apt-get install make build-essential gcc git jq chrony -y
wget https://golang.org/dl/go1.18.1.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.18.1.linux-amd64.tar.gz
rm -rf go1.18.1.linux-amd64.tar.gz

export GOROOT=/usr/local/go
export GOPATH=$HOME/go
export GO111MODULE=on
export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin

rustup default stable
rustup target add wasm32-unknown-unknown

git clone https://github.com/CosmosContracts/juno
cd juno
git fetch
git checkout v9.0.0
make install