use super::TestingGameBoy;

gb_tests!(
    brk; // clock until breakpoint

    lycscx,
    "scribbltests/lycscx/lycscx.gb",
    14138466600053577134;

    lycscy,
    "scribbltests/lycscy/lycscy.gb",
    10627942335066809926;

    palettely,
    "scribbltests/palettely/palettely.gb",
    3173975241828002923;

    scxly,
    "scribbltests/scxly/scxly.gb",
    1708932292293937725;

);

#[test]
#[allow(dead_code)]
fn statcount_auto() {
    let mut gb = TestingGameBoy::new(
        "../test_roms/scribbltests/statcount/statcount-auto.gb",
        false,
    )
    .unwrap();

    let regs = gb.clock_until_breakpoint();

    gb.print_screen_buffer();

    // how many failed checks
    if regs.a != 0 {
        panic!("test failed, wrong states count is {}", regs.a);
    }
}
