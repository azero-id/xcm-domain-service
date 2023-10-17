#!/bin/bash
# Based on https://github.com/paritytech/extended-parachain-template/blob/main/zombienet.sh

ZOMBIENET_V=v1.3.69
POLKADOT_V=release-v1.0.0
PARACHAIN_V=contracts-v1.0.0

case "$(uname -s)" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    *)          exit 1
esac

if [ $MACHINE = "Linux" ]; then
  ZOMBIENET_BIN=zombienet-linux-x64
elif [ $MACHINE = "Mac" ]; then
  ZOMBIENET_BIN=zombienet-macos
fi

build_polkadot(){
  echo "cloning polkadot repository..."
  CWD=$(pwd)
  mkdir -p bin
  pushd /tmp
    git clone --depth 1 https://github.com/paritytech/polkadot.git --branch $POLKADOT_V
    pushd polkadot
      echo "building polkadot executable..."
      cargo build --release --features fast-runtime
      cp target/release/polkadot $CWD/bin
    popd
  popd
}

build_parachain(){
  echo "cloning azero-id:contracts-parachain repository..."
  CWD=$(pwd)
  mkdir -p bin
  pushd /tmp
    git clone https://github.com/azero-id/contracts-parachain.git
    pushd contracts-parachain
      git checkout $PARACHAIN_V
      echo "building parachain executable..."
      cargo build --release
      cp target/release/parachain-template-node $CWD/bin/parachain
    popd
  popd
}

zombienet_init() {
  if [ ! -f $ZOMBIENET_BIN ]; then
    echo "fetching zombienet executable..."
    curl -LO https://github.com/paritytech/zombienet/releases/download/$ZOMBIENET_V/$ZOMBIENET_BIN
    chmod +x $ZOMBIENET_BIN
  fi
  if [ ! -f bin/polkadot ]; then
    build_polkadot
  fi
  if [ ! -f bin/parachain ]; then
    build_parachain
  fi
}

zombienet_spawn() {
  zombienet_init
  echo "spawning local relay chain plus contracts-parachain..."
  ./$ZOMBIENET_BIN spawn config/native.toml -c 1
}

print_help() {
  echo "This is a shell script to automate the execution of zombienet."
  echo ""
  echo "$ ./zombienet.sh init         # fetches zombienet and polkadot executables"
  echo "$ ./zombienet.sh spawn        # spawns a rococo-local relay chain plus contracts-parachain"
}

SUBCOMMAND=$1
case $SUBCOMMAND in
  "" | "-h" | "--help")
    print_help
    ;;
  *)
    shift
    zombienet_${SUBCOMMAND} $@
    if [ $? = 127 ]; then
      echo "Error: '$SUBCOMMAND' is not a known SUBCOMMAND." >&2
      echo "Run './zombienet.sh --help' for a list of known subcommands." >&2
        exit 1
    fi
  ;;
esac