.PHONY: motor_controller test

motor_controller:
	cd motor_controller && cargo build --release

test:
	cd libraries && cargo test

clippy:
	cd libraries && cargo clippy
	cd motor_controller && cargo clippy