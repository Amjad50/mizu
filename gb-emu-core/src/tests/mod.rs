#![cfg(test)]

mod error;

use super::cartridge::{Cartridge, CartridgeError};
use super::cpu::{Cpu, CpuBusProvider, CpuState};
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
            fn $test_name() -> Result<(), crate::tests::error::TestError> {
                let mut gb = crate::tests::TestingGameBoy::new(
                    concat!("../test_roms/", $file_path)
                )?;

                gb.$looping_statement();

                let screen_buffer = gb.screen_buffer();
                crate::tests::print_screen_buffer(&screen_buffer);

                assert_eq!(
                    crc::crc64::checksum_ecma(&screen_buffer),
                    $crc_checksome
                );

                Ok(())
            }
        )*
    };
}

// defined after the macro so that it can use it
mod acid2_test;
mod blargg_tests;

fn print_screen_buffer(buffer: &[u8]) {
    const TV_WIDTH: u32 = 160;
    const TV_HEIGHT: u32 = 144;

    for i in 0..TV_HEIGHT as usize {
        for j in 0..TV_WIDTH as usize {
            print!(
                "{}",
                if buffer[i * TV_WIDTH as usize + j] == 0 {
                    0
                } else {
                    1
                }
            )
        }
        println!()
    }
}

struct TestingGameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl TestingGameBoy {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;

        Ok(Self {
            bus: Bus::new(cartridge),
            cpu: Cpu::new(),
        })
    }

    pub fn screen_buffer(&self) -> Vec<u8> {
        self.bus.screen_buffer()
    }

    pub fn clock_until_infinte_loop(&mut self) {
        while self.cpu.next_instruction(&mut self.bus) != CpuState::InfiniteLoop {}
    }

    pub fn clock_until_breakpoint(&mut self) {
        loop {
            if let CpuState::Breakpoint(_) = self.cpu.next_instruction(&mut self.bus) {
                break;
            }
        }
    }
}
