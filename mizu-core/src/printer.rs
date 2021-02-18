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
    /// data packet, otherwise the print command will be ignored
    ready_to_print_next: bool,
    printing_delay: u8,
    received_bit_counter: u8,

    /// dynamically sized image buffer that will increase on each row print,
    /// trying to simulate the paper that the gameboy printer used.
    image_buffer: Vec<u8>,
    image_size: (u32, u32),
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
            image_buffer: Vec::new(),
            image_size: (0, 0),
        }
    }
}

impl Printer {
    pub fn get_image_buffer(&self) -> &[u8] {
        &self.image_buffer
    }

    pub fn get_image_size(&self) -> (u32, u32) {
        self.image_size
    }

    pub fn clear_image_buffer(&mut self) {
        self.image_buffer.clear();
        self.image_size = (0, 0);
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

                        let number_of_sheets = self.current_packet.data[0];
                        let margins = self.current_packet.data[1];
                        let palette = self.current_packet.data[2];
                        let exposure = self.current_packet.data[3];

                        self.print(
                            number_of_sheets,
                            margins,
                            palette,
                            exposure,
                            self.ram_next_write_pointer,
                        );
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
    fn print(
        &mut self,
        number_of_sheets: u8,
        margins: u8,
        palette: u8,
        exposure: u8,
        max_data_len: usize,
    ) {
        // line feed only
        if number_of_sheets == 0 {
            self.print_line_feed();
            return;
        }

        // high nibble
        let margin_before = margins >> 4;
        // low nibble
        let margin_after = margins & 0xF;

        let exposure_multiply = compute_exposure_multiply(exposure);

        // TODO: check if margin count is in pixel units or not, because its
        //  a bit small, maximum of 15 pixels.
        for _ in 0..margin_before {
            self.print_line_feed();
        }

        let rows_to_print = max_data_len / 40;

        let (_, old_height) = self.image_size;
        let new_width = 160;
        let new_height = old_height + rows_to_print as u32;

        self.image_size = (new_width, new_height);

        // reserve space for the rows
        let old_size = self.image_buffer.len();
        let extra_space = rows_to_print * 160 * 3;
        self.image_buffer.reserve(extra_space);

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

                    // we use inverted gray shade, because white should not be
                    // changed due to exposure, only black does
                    let inverted_gray_shade = 85 * color;
                    // apply exposure, the multiply value ranges are (0.75 ~ 1.25)
                    let exposured_invertd_gray_shade =
                        inverted_gray_shade as f64 * exposure_multiply;
                    // lastly, just make sure the values do not exceed 255 and
                    // not negative
                    let exposured_invertd_gray_shade =
                        exposured_invertd_gray_shade.min(255.).max(0.);

                    // flip to convert to normal gray shade (255 white, 0 black)
                    let gray_shade = 255 - (exposured_invertd_gray_shade as u8);

                    // RGB
                    for _ in 0..3 {
                        self.image_buffer.push(gray_shade);
                    }
                }
            }
        }

        // we should not exceed the space we have
        assert_eq!(old_size + extra_space, self.image_buffer.len());

        for _ in 0..margin_after {
            self.print_line_feed();
        }

        if number_of_sheets > 1 {
            // recursively print the next sheet if there is more than one
            self.print(
                number_of_sheets - 1,
                margins,
                palette,
                exposure,
                max_data_len,
            );
        }
    }

    /// prints one row of pixels
    fn print_line_feed(&mut self) {
        let (_, old_height) = self.image_size;
        // add one line
        self.image_size = (160, old_height + 1);

        // add one row of white
        self.image_buffer.reserve(160 * 3);

        for _ in 0..(160 * 3) {
            self.image_buffer.push(255);
        }
    }
}

// util function to map between two number ranges
fn map_num(inp: i32, inp_start: i32, inp_end: i32, out_start: i32, out_end: i32) -> i32 {
    ((inp - inp_start) as f64 / (inp_end - inp_start) as f64 * (out_end - out_start) as f64
        + out_start as f64) as i32
}

/// maps the 7 bits from exposure to (75% ~ 125%) which is equivalent to
/// an increase by the range (-25% ~ 25%)
fn compute_exposure_multiply(exposure: u8) -> f64 {
    (100 + map_num((exposure & 0x7F) as i32, 0, 0x7F, -25, 25)) as f64 / 100.
}

#[test]
fn map_num_test() {
    let a = map_num(5, 0, 100, 1, 11);
    assert_eq!(a, 1);
}

#[test]
fn compute_exposure_multiply_test() {
    let mut min = 200f64;
    let mut max = -200f64;

    for exposure in 0..=0x7F {
        let a = compute_exposure_multiply(exposure);

        // a should always increase, the first time we are just setting the numbers
        // so we cannot compare in the first time
        if exposure != 0 {
            assert!(a >= min);
            assert!(a >= max);
        }

        min = min.min(a);
        max = max.max(a);
    }

    // these are the possible ranges of exposure
    assert_eq!(min, 0.75);
    assert_eq!(max, 1.25);
}

impl SerialDevice for Printer {
    fn exchange_bit_external_clock(&mut self, bit: bool) -> bool {
        self.received_bit_counter += 1;

        if self.received_bit_counter == 9 {
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
