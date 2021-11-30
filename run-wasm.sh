#!/usr/bin/env bash

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown

mkdir -p target/wasm-example/particle-demo
wasm-bindgen --target web --out-dir target/wasm-example/particles-demo target/wasm32-unknown-unknown/debug/particles-demo.wasm
cat wasm-resources/index.template.html | sed "s/{{example}}/particles-demo/g" > target/wasm-example/particles-demo/index.html

basic-http-server target/wasm-example/particles-demo -a 127.0.0.1:1234