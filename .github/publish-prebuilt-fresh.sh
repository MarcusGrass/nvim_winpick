#!/bin/sh
set -ex
git checkout --orphan "tmp-prebuilt-$1"
git rm -r --cached .
rm .gitignore
printf "*/**/target/\nassets/\nlua/nvim_winpick/\n.github/\nbuild.lua\n" > .gitignore
cd ./lua/nvim_winpick
cargo b -p nvim_winpick --profile lto --target $1
cp "./target/$1/lto/$2" ../$3
cd ../../
git config --global user.email nvim_winpick_ci@email.com
git config --global user.name nvim_winpick_ci
git add .
git commit -m "publish latest"
git push --force origin "tmp-prebuilt-$1":"$1-latest"

