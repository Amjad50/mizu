/// We load the acid test, run it until the test finish, and we make sure it passes
/// then we save the state, create a new emulation and load the state, the test
/// should be passing
#[test]
fn load_state() {
    // 0- perform normal test (this part should always pass)
    const CGB_CRC: u64 = 4378550468433865064;
    let file_path = "../test_roms/cgb-acid2.gbc";

    // 1- make sure after start and advancing 2 frames does not pass the test
    let mut gb = crate::tests::TestingGameBoy::new(file_path, false).unwrap();
    gb.clock_for_frame();
    gb.clock_for_frame();
    let screen_buffer = gb.raw_screen_buffer();
    assert_ne!(crc::crc64::checksum_ecma(screen_buffer), CGB_CRC);

    gb = crate::tests::TestingGameBoy::new(file_path, false).unwrap();
    gb.clock_until_breakpoint();
    let screen_buffer = gb.raw_screen_buffer();
    assert_eq!(crc::crc64::checksum_ecma(screen_buffer), CGB_CRC);

    // 2- save the state at which it was passing
    let saved_data = save_state::save_object(&gb).unwrap();

    // 3- create a new object and load the state
    gb = crate::tests::TestingGameBoy::new(file_path, false).unwrap();
    save_state::load_object(&mut gb, &saved_data).unwrap();

    // 4- the image should be the passing state (needs to advance two frames)
    gb.clock_for_frame();
    gb.clock_for_frame();
    let screen_buffer = gb.raw_screen_buffer();
    assert_eq!(crc::crc64::checksum_ecma(screen_buffer), CGB_CRC);
}
