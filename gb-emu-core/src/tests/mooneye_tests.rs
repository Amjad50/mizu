macro_rules! mooneye_tests {
    ($prefix:expr; $($test_name: ident $(- $suffix_name:ident)? $(,)?),*) => {
        $(
            /// Run the test and check registers values (take from mooneye)
            #[test]
            fn $test_name() {
                let file_path = concat!(
                    "../test_roms/mooneye-gb_hwtests/",
                    $prefix, "/",
                    stringify!($test_name), $('-', stringify!($suffix_name),)? ".gb");

                let mut gb = crate::tests::TestingGameBoy::new(
                    file_path
                ).unwrap();

                let regs = gb.clock_until_breakpoint();

                let screen_buffer = gb.screen_buffer();
                crate::tests::print_screen_buffer(screen_buffer);

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
    mooneye_tests!("emulator-only/mbc1";
        bits_bank1, bits_ramg, ram_64kb, rom_2Mb, rom_8Mb, bits_bank2,
        // multicart_rom_8Mb,
        rom_16Mb, rom_4Mb, bits_mode, ram_256kb, rom_1Mb, rom_512kb
    );
}

#[allow(non_snake_case)]
mod mbc2 {
    mooneye_tests!("emulator-only/mbc2";
        bits_romb,
        bits_ramg,
        bits_unused,
        ram,
        rom_1Mb,
        rom_2Mb,
        rom_512kb
    );
}

#[allow(non_snake_case)]
mod acceptance {
    mooneye_tests!("acceptance";
        add_sp_e_timing,
        boot_div-dmgABCmgb,
        boot_hwio-dmgABCmgb,
        boot_regs-dmgABC,
        call_cc_timing2,
        call_cc_timing,
        call_timing2,
        call_timing,
        di_timing-GS,
        div_timing,
        ei_sequence,
        ei_timing,
        halt_ime0_ei,
        halt_ime0_nointr_timing,
        halt_ime1_timing2-GS,
        halt_ime1_timing,
        if_ie_registers,
        intr_timing,
        jp_cc_timing,
        jp_timing,
        ld_hl_sp_e_timing,
        oam_dma_restart,
        oam_dma_start,
        oam_dma_timing,
        pop_timing,
        push_timing,
        rapid_di_ei,
        ret_cc_timing,
        reti_intr_timing,
        reti_timing,
        ret_timing,
        rst_timing
    );

    mod bits {
        mooneye_tests!("acceptance/bits"; mem_oam, reg_f, unused_hwio-GS);
    }

    mod instr {
        mooneye_tests!("acceptance/instr"; daa);
    }

    mod interrupts {
        // mooneye_tests!("acceptance/interrupts"; ie_push);
    }

    mod oam_dma {
        mooneye_tests!("acceptance/oam_dma";
            basic, reg_read,
            //sources-GS
        );
    }

    mod ppu {
        mooneye_tests!("acceptance/ppu";
            //hblank_ly_scx_timing-GS,
            //intr_1_2_timing-GS,
            //intr_2_0_timing,
            //intr_2_mode0_timing,
            //intr_2_mode0_timing_sprites,
            //intr_2_mode3_timing,
            //intr_2_oam_ok_timing,
            //lcdon_timing-GS,
            //lcdon_write_timing-GS,
            stat_irq_blocking,
            stat_lyc_onoff,
            vblank_stat_intr-GS,
        );
    }

    mod timer {
        mooneye_tests!("acceptance/timer";
            div_write,
            rapid_toggle,
            tim00_div_trigger,
            tim00,
            tim01_div_trigger,
            tim01,
            tim10_div_trigger,
            tim10,
            tim11_div_trigger,
            tim11,
            tima_reload,
            tima_write_reloading,
            tma_write_reloading
        );
    }
}
