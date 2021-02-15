use crate::serial::SerialDevice;
use bitflags::bitflags;

bitflags! {
    struct PrinterStatus : u8 {
        const LOW_BATTERY     = 1 << 7;
        const OTHER_ERR       = 1 << 6;
        const PAPER_JAM       = 1 << 5;
        const PACKET_ERR      = 1 << 4;
        const READY_TO_PRINT  = 1 << 3;
        const IMAGE_DATA_FULL = 1 << 2;
        const PRINTING        = 1 << 1;
        const CHECKSUM_ERR    = 1;
    }
}

enum PacketState {
    Magic1,
    Magic2,
    Command,
    CompressionFlag,
    DataLengthLow,
    DataLengthHigh,
    CommandData,
    ChecksumLow,
    ChecksumHigh,
    AliveIndicator,
    Status,
}

#[derive(Default, Debug)]
struct Packet {
    command: u8,
    compression_flag: u8,
    data_length: u16,
    data: Vec<u8>,
    checksum: u16,
}

impl Packet {
    fn clear(&mut self) {
        let _ = std::mem::take(self);
    }

    fn compute_checksum(&self) -> u16 {
        let mut sum = 0;
        sum += self.command as u16;
        sum += self.compression_flag as u16;
        sum += self.data_length & 0xFF;
        sum += self.data_length >> 8;
        sum += self
            .data
            .iter()
            .map(|&x| x as u16)
            .fold(0u16, |a, b| a.wrapping_add(b));

        sum
    }
}

pub struct Printer {
    ram: [u8; 0x2000],
    ram_next_write_pointer: usize,
    current_packet: Packet,
    packet_input_state: PacketState,
    remaining_data_length: u16,
    byte_to_send: u8,
    received_byte: u8,
    status: PrinterStatus,
    /// In order to print, after sending data packets, the GB must send an empty
    /// data packet, otherwize the print command will be ignored
    ready_to_print_next: bool,
    printing_delay: u8,
    received_bit_counter: u8,
}

impl Default for Printer {
    fn default() -> Self {
        Self {
            ram: [0; 0x2000],
            ram_next_write_pointer: 0,
            current_packet: Packet::default(),
            packet_input_state: PacketState::Magic1,
            remaining_data_length: 0,
            byte_to_send: 0,
            received_byte: 0,
            status: PrinterStatus::empty(),
            ready_to_print_next: false,
            printing_delay: 0,
            received_bit_counter: 0,
        }
    }
}

impl Printer {
    fn handle_next_byte(&mut self, byte: u8) {
        match self.packet_input_state {
            PacketState::Magic1 => {
                if byte == 0x88 {
                    self.packet_input_state = PacketState::Magic2;
                    self.current_packet.clear();
                }
            }
            PacketState::Magic2 => {
                if byte == 0x33 {
                    self.packet_input_state = PacketState::Command;
                } else {
                    self.packet_input_state = PacketState::Magic1;
                }
            }
            PacketState::Command => {
                self.current_packet.command = byte;
                self.packet_input_state = PacketState::CompressionFlag;
            }
            PacketState::CompressionFlag => {
                self.current_packet.compression_flag = byte;
                self.packet_input_state = PacketState::DataLengthLow;
            }
            PacketState::DataLengthLow => {
                self.current_packet.data_length &= 0xFF00;
                self.current_packet.data_length |= byte as u16;
                self.packet_input_state = PacketState::DataLengthHigh;
            }
            PacketState::DataLengthHigh => {
                self.current_packet.data_length &= 0x00FF;
                self.current_packet.data_length |= (byte as u16) << 8;

                self.remaining_data_length = self.current_packet.data_length;
                self.packet_input_state = if self.remaining_data_length != 0 {
                    PacketState::CommandData
                } else {
                    PacketState::ChecksumLow
                };
            }
            PacketState::CommandData => {
                self.current_packet.data.push(byte);

                self.remaining_data_length -= 1;

                if self.remaining_data_length == 0 {
                    self.packet_input_state = PacketState::ChecksumLow;
                }
            }
            PacketState::ChecksumLow => {
                self.current_packet.checksum &= 0xFF00;
                self.current_packet.checksum |= byte as u16;
                self.packet_input_state = PacketState::ChecksumHigh;
            }

            PacketState::ChecksumHigh => {
                self.current_packet.checksum &= 0x00FF;
                self.current_packet.checksum |= (byte as u16) << 8;
                self.packet_input_state = PacketState::AliveIndicator;

                // alive
                self.byte_to_send = 0x81;
            }
            PacketState::AliveIndicator => {
                self.packet_input_state = PacketState::Status;

                self.byte_to_send = self.status.bits();
                self.process_packet();
            }
            PacketState::Status => {
                // go back to the beginning
                self.packet_input_state = PacketState::Magic1;
            }
        }
    }

