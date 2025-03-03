#!/bin/sh
set -ex
git fetch
cd ./lua/nvim_winpick
cargo b -p nvim_winpick --profile lto --target $1
TMP_LIB="$4/lib.so"
cp "./target/$1/lto/$2" "$TMP_LIB"
git config --global user.email nvim_winpick_ci@email.com
git config --global user.name nvim_winpick_ci
git stash
git checkout "$1-latest"
if diff $TMP_LIB ../$3; then
    echo "No need to publish"
else
    cp "$TMP_LIB" ../$3
    cd ../../
    git add "lua/$3"
    git commit -m "publish latest"
    git push origin "$1-latest"
fi

