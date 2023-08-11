gb_tests!(
    inf; // clock until infinite loop

    // FIXME: the test passes but the screen is not correct
    //  the label for test 11 does not show "OK"
    blargg_cpu_instrs,
    "blargg-gb-tests/cpu_instrs/cpu_instrs.gb",
    16599394073517471602,
    3892107528677358037;

    blargg_instr_timing,
    "blargg-gb-tests/instr_timing/instr_timing.gb",
    14586804626949338345,
    14586804626949338345;

    blargg_halt_bug,
    "blargg-gb-tests/halt_bug.gb",
    3778129031474618196,
    3778129031474618196;

    blargg_mem_timing_2,
    "blargg-gb-tests/mem_timing-2/mem_timing.gb",
    12164226896603567743,
    12164226896603567743;

    blargg_dmg_sound_all for dmg,
    "blargg-gb-tests/dmg_sound/dmg_sound.gb",
    9608420910100250529,
    0; // cannot test on CGB as it goes into `STOP` for some reason

    blargg_cgb_sound_all,
    "blargg-gb-tests/cgb_sound/cgb_sound.gb",
    18396380547272095665, // some tests only should fail in DMG (check which are failing)
    4141669196667164762;

    // FIXME: the test passes but the screen is not correct
    //  it shows the numbers, but doesn't print "Passed"
    blargg_interrupt_time for cgb,
    "blargg-gb-tests/interrupt_time/interrupt_time.gb",
    0, // this test is designed for cgb
    3220739068587521835;
);

#[test]
fn blargg_oam_bug_all() {
    for &(is_dmg, crc) in &[
        (true, 15533008004237088224u64),
        (false, 2687058989347279874), // cgb should fail, but with this screen
    ] {
        let mut gb = crate::tests::TestingGameBoy::new(
            "../test_roms/blargg-gb-tests/oam_bug/oam_bug.gb",
            is_dmg,
        )
        .unwrap();

        gb.clock_until_infinte_loop();

        // When the infinite loop is reached the display is still blank so wait for
        // a bit before taking a screenshot
        for _ in 0..10 {
            gb.clock_for_frame();
        }

        let screen_buffer = gb.raw_screen_buffer();
        gb.print_screen_buffer();

        assert_eq!(
            crc::Crc::<u64>::new(&crc::CRC_64_XZ).checksum(screen_buffer),
            crc
        );
    }
}
