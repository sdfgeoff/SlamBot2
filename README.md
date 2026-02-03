# SlamBot2

SlamBot2 is a multi-component robotics stack that ties together a Rust robot runtime, an embedded motor controller firmware, and a React web interface. The system uses a shared packet schema and a topic-based router so messages can flow between the robot nodes, the web UI, and the embedded controller using a consistent, versioned data model.

## 🧭 Architecture at a glance

```
Web UI (React + WASM codec)
        │  WebSocket (CBOR/COBS packets)
        ▼
Robot runtime (libraries/robot)
  ├─ packet_router: topic/direct-address routing
  ├─ nodes: clock, log, position_estimator, motion_controller
  ├─ serial_adapter: scans and connects to motor controller
  ▼
Motor controller firmware (ESP32-C3)
  └─ encoders + motor drivers + odometry + diagnostics
```

Key idea: **one packet format everywhere**. The Rust `topics` crate defines message types. The Rust `packet_encoding` crate encodes/decodes them (CBOR + CRC16 + COBS). The `packet_wasm` crate exposes the same codec to the web app via WebAssembly.

## 📦 Repository layout

- `libraries/` – Rust workspace with shared crates:
  - `topics` – packet schemas (`PacketData`, `PacketFormat`) and diagnostic/ROS helpers.
  - `packet_trait` – trait for routing and addressing.
  - `packet_router` – in-process topic router for clients/nodes.
  - `packet_encoding` – CBOR + CRC16 + COBS codec and framing.
  - `packet_wasm` – WASM wrapper around the Rust codec.
  - `robot` – desktop runtime that wires nodes together and bridges serial/WebSocket.
- `motor_controller/` – embedded firmware for an ESP32-C3 motor controller.
- `web_interface/my-app/` – React + Vite UI that speaks the same packet protocol.
- `Makefile` – convenience commands to run/build/test key components.

## 🧩 Core concepts

### Packet schema (`topics`)
The `topics` crate defines all message types (clock sync, odometry, motion commands, diagnostics, etc.) and wraps them in:

- `PacketData` – enum of all message variants.
- `PacketFormat<T>` – message envelope (to/from/id/time + data).

### Codec (`packet_encoding`)
All packets are encoded using:

$$\text{COBS}(\text{CBOR}(\text{message}) + \text{CRC16})$$

This framing enables a single 0x00 delimiter for packet boundaries and a CRC for integrity. `PacketFinder` helps reconstruct packets from a byte stream.

### Router (`packet_router`)
The robot runtime uses an in-process router that delivers packets:

- by **topic subscription** (pub-sub), or
- by **explicit address** (direct delivery).

The router is shared across nodes via `Rc<RefCell<Router<PacketFormat<PacketData>>>>`.

### Web codec (`packet_wasm`)
The `packet_wasm` crate exposes `encode_packet` / `decode_packet` to JavaScript. This keeps the web UI in sync with the Rust packet format without a separate TypeScript encoder.

## 🏗 Runtime components

### Robot runtime (`libraries/robot`)
A single-threaded loop wires everything together:

- `Clock` – local time sync and clock requests.
- `Log` – emits diagnostic packets.
- `PositionEstimator` – estimates robot pose from odometry.
- `MotionController` – converts targets into velocity requests.
- `SerialAdapter` – discovers and bridges motor controller via serial.
- `WebsocketAcceptor` – WebSocket server (default `127.0.0.1:9001`) for the web UI.

### Motor controller firmware (`motor_controller`)
Runs on an ESP32-C3 and handles:

- wheel encoders and odometry integration
- motor PWM control via LEDC
- host connection over USB JTAG serial
- `MotionVelocityRequest` handling and `OdometryDelta` publishing
- periodic clock sync + diagnostics

### Web interface (`web_interface/my-app`)
A React UI that:

- connects to the robot runtime over WebSocket
- uses the WASM codec for packet encode/decode
- visualizes state and issues motion/target requests

## 🧪 Development workflow

### Requirements (from repo config)
- Rust stable toolchain (see `libraries/rust-toolchain.toml`).
- `wasm-pack` for building the web codec.
- Node.js + npm for the web interface.
- Embedded target for the motor controller (`riscv32imc-unknown-none-elf`).

### Common tasks (from `Makefile`)

- Run robot runtime: `make run`
- Build WASM codec: `make wasm`
- Start web UI (builds WASM first): `make web_interface`
- Run motor controller firmware: `make motor_controller`
- Run Rust tests: `make test`
- Format/lint: `make fmt`, `make clippy`

## ❓Questions / context needed

To make this README even more accurate, could you clarify:

1. What exact hardware platform is SlamBot2 targeting (motor controller board + sensors)?
2. How is the robot runtime deployed (local dev machine, SBC on the robot, etc.)?
3. Are there any additional services (e.g., SLAM stack, map server) not represented in this repo?

Once I have that, I can refine the architecture section and add deployment instructions.
