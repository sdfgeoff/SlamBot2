# Packet Encoding

A `no_std` compatible Rust crate for robust packet encoding and decoding using a layered approach with CBOR serialization, CRC16 checksums, and COBS (Consistent Overhead Byte Stuffing) framing.

## Overview

This crate provides reliable packet communication by combining three encoding layers:

1. **CBOR Serialization** - Converts Rust structs to/from compact binary format using `serde_cbor`
2. **CRC16 Checksums** - Adds data integrity verification using ARC polynomial
3. **COBS Framing** - Provides packet boundary detection with consistent overhead byte stuffing

The encoding format is: `COBS(CBOR(MESSAGE) + CRC16(CBOR(MESSAGE)))`

## Features

- **`no_std` and `std` compatible** - Works in embedded environments without heap allocation
- **Serde integration** - Automatic serialization for any type implementing `Serialize`/`Deserialize`
- **Packet finder** - Stream-based packet detection for serial communication, looks for 0x00 byte leading and trailing bytes
