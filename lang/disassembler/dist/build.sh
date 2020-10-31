#!/bin/bash

set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

# cd ..
cargo build -p disassembler --lib --target wasm32-unknown-unknown --release

wasm-bindgen target/wasm32-unknown-unknown/release/disassembler.wasm --out-dir ./dist --no-modules --no-modules-global disassembler

wasm-snip --snip-rust-panicking-code --skip-producers-section lang/disassembler/dist/disassembler_bg.wasm -o lang/disassembler/dist/disassembler_bg_snip.wasm

# TODO:
# bin/wasm-opt [.wasm or .wat file] [options] [passes, see --help] [--help]
# run wasm-opt again with the --dce flag


# cd $DIR
