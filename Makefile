# 我也不想写这个 makefile ，但 stable 1.86 还不支持 per crate target ，导致我不得不分别 cargo build 编译 wasm
build:
	cargo build -p wasm_resource --target wasm32-wasip2
	cargo build -p wasm_simple --target wasm32-wasip2

