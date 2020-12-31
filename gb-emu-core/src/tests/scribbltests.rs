use super::TestingGameBoy;

gb_tests!(
    brk; // clock until breakpoint

    lycscx,
    "scribbltests/lycscx/lycscx.gb",
    13365938285444402733;

    lycscy,
    "scribbltests/lycscy/lycscy.gb",
    16456385004234895650;

    // FIXME: this test require 4 color support, in testing we only have 2
    palettely,
    "scribbltests/palettely/palettely.gb",
    4766554184901124846;

    scxly,
    "scribbltests/scxly/scxly.gb",
    222307982374708667;

);

#[test]
#[allow(dead_code)]
fn statcount_auto() {
    let mut gb =
        TestingGameBoy::new("../test_roms/scribbltests/statcount/statcount-auto.gb").unwrap();

    let regs = gb.clock_until_breakpoint();

    let screen_buffer = gb.screen_buffer();
    crate::tests::print_screen_buffer(screen_buffer);

    // how many failed checks
    if regs.a != 0 {
        panic!("test failed, wrong states count is {}", regs.a);
    }
}
