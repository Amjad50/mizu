#![cfg(test)]

use super::cartridge::{Cartridge, CartridgeError};
use super::cpu::{Cpu, CpuRegisters, CpuState};
use super::memory::Bus;
use super::GameboyConfig;

use std::path::Path;

macro_rules! gb_tests {
    // clock until infinite loop
    (inf; $($test_name: ident $(for $emu: ident)?, $file_path: expr, $dmg_crc: expr, $cgb_crc: expr;)*) => {
        gb_tests!($($test_name $(for $emu)?, $file_path, $dmg_crc, $cgb_crc;)*, clock_until_infinte_loop);
    };

    // clock until breakpoint
    (brk; $($test_name: ident $(for $emu: ident)?, $file_path: expr, $dmg_crc: expr, $cgb_crc: expr;)*) => {
        gb_tests!($($test_name $(for $emu)?, $file_path, $dmg_crc, $cgb_crc;)*, clock_until_breakpoint);
    };

    ($($test_name: ident $(for $emu: ident)?, $file_path: expr, $dmg_crc: expr, $cgb_crc: expr;)*, $looping_statement: tt) => {
        $(
            /// Run the test and check the checksum of the screen buffer
            #[test]
            #[allow(unused_mut)]
            fn $test_name() {
                // inner tester to test DMG and CGB separately
                fn test(file_path: &str, is_dmg: bool, crc_checksum: u64) {
                    let mut gb = crate::tests::TestingGameBoy::new(file_path, is_dmg).unwrap();

                    gb.$looping_statement();

                    let screen_buffer = gb.raw_screen_buffer();
                    gb.print_screen_buffer();

                    assert_eq!(crc::Crc::<u64>::new(&crc::CRC_64_XZ).checksum(screen_buffer), crc_checksum);
                }


                let file_path = concat!("../test_roms/", $file_path);

                let mut emu = String::new();
                $(emu += stringify!($emu);)?

                assert!(emu == "" || emu == "dmg" || emu == "cgb",
                    "emu parameter can only be \"dmg\" or \"cgb\"");

                let is_dmg = true && emu != "cgb";
                let is_cgb = true && emu != "dmg";

                if is_dmg {
                    test(file_path, true, $dmg_crc);
                }
                if is_cgb {
                    test(file_path, false, $cgb_crc);
                }
            }
        )*
    };
}

// defined after the macro so that it can use it
mod acid2_test;
mod blargg_tests;
mod mooneye_tests;
mod rtc3;
mod samesuite_tests;
mod save_state_tests;
mod scribbltests;

#[derive(save_state::Savable)]
struct TestingGameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl TestingGameBoy {
    pub fn new<P: AsRef<Path>>(file_path: P, is_dmg: bool) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file::<_, String>(file_path, None, false)?;

        let config = GameboyConfig { is_dmg };

        let is_cartridge_color = cartridge.is_cartridge_color();
        Ok(Self {
            bus: Bus::new_without_boot_rom(cartridge, config),
            cpu: Cpu::new_without_boot_rom(config, is_cartridge_color),
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

    pub fn clock_for_frame(&mut self) {
        const PPU_CYCLES_PER_FRAME: u32 = 456 * 154;
        let mut cycles = 0u32;
        while cycles < PPU_CYCLES_PER_FRAME {
            self.cpu.next_instruction(&mut self.bus);
            cycles += self.bus.elapsed_ppu_cycles() as u32;
        }
    }
}
