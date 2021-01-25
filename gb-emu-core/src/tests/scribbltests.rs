use super::TestingGameBoy;

gb_tests!(
    brk; // clock until breakpoint

    lycscx,
    "scribbltests/lycscx/lycscx.gb",
    1239459159191104188;

    lycscy,
    "scribbltests/lycscy/lycscy.gb",
    9765346434603212938;

    palettely,
    "scribbltests/palettely/palettely.gb",
    17545493111125126301;

    scxly,
    "scribbltests/scxly/scxly.gb",
    13491206425213749962;

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