    fn process_packet(&mut self) {
        if self.current_packet.checksum != self.current_packet.compute_checksum() {
            self.status |= PrinterStatus::CHECKSUM_ERR;
            panic!("checksum failed");
        }

        match self.current_packet.command {
            1 => {
                self.ram = [0; 0x2000];
                self.ram_next_write_pointer = 0;
                self.status = PrinterStatus::empty();
                self.ready_to_print_next = false;
            }
            2 => {
                if self.ready_to_print_next {
                    if self.current_packet.data_length != 4 {
                        self.status |= PrinterStatus::PACKET_ERR;
                    } else {
                        // print done
                        self.status |= PrinterStatus::PRINTING;
                        self.status.remove(PrinterStatus::READY_TO_PRINT);
                        self.printing_delay = 20;
                        self.print(&self.current_packet.data, self.ram_next_write_pointer);
                    }
                    self.ready_to_print_next = false;
                }
            }
            4 => {
                if self.current_packet.data_length == 0 {
                    self.ready_to_print_next = true;
                } else {
                    let start = self.ram_next_write_pointer;
                    let end = start + self.current_packet.data_length as usize;
                    self.ram_next_write_pointer = end;
                    if end > self.ram.len() {
                        // Should a flag be specified here?
                        panic!("end is bigger than ram size");
                    }

                    assert_eq!(
                        self.current_packet.data.len(),
                        self.current_packet.data_length as usize
                    );
                    self.ram[start..end].copy_from_slice(&self.current_packet.data);
                }

                self.status |= PrinterStatus::READY_TO_PRINT
            }
            0xF => {
                if self.status.contains(PrinterStatus::PRINTING) {
                    self.printing_delay = self.printing_delay.saturating_sub(1);
                    if self.printing_delay == 0 {
                        self.status.remove(PrinterStatus::PRINTING);
                    }
                }
            }
            _ => {
                self.status |= PrinterStatus::PACKET_ERR;
            }
        };
    }

    /// The data in ram are stored as normal tiles, every 16 bytes form one tile
    /// (8x8). Every 16 * 20 bytes form one tile row (160x8).
    fn print(&self, params: &[u8], max_data_len: usize) {
        const BRIGHTNESS: [char; 4] = [' ', '.', '-', '#'];

        assert_eq!(params.len(), 4);

        // FIXME: use these parameters
        let _number_of_sheets = params[0];
        let _margin = params[1];
        let palette = params[2];
        let _exposure = params[3];

        let rows_to_print = max_data_len / 40;

        for y in 0..rows_to_print {
            for x in 0..20 {
                let scroll_y = y / 8;
                let fine_y_scroll = y % 8;

                let tile = scroll_y * 20 + x;
                let tile_index = tile * 16;
                let ram_index = tile_index + (fine_y_scroll * 2);

                let low = self.ram[ram_index];
                let high = self.ram[ram_index + 1];

                let mut result = [0; 8];
                for (i, result_item) in result.iter_mut().enumerate() {
                    let bin_i = 7 - i;
                    *result_item = ((high >> bin_i) & 1) << 1 | ((low >> bin_i) & 1);
                }

                for pixel in &result {
                    let color = (palette >> (pixel * 2)) & 0b11;

                    // TODO: render to a screen or an image buffer to be used
                    //  by the front-end
                    print!("{}", BRIGHTNESS[color as usize]);
                }
            }
            println!();
        }
    }
}

impl SerialDevice for Printer {
    fn exchange_bit_external_clock(&mut self, bit: bool) -> bool {
        self.received_bit_counter += 1;

        if self.received_bit_counter == 9 {
            //println!("got byte {:02X}", self.received_byte);
            self.handle_next_byte(self.received_byte);
            self.received_byte = 0;
            self.received_bit_counter = 1;
        }

        self.received_byte = self.received_byte.wrapping_shl(1);
        self.received_byte |= bit as u8;

        let out = self.byte_to_send & 0x80 != 0;
        self.byte_to_send = self.byte_to_send.wrapping_shl(1);

        out
    }
}
