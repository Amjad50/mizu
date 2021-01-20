gb_tests!(
    inf; // clock until infinite loop

    // FIXME: the test passes but the screen is not correct
    //  the label for test 11 does not show "OK"
    blargg_cpu_instrs,
    "blargg-gb-tests/cpu_instrs/cpu_instrs.gb",
    3892107528677358037;

    blargg_instr_timing,
    "blargg-gb-tests/instr_timing/instr_timing.gb",
    14586804626949338345;

    blargg_halt_bug,
    "blargg-gb-tests/halt_bug.gb",
    3778129031474618196;

    blargg_mem_timing_2,
    "blargg-gb-tests/mem_timing-2/mem_timing.gb",
    12164226896603567743;

    // dmg_sound individual temporary for now, until all passes, we can use
    // the full one
    blargg_dmg_sound_01_registers,
    "blargg-gb-tests/dmg_sound/rom_singles/01-registers.gb",
    14537940686373650876;

    blargg_dmg_sound_02_len_ctr,
    "blargg-gb-tests/dmg_sound/rom_singles/02-len ctr.gb",
    11076959017028213871;

    blargg_dmg_sound_03_trigger,
    "blargg-gb-tests/dmg_sound/rom_singles/03-trigger.gb",
    14177237474296683138;

    blargg_dmg_sound_04_sweep,
    "blargg-gb-tests/dmg_sound/rom_singles/04-sweep.gb",
    9687063080497492333;

    blargg_dmg_sound_05_sweep_details,
    "blargg-gb-tests/dmg_sound/rom_singles/05-sweep details.gb",
    12233984480315533249;

    blargg_dmg_sound_06_overflow_on_trigger,
    "blargg-gb-tests/dmg_sound/rom_singles/06-overflow on trigger.gb",
    2741900054980679919;

    blargg_dmg_sound_07_len_sweep_period_sync,
    "blargg-gb-tests/dmg_sound/rom_singles/07-len sweep period sync.gb",
    1423221729427222721;

    blargg_dmg_sound_all for "dmg",
    "blargg-gb-tests/dmg_sound/dmg_sound.gb",
    1594458519587061298;

    blargg_cgb_sound_all,
    "blargg-gb-tests/cgb_sound/cgb_sound.gb",
    4141669196667164762;
);
