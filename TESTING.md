# Tests
A documentation of all hardware tests tested on `mizu`.

- [Tests](#tests)
  - [Acid2 tests](#acid2-tests)
  - [Blargg tests](#blargg-tests)
  - [Scribble tests](#scribble-tests)
  - [Mooneye tests](#mooneye-tests)
    - [Acceptance](#acceptance)
      - [Bits (unusable bits in memory and registers)](#bits-unusable-bits-in-memory-and-registers)
      - [Instructions](#instructions)
      - [Interrupt handling](#interrupt-handling)
      - [OAM DMA](#oam-dma)
      - [PPU](#ppu)
      - [Serial](#serial)
      - [Timer](#timer)
    - [emulator-only](#emulator-only)
      - [MBC1](#mbc1)
      - [MBC2](#mbc2)
      - [MBC5](#mbc5)
    - [manual](#manual)
    - [misc (CGB)](#misc-cgb)
      - [Bits](#bits)
      - [PPU](#ppu-1)
  - [SameSuite](#samesuite)
    - [HDMA](#hdma)
    - [PPU](#ppu-2)
    - [APU](#apu)
      - [Channel1](#channel1)
      - [Channel2](#channel2)
      - [Channel3](#channel3)
      - [Channel4](#channel4)
  - [Extra](#extra)

## Acid2 tests

| Tests           | State |
| --------------- | ----- |
| [dmg_acid2]     | :+1:  |
| [cgb_acid2]     | :+1:  |
| [cgb_acid_hell] | :x:  |

## [Blargg tests][blargg_tests]

| Test         | State |
| ------------ | ----- |
| cpu_instrs   | :+1:  |
| instr_timing | :+1:  |
| halt_bug     | :+1:  |
| mem_timing-2 | :+1:  |
| dmg_sound    | :+1:  |
| cgb_sound    | :+1:  |
| oam_bug      | :+1:  |

## [Scribble tests][scribbltests]

| Test         | State |
| ------------ | ----- |
| lycscx       | :+1:  |
| lycscy       | :+1:  |
| palettely    | :+1:  |
| scxly        | :+1:  |
| statcount    | :+1:  |

## [Mooneye tests][mooneye_tests]

### Acceptance

| Test                    | State |
| ----------------------- | ----- |
| add_sp_e_timing         | :+1:  |
| boot_div-dmgABCmgb      | :+1:  |
| boot_hwio-dmgABCmgb     | :+1:  |
| boot_regs-dmgABC        | :+1:  |
| call_timing             | :+1:  |
| call_timing2            | :+1:  |
| call_cc_timing          | :+1:  |
| call_cc_timing2         | :+1:  |
| di_timing GS            | :+1:  |
| div_timing              | :+1:  |
| ei_sequence             | :+1:  |
| ei_timing               | :+1:  |
| halt_ime0_ei            | :+1:  |
| halt_ime0_nointr_timing | :+1:  |
| halt_ime1_timing        | :+1:  |
| halt_ime1_timing2-GS    | :+1:  |
| if_ie_registers         | :+1:  |
| intr_timing             | :+1:  |
| jp_timing               | :+1:  |
| jp_cc_timing            | :+1:  |
| ld_hl_sp_e_timing       | :+1:  |
| oam_dma_restart         | :+1:  |
| oam_dma_start           | :+1:  |
| oam_dma_timing          | :+1:  |
| pop_timing              | :+1:  |
| push_timing             | :+1:  |
| rapid_di_ei             | :+1:  |
| ret_timing              | :+1:  |
| ret_cc_timing           | :+1:  |
| reti_timing             | :+1:  |
| reti_intr_timing        | :+1:  |
| rst_timing              | :+1:  |

#### Bits (unusable bits in memory and registers)

| Test           | State |
| -------------- | ----- |
| mem_oam        | :+1:  |
| reg_f          | :+1:  |
| unused_hwio-GS | :+1:  |

#### Instructions

| Test | State |
| ---- | ----- |
| daa  | :+1:  |

#### Interrupt handling

| Test                        | State |
| --------------------------- | ----- |
| ie_push                     | :+1:  |

#### OAM DMA

| Test       | State     |
| ---------- | --------- |
| basic      | :+1:      |
| reg_read   | :+1:      |
| sources-GS | :+1:/:x:* |

> \* `sources-GS` passes on both CGB and DMG in `mizu` but it should
> pass on DMG and fail on CGB.

#### PPU

| Test                        | State |
| --------------------------- | ----- |
| hblank_ly_scx_timing-GS     | :x:   |
| intr_1_2_timing-GS          | :+1:  |
| intr_2_0_timing             | :+1:  |
| intr_2_mode0_timing         | :x:   |
| intr_2_mode3_timing         | :x:   |
| intr_2_oam_ok_timing        | :+1:  |
| intr_2_mode0_timing_sprites | :x:   |
| lcdon_timing-GS             | :x:   |
| lcdon_write_timing-GS       | :x:   |
| stat_irq_blocking           | :+1:  |
| stat_lyc_onoff              | :+1:  |
| vblank_stat_intr-GS         | :+1:  |

#### Serial 

| Test                       | State |
| -------------------------- | ----- |
| boot_sclk_align-dmgABCmgb  | :+1:  |


#### Timer

| Test                 | State |
| -------------------- | ----- |
| div_write            | :+1:  |
| rapid_toggle         | :+1:  |
| tim00_div_trigger    | :+1:  |
| tim00                | :+1:  |
| tim01_div_trigger    | :+1:  |
| tim01                | :+1:  |
| tim10_div_trigger    | :+1:  |
| tim10                | :+1:  |
| tim11_div_trigger    | :+1:  |
| tim11                | :+1:  |
| tima_reload          | :+1:  |
| tima_write_reloading | :+1:  |
| tma_write_reloading  | :+1:  |

### emulator-only

#### MBC1

| Test              | State |
| ----------------- | ----- |
| bits_bank1        | :+1:  |
| bits_bank2        | :+1:  |
| bits_mode         | :+1:  |
| bits_ramg         | :+1:  |
| rom_512kb         | :+1:  |
| rom_1Mb           | :+1:  |
| rom_2Mb           | :+1:  |
| rom_4Mb           | :+1:  |
| rom_8Mb           | :+1:  |
| rom_16Mb          | :+1:  |
| ram_64kb          | :+1:  |
| ram_256kb         | :+1:  |
| multicart_rom_8Mb | :+1:  |

#### MBC2

| Test              | State |
| ----------------- | ----- |
| bits_ramg         | :+1:  |
| bits_romb         | :+1:  |
| bits_unused       | :+1:  |
| rom_512kb         | :+1:  |
| rom_1Mb           | :+1:  |
| rom_2Mb           | :+1:  |
| ram               | :+1:  |

#### MBC5

| Test              | State |
| ----------------- | ----- |
| rom_512kb         | :+1:  |
| rom_1Mb           | :+1:  |
| rom_2Mb           | :+1:  |
| rom_4Mb           | :+1:  |
| rom_8Mb           | :+1:  |
| rom_16Mb          | :+1:  |
| rom_32Mb          | :+1:  |
| rom_64Mb          | :+1:  |

### manual

| Test            | State |
| --------------- | ----- |
| sprite_priority | :+1:  |

### misc (CGB)

| Test              | State |
| ---------------   | ----- |
| boot_div-cgbABCDE | :+1:  |
| boot_hwio-C       | :+1:  |
| boot_regs-cgb     | :+1:  |

#### Bits

| Test          | State |
| ------------- | ----- |
| unused_hwio-C | :+1:  |

#### PPU

| Test               | State |
| ------------------ | ----- |
| vblank_stat_intr-C | :+1:  |

## [SameSuite]

### HDMA

| Test           | State |
| -------------- | ----- |
| gbc_dma_cont   | :+1:  |
| gdma_addr_mask | :+1:  |
| hdma_lcd_off   | :+1:  |
| hdma_mode0     | :+1:  |

### PPU 

| Test                   | State |
| ---------------------- | ----- |
| blocking_bgpi_increase | :+1:  |

### APU

| Test                        | State |
| --------------------------- | ----- |
| div_write_trigger_10        | :+1:  |
| div_write_trigger           | :+1:  |
| div_write_trigger_volume    | :+1:  |
| div_write_trigger_volume_10 | :+1:  |
| div_trigger_volume_10       | :+1:  |


#### Channel1

| Test                                  | State |
| ------------------------------------- | ----- |
| channel_1_align                       | :x:   |
| channel_1_align_cpu                   | :x:   |
| channel_1_delay                       | :x:   |
| channel_1_duty                        | :x:   |
| channel_1_duty_delay                  | :x:   |
| channel_1_extra_length_clocking-cgb0B | :x:   |
| channel_1_freq_change                 | :x:   |
| channel_1_freq_change_timing-A        | :x:   |
| channel_1_freq_change_timing-cgb0BC   | :x:   |
| channel_1_freq_change_timing-cgbDE    | :x:   |
| channel_1_nrx2_glitch                 | :x:   |
| channel_1_nrx2_speed_change           | :x:   |
| channel_1_restart                     | :x:   |
| channel_1_restart_nrx2_glitch         | :x:   |
| channel_1_stop_div                    | :x:   |
| channel_1_stop_restart                | :x:   |
| channel_1_sweep                       | :x:   |
| channel_1_sweep_restart               | :x:   |
| channel_1_sweep_restart_2             | :x:   |
| channel_1_volume                      | :x:   |
| channel_1_volume_div                  | :x:   |

#### Channel2

| Test                                  | State |
| ------------------------------------- | ----- |
| channel_2_align                       | :x:   |
| channel_2_align_cpu                   | :x:   |
| channel_2_delay                       | :x:   |
| channel_2_duty                        | :x:   |
| channel_2_duty_delay                  | :x:   |
| channel_2_extra_length_clocking-cgb0B | :x:   |
| channel_2_freq_change                 | :x:   |
| channel_2_nrx2_glitch                 | :x:   |
| channel_2_nrx2_speed_change           | :x:   |
| channel_2_restart                     | :x:   |
| channel_2_restart_nrx2_glitch         | :x:   |
| channel_2_stop_div                    | :x:   |
| channel_2_stop_restart                | :x:   |
| channel_2_volume                      | :x:   |
| channel_2_volume_div                  | :x:   |

#### Channel3

| Test                                  | State |
| ------------------------------------- | ----- |
| channel_3_and_glitch                  | :x:   |
| channel_3_delay                       | :x:   |
| channel_3_extra_length_clocking-cgb0  | :x:   |
| channel_3_extra_length_clocking-cgbB  | :x:   |
| channel_3_first_sample                | :x:   |
| channel_3_freq_change_delay           | :x:   |
| channel_3_restart_delay               | :x:   |
| channel_3_restart_during_delay        | :x:   |
| channel_3_restart_stop_delay          | :x:   |
| channel_3_shift_delay                 | :x:   |
| channel_3_shift_skip_delay            | :x:   |
| channel_3_stop_delay                  | :+1:  |
| channel_3_stop_div                    | :x:   |
| channel_3_wave_ram_locked_write       | :+1:  |
| channel_3_wave_ram_sync               | :x:   |

#### Channel4

| Test                                  | State |
| ------------------------------------- | ----- |
| channel_4_align                       | :x:   |
| channel_4_delay                       | :x:   |
| channel_4_equivalent_frequencies      | :x:   |
| channel_4_extra_length_clocking-cgb0B | :x:   |
| channel_4_freq_change                 | :x:   |
| channel_4_frequency_alignment         | :x:   |
| channel_4_lfsr                        | :x:   |
| channel_4_lfsr15                      | :x:   |
| channel_4_lfsr_15_7                   | :x:   |
| channel_4_lfsr_7_15                   | :x:   |
| channel_4_lfsr_restart                | :x:   |
| channel_4_lfsr_restart_fast           | :x:   |
| channel_4_volume_div                  | :x:   |

## Extra
These are valuable tests, they come in a single rom, so they were grouped into
a single table

| Test             | State |
| ---------------- | ----- |
| [rtc3test]       | :+1:  |
| [bullyGB] in DMG | :+1:  |
| [bullyGB] in CGB | :+1:  |


[dmg_acid2]: https://github.com/mattcurrie/dmg-acid2
[cgb_acid2]: https://github.com/mattcurrie/cgb-acid2
[cgb_acid_hell]: https://github.com/mattcurrie/cgb-acid-hell
[blargg_tests]: https://gbdev.gg8.se/wiki/articles/Test_ROMs
[scribbltests]: https://github.com/Hacktix/scribbltests
[mooneye_tests]: https://github.com/Gekkio/mooneye-gb/tree/master/tests
[SameSuite]: https://github.com/LIJI32/SameSuite
[rtc3test]: https://github.com/aaaaaa123456789/rtc3test
[bullyGB]: https://github.com/Hacktix/BullyGB

