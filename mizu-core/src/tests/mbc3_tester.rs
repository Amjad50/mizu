#[test]
fn mbc3_tester() {
    for &(is_dmg, crc) in &[
        (true, 11138354573804784769),
        (false, 11138354573804784769), // cgb should fail, but with this screen
    ] {
        let mut gb = crate::tests::TestingGameBoy::new(
            "../test_roms/game-boy-test-roms/mbc3-tester/mbc3-tester.gb",
            is_dmg,
        )
        .unwrap();

        for _ in 0..70 {
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
