#!/bin/bash
set -e 
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/bevy_vaporwave.wasm

sed -i 's/getObject(arg0).focus();/const scrollPos = window.scrollY; getObject(arg0).focus(); window.scrollTo(0, scrollPos);/' ./out/bevy_vaporwave.js

