.PHONY: motor_controller test wasm

run:
	cd libraries && cargo run --bin robot

web_interface:
	cd web_interface && npm install && npm run dev

wasm:
	wasm-pack build --target bundler --out-dir ../../web_interface/my-app/src/wasm libraries/packet_wasm

motor_controller:
	cd motor_controller && cargo run --release

test:
	cd libraries && cargo test

clippy:
	cd libraries && cargo clippy
	cd motor_controller && cargo clippy

fmt:
	cd libraries && cargo fmt
	cd motor_controller && cargo fmt

fmt-check:
	cd libraries && cargo fmt -- --check
	cd motor_controller && cargo fmt -- --check