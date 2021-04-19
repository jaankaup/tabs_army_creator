#!/bin/bash

set -ex

if [ -z "$1" ] || [ $1 = "all" ]; then
  echo "Compiling the project and copying the files to the server."
  wasm-pack build --target web
elif [ "$1" = "copy" ]; then
  echo "Just copying files to the server."
fi

echo "Clearing server folder."
ssh jaankaup@130.234.208.250 "cd ~/public_html/webassembly && ./remove_wasm.sh"
ssh jaankaup@130.234.208.250 "cd ~/public_html/src/tabs_projekti && ./remove_wasm_src.sh"
echo "Copying files to the server."
scp -r index.html *.js the_css.css pkg jaankaup@130.234.208.250:/home/jaankaup/public_html/webassembly
scp -r index.html Cargo.* src *.js the_css.css pkg jaankaup@130.234.208.250:/home/jaankaup/public_html/src/tabs_projekti
