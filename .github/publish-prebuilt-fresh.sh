#!/bin/sh
set -ex
git checkout --orphan "tmp-prebuilt-$1"
git rm -r --cached .
rm .gitignore
printf "*/**/target/\ntarget/\nassets/\nintegration-tests\nnvim-winpick-core/\nnvim_winpick/\n.github/\nbuild.lua\n" > .gitignore
cargo b -p nvim_winpick --profile lto --target $1
cp "./target/$1/lto/$2" lua/$3
git config --global user.email nvim_winpick_ci@email.com
git config --global user.name nvim_winpick_ci
git add .
git commit -m "publish latest"
git push --force origin "tmp-prebuilt-$1":"$1-latest"

