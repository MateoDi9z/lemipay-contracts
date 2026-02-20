build-treasury:
	stellar contract build --features testnet --package treasury_contract

build-group:
	stellar contract build --features testnet --package group_contract

deploy-treasury: build-treasury
	stellar contract deploy \
	  --wasm target/wasm32v1-none/release/treasury_contract.wasm \
	  --source-account mate \
	  --network testnet \
	  --alias treasury

test:
	cargo test
clean:
	cargo clean