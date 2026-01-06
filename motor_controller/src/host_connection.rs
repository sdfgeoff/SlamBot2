use crate::Clock;
use esp_hal::peripherals::USB_DEVICE;
use esp_hal::time::{Duration, Instant};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use packet_encoding::encode_packet;
use topics::{PacketData, PacketFormat};

#[derive(Debug)]
pub enum SendError {
    EncodeError,
    Timeout,
    ReadWouldBlock,
}

/**
 * The UsbSerialJtagTx API doesnt have a nice non-blocking write. I couldn't get the performance I wanted.
 * So this implementation reimplements a non-blocking write with timeout using direct register access.
 */
pub struct NonBlockingJtagUart<'a> {
    peripheral: USB_DEVICE<'a>,
    write_timeout: Duration,
}

impl<'a> NonBlockingJtagUart<'a> {
    pub fn new(peripheral: USB_DEVICE<'a>, write_timeout: Duration) -> Self {
        // Set up the peripheral using UsbSerialJtag internal function
        let _ = UsbSerialJtag::new(unsafe { peripheral.clone_unchecked() });

        NonBlockingJtagUart {
            peripheral,
            write_timeout,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), SendError> {
        let regs = self.peripheral.register_block();
        let now = Instant::now();
        for chunk in data.chunks(64) {
            for byte in chunk {
                regs.ep1().write(|w| unsafe { w.rdwr_byte().bits(*byte) });
            }
            regs.ep1_conf().modify(|_, w| w.wr_done().set_bit());

            // FIXME: raw register access
            while regs.ep1_conf().read().bits() & 0b011 == 0b000 {
                if now.elapsed() >= self.write_timeout {
                    return Err(SendError::Timeout);
                }
            }
        }

        Ok(())
    }

    /// Read a byte from the UART in a non-blocking manner
    pub fn read_byte(&mut self) -> Result<u8, SendError> {
        let regs = self.peripheral.register_block();

        // Check if there are any bytes to read
        if regs
            .ep1_conf()
            .read()
            .serial_out_ep_data_avail()
            .bit_is_set()
        {
            let value = regs.ep1().read().rdwr_byte().bits();

            Ok(value)
        } else {
            Err(SendError::ReadWouldBlock)
        }
    }
}

fn send_message(usb: &mut NonBlockingJtagUart, message: &PacketFormat) -> Result<(), SendError> {
    let mut encode_buffer = [0u8; 600];
    encode_buffer[0] = 0; // COBS initial byte
    let encoded_size =
        encode_packet(message, &mut encode_buffer[1..]).map_err(|_| SendError::EncodeError)?;
    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
    let encode_sized = &encode_buffer[..encoded_size + 2];

    usb.write(encode_sized)
}

pub struct HostConnection<'a> {
    usb: NonBlockingJtagUart<'a>,
    packet_finder: packet_encoding::PacketFinder,
    message_id: u32,
    decode_errors: u32,
}

impl<'a> HostConnection<'a> {
    pub fn new(usb: NonBlockingJtagUart<'a>) -> Self {
        HostConnection {
            usb,
            packet_finder: packet_encoding::PacketFinder::new(),
            message_id: 0,
            decode_errors: 0,
        }
    }

    pub fn send_packet(
        &mut self,
        clock: &Clock,
        data: PacketData,
        to: Option<u16>,
    ) -> Result<(), SendError> {
        let packet = PacketFormat {
            to,
            from: None,
            data,
            time: clock.get_time(),
            id: self.message_id,
        };
        self.message_id = self.message_id.wrapping_add(1);
        send_message(&mut self.usb, &packet)
    }

    pub fn step(&mut self) -> Option<PacketFormat> {
        while let Ok(byte) = self.usb.read_byte() {
            if let Some(mut packet) = self.packet_finder.push_byte(byte)
                && !packet.is_empty()
            {
                if let Ok(packet) = packet_encoding::decode_packet::<PacketFormat>(&mut packet) {
                    return Some(packet);
                } else {
                    self.decode_errors = self.decode_errors.wrapping_add(1);
                }
            }
        }
        None
    }
}
