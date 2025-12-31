.PHONY: motor_controller test

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