macro_rules! mooneye_tests {
    ($($test_name: ident, $file_path: expr;)*) => {
        $(
            /// Run the test and check registers values (take from mooneye)
            #[test]
            fn $test_name() {
                let mut gb = crate::tests::TestingGameBoy::new(
                    concat!("../test_roms/mooneye-gb_hwtests/", $file_path)
                ).unwrap();

                let regs = gb.clock_until_breakpoint();

                let screen_buffer = gb.screen_buffer();
                crate::tests::print_screen_buffer(&screen_buffer);

                // These checks are taken from mooneye emulator
                if regs.a != 0 {
                    panic!(
                      "mooneye test failed with A = {}",
                      regs.a
                    );
                }
                if regs.b != 3 || regs.c != 5 || regs.d != 8 || regs.e != 13 || regs.h != 21 || regs.l != 34 {
                    panic!("mooneye test failed regs = {:?}", regs);
                }
            }
        )*
    };
}

#[allow(non_snake_case)]
mod mbc1 {
    macro_rules! mbc1_tests {
        ($($name: ident),*) => {
            mooneye_tests!(
                $(
                $name,
                concat!("emulator-only/mbc1/", stringify!($name), ".gb");
                )*
            );
        };
    }

    mbc1_tests!(
        bits_bank1, bits_ramg, ram_64kb, rom_2Mb, rom_8Mb, bits_bank2,
        // multicart_rom_8Mb,
        rom_16Mb, rom_4Mb, bits_mode, ram_256kb, rom_1Mb, rom_512kb
    );
}

mod acceptance {
    macro_rules! acceptance_tests {
         ($($name: tt $(.$folder:tt)? $(,)?),*) => {
             mooneye_tests!(
                 $(
                 $name,
                 concat!("acceptance/", $(stringify!($folder), "/",)? stringify!($name), ".gb");
                 )*
             );
         };
     }

    acceptance_tests!(
        add_sp_e_timing,
        // boot_div - dmg0,
        // boot_hwio - dmg0,
        // boot_regs - dmg0,
        call_cc_timing2,
        call_cc_timing,
        call_timing2,
        call_timing,
        div_timing,
        //ei_sequence,
        //ei_timing,
        halt_ime0_ei,
        //halt_ime0_nointr_timing,
        halt_ime1_timing,
        if_ie_registers,
        //intr_timing,
        jp_cc_timing,
        jp_timing,
        ld_hl_sp_e_timing,
        oam_dma_restart,
        oam_dma_start,
        oam_dma_timing,
        pop_timing,
        push_timing,
        //rapid_di_ei,
        //ret_cc_timing,
        //reti_intr_timing,
        //reti_timing,
        //ret_timing,
        rst_timing
    );

    mod bits {
        acceptance_tests!(mem_oam.bits, reg_f.bits);
    }

    mod instr {
        acceptance_tests!(daa.instr);
    }

    mod interrupts {
        //acceptance_tests!(ie_push.interrupts);
    }

    mod oam_dma {
        acceptance_tests!(basic.oam_dma, reg_read.oam_dma);
    }

    mod ppu {
        acceptance_tests!(
            //intr_2_0_timing.ppu,
            //intr_2_mode0_timing.ppu,
            //intr_2_mode0_timing_sprites.ppu,
            //intr_2_mode3_timing.ppu,
            //intr_2_oam_ok_timing.ppu,
            //stat_irq_blocking.ppu,
            //stat_lyc_onoff.ppu
        );
    }

    mod timer {
        acceptance_tests!(
            div_write.timer,
            //rapid_toggle.timer,
            //tim00_div_trigger.timer,
            tim00.timer,
            //tim01_div_trigger.timer,
            tim01.timer,
            //tim10_div_trigger.timer,
            tim10.timer,
            //tim11_div_trigger.timer,
            tim11.timer,
            //tima_reload.timer,
            //tima_write_reloading.timer,
            //tma_write_reloading.timer
        );
    }
}
