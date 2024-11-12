clean:
	rm -rf target/wasm32-unknown-unknown/release
	cargo fmt --all

build:
	make clean
	stellar contract build
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release/kale_sc.wasm

install:
	make build
	stellar contract install --wasm target/wasm32-unknown-unknown/release/kale_sc.optimized.wasm --network testnet --source default

deploy:
	make build
	stellar contract deploy --wasm target/wasm32-unknown-unknown/release/kale_sc.optimized.wasm --network testnet --source default
	
bindings:
	stellar contract bindings typescript --network vc --wasm target/wasm32-unknown-unknown/release/kale_sc.optimized.wasm --contract-id NIL --output-dir kale-sc-sdk --overwrite