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
    17440332955472206150;

    blargg_dmg_sound_02_len_ctr,
    "blargg-gb-tests/dmg_sound/rom_singles/02-len ctr.gb",
    4060957075670844583;

    blargg_dmg_sound_03_trigger,
    "blargg-gb-tests/dmg_sound/rom_singles/03-trigger.gb",
    7262377342751395856;

    blargg_dmg_sound_04_sweep,
    "blargg-gb-tests/dmg_sound/rom_singles/04-sweep.gb",
    13946185113601916246;

    blargg_dmg_sound_05_sweep_details,
    "blargg-gb-tests/dmg_sound/rom_singles/05-sweep details.gb",
    11178214727532716427;

    blargg_dmg_sound_06_overflow_on_trigger,
    "blargg-gb-tests/dmg_sound/rom_singles/06-overflow on trigger.gb",
    2121820975398441942;

    blargg_dmg_sound_07_len_sweep_period_sync,
    "blargg-gb-tests/dmg_sound/rom_singles/07-len sweep period sync.gb",
    10534885036225965023;

    // fail in CGB
    // blargg_dmg_sound_08_len_ctr_during_power,
    // "blargg-gb-tests/dmg_sound/rom_singles/08-len ctr during power.gb",
    // 12779871937667929616;

    // fail in CGB
    //blargg_dmg_sound_11_regs_after_power,
    //"blargg-gb-tests/dmg_sound/rom_singles/11-regs after power.gb",
    //14344078491883240837;

    blargg_cgb_sound_all,
    "blargg-gb-tests/cgb_sound/cgb_sound.gb",
    4141669196667164762;
);
