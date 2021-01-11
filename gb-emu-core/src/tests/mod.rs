#![cfg(test)]

use super::cartridge::{Cartridge, CartridgeError};
use super::cpu::{Cpu, CpuRegisters, CpuState};
use super::memory::Bus;

use std::path::Path;

macro_rules! gb_tests {
    // clock until infinite loop
    (inf; $($test_name: ident, $file_path: expr, $crc_checksome: expr;)*) => {
        gb_tests!($($test_name, $file_path, $crc_checksome;)*, clock_until_infinte_loop);
    };

    // clock until breakpoint
    (brk; $($test_name: ident, $file_path: expr, $crc_checksome: expr;)*) => {
        gb_tests!($($test_name, $file_path, $crc_checksome;)*, clock_until_breakpoint);
    };

    ($($test_name: ident, $file_path: expr, $crc_checksome: expr;)*, $looping_statement: tt) => {
        $(
            /// Run the test and check the checksum of the screen buffer
            #[test]
            fn $test_name() {
                let mut gb = crate::tests::TestingGameBoy::new(
                    concat!("../test_roms/", $file_path)
                ).unwrap();

                gb.$looping_statement();

                let screen_buffer = gb.raw_screen_buffer();
                gb.print_screen_buffer();

                assert_eq!(
                    crc::crc64::checksum_ecma(screen_buffer),
                    $crc_checksome
                );
            }
        )*
    };
}

// defined after the macro so that it can use it
mod acid2_test;
mod blargg_tests;
mod mooneye_tests;
mod samesuite_tests;
mod scribbltests;

struct TestingGameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl TestingGameBoy {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;

        Ok(Self {
            bus: Bus::new_without_boot_rom(cartridge),
            cpu: Cpu::new_without_boot_rom(),
        })
    }

    pub fn raw_screen_buffer(&self) -> &[u8] {
        self.bus.raw_screen_buffer()
    }

    pub fn print_screen_buffer(&self) {
        let buffer = self.raw_screen_buffer();

        const TV_WIDTH: u32 = 160;
        const TV_HEIGHT: u32 = 144;

        const BRIGHTNESS_ASCII: [char; 10] = ['@', '%', '#', '*', '+', '=', '-', ':', '.', ' '];

        let mut i = 0;
        let mut j = 0;
        for pixel in buffer.chunks(3) {
            // we shouldn't go beyond the limit
            assert_ne!(j, TV_HEIGHT);

            let r = pixel[0] as f32;
            let g = pixel[0] as f32;
            let b = pixel[0] as f32;
            let brightness = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let brightness_index = (brightness / (31.0 / 9.0)).round() as usize;

            print!("{}", BRIGHTNESS_ASCII[brightness_index]);

            i += 1;
            if i == TV_WIDTH {
                j += 1;
                i = 0;

                println!();
            }
        }

        println!();
    }

    pub fn clock_until_infinte_loop(&mut self) {
        while self.cpu.next_instruction(&mut self.bus) != CpuState::InfiniteLoop {}
    }

    pub fn clock_until_breakpoint(&mut self) -> CpuRegisters {
        loop {
            if let CpuState::Breakpoint(regs) = self.cpu.next_instruction(&mut self.bus) {
                return regs;
            }
        }
    }
}
