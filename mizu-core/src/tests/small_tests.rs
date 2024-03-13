gb_tests!(
    brk; // clock until break

    dmg_acid2_test,
    "dmg-acid2/dmg-acid2.gb",
    13523824884037480967,
    13523824884037480967;

    cgb_acid2_test for cgb,
    "cgb-acid2/cgb-acid2.gbc",
    0,
    4378550468433865064;

    rtc3test_1,
    "../rtc3test/rtc3test-1.gb",
    5668068657756263343,
    13700459787635561240;

    rtc3test_2,
    "../rtc3test/rtc3test-2.gb",
    9943343428460028138,
    14288417446659987136;

    rtc3test_3,
    "../rtc3test/rtc3test-3.gb",
    11694994367292180084,
    2728978286698242625;
);

gb_tests!(
    frames 70; // clock for 70 frames

    mbc3_tester,
    "mbc3-tester/mbc3-tester.gb",
    11138354573804784769,
    11138354573804784769;

    turtle_window_y_trigger_wx_offscreen,
    "turtle-tests/window_y_trigger_wx_offscreen/window_y_trigger_wx_offscreen.gb",
    7770242352201540162,
    7770242352201540162;

    // turtle_window_y_trigger,
    // "turtle-tests/window_y_trigger/window_y_trigger.gb",
    // 0,
    // 0;
);
