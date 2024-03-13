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

| Test           | State |
| -------------- | ----- |
| cpu_instrs     | :+1:  |
| instr_timing   | :+1:  |
| halt_bug       | :+1:  |
| mem_timing     | :+1:  |
| mem_timing-2   | :+1:  |
| dmg_sound      | :+1:  |
| cgb_sound      | :+1:  |
| oam_bug        | :+1:  |
| interrupt_time | :+1:  |

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

## [GBMicrotest]

| Test                                  | State |
| ------------------------------------- | ----- |
| audio_testbench | :x: |
| 803-ppu-latch-bgdisplay | :x: |
| cpu_bus_1 | :x: |
| div_inc_timing_a | :+1: |
| div_inc_timing_b | :+1: |
| dma_0x1000 | :+1: |
| dma_0x9000 | :+1: |
| dma_0xA000 | :+1: |
| dma_0xC000 | :+1: |
| dma_0xE000 | :+1: |
| dma_basic | :x: |
| dma_timing_a | :+1: |
| 400-dma | :x: |
| flood_vram | :x: |
| halt_bug | :x: |
| halt_op_dupe_delay | :x: |
| halt_op_dupe | :+1: |
| hblank_int_di_timing_a | :x: |
| hblank_int_di_timing_b | :+1: |
| hblank_int_if_a | :x: |
| hblank_int_if_b | :+1: |
| hblank_int_l0 | :x: |
| hblank_int_l1 | :x: |
| hblank_int_l2 | :x: |
| hblank_int_scx0_if_a | :+1: |
| hblank_int_scx0_if_b | :+1: |
| hblank_int_scx0_if_c | :+1: |
| hblank_int_scx0_if_d | :+1: |
| hblank_int_scx0 | :x: |
| hblank_int_scx1_if_a | :+1: |
| hblank_int_scx1_if_b | :+1: |
| hblank_int_scx1_if_c | :+1: |
| hblank_int_scx1_if_d | :+1: |
| hblank_int_scx1_nops_a | :+1: |
| hblank_int_scx1_nops_b | :+1: |
| hblank_int_scx1 | :x: |
| hblank_int_scx2_if_a | :+1: |
| hblank_int_scx2_if_b | :x: |
| hblank_int_scx2_if_c | :x: |
| hblank_int_scx2_if_d | :x: |
| hblank_int_scx2_nops_a | :x: |
| hblank_int_scx2_nops_b | :x: |
| hblank_int_scx2 | :x: |
| hblank_int_scx3_if_a | :+1: |
| hblank_int_scx3_if_b | :+1: |
| hblank_int_scx3_if_c | :+1: |
| hblank_int_scx3_if_d | :+1: |
| hblank_int_scx3_nops_a | :+1: |
| hblank_int_scx3_nops_b | :+1: |
| hblank_int_scx3 | :+1: |
| hblank_int_scx4_if_a | :+1: |
| hblank_int_scx4_if_b | :+1: |
| hblank_int_scx4_if_c | :+1: |
| hblank_int_scx4_if_d | :+1: |
| hblank_int_scx4_nops_a | :+1: |
| hblank_int_scx4_nops_b | :+1: |
| hblank_int_scx4 | :x: |
| hblank_int_scx5_if_a | :+1: |
| hblank_int_scx5_if_b | :+1: |
| hblank_int_scx5_if_c | :+1: |
| hblank_int_scx5_if_d | :+1: |
| hblank_int_scx5_nops_a | :+1: |
| hblank_int_scx5_nops_b | :+1: |
| hblank_int_scx5 | :x: |
| hblank_int_scx6_if_a | :+1: |
| hblank_int_scx6_if_b | :x: |
| hblank_int_scx6_if_c | :x: |
| hblank_int_scx6_if_d | :x: |
| hblank_int_scx6_nops_a | :x: |
| hblank_int_scx6_nops_b | :x: |
| hblank_int_scx6 | :x: |
| hblank_int_scx7_if_a | :+1: |
| hblank_int_scx7_if_b | :+1: |
| hblank_int_scx7_if_c | :+1: |
| hblank_int_scx7_if_d | :+1: |
| hblank_int_scx7_nops_a | :+1: |
| hblank_int_scx7_nops_b | :+1: |
| hblank_int_scx7 | :x: |
| hblank_scx2_if_a | :x: |
| hblank_scx3_if_a | :+1: |
| hblank_scx3_if_b | :x: |
| hblank_scx3_if_c | :x: |
| hblank_scx3_if_d | :x: |
| hblank_scx3_int_a | :+1: |
| hblank_scx3_int_b | :x: |
| int_hblank_halt_bug_a | :+1: |
| int_hblank_halt_bug_b | :+1: |
| int_hblank_halt_scx0 | :x: |
| int_hblank_halt_scx1 | :x: |
| int_hblank_halt_scx2 | :x: |
| int_hblank_halt_scx3 | :x: |
| int_hblank_halt_scx4 | :x: |
| int_hblank_halt_scx5 | :x: |
| int_hblank_halt_scx6 | :x: |
| int_hblank_halt_scx7 | :x: |
| int_hblank_incs_scx0 | :x: |
| int_hblank_incs_scx1 | :x: |
| int_hblank_incs_scx2 | :x: |
| int_hblank_incs_scx3 | :x: |
| int_hblank_incs_scx4 | :x: |
| int_hblank_incs_scx5 | :x: |
| int_hblank_incs_scx6 | :x: |
| int_hblank_incs_scx7 | :x: |
| int_hblank_nops_scx0 | :x: |
| int_hblank_nops_scx1 | :x: |
| int_hblank_nops_scx2 | :x: |
| int_hblank_nops_scx3 | :x: |
| int_hblank_nops_scx4 | :x: |
| int_hblank_nops_scx5 | :x: |
| int_hblank_nops_scx6 | :x: |
| int_hblank_nops_scx7 | :x: |
| int_lyc_halt | :x: |
| int_lyc_incs | :+1: |
| int_lyc_nops | :x: |
| int_oam_halt | :x: |
| int_oam_incs | :x: |
| int_oam_nops | :x: |
| int_timer_halt_div_a | :+1: |
| int_timer_halt_div_b | :x: |
| int_timer_halt | :x: |
| int_timer_incs | :+1: |
| int_timer_nops_div_a | :+1: |
| int_timer_nops_div_b | :+1: |
| int_timer_nops | :+1: |
| int_vblank1_halt | :x: |
| int_vblank1_incs | :x: |
| int_vblank1_nops | :x: |
| int_vblank2_halt | :x: |
| int_vblank2_incs | :x: |
| int_vblank2_nops | :x: |
| is_if_set_during_ime0 | :+1: |
| 007-lcd_on_stat | :x: |
| lcdon_halt_to_vblank_int_a | :x: |
| lcdon_halt_to_vblank_int_b | :+1: |
| lcdon_nops_to_vblank_int_a | :x: |
| lcdon_nops_to_vblank_int_b | :+1: |
| lcdon_to_if_oam_a | :+1: |
| lcdon_to_if_oam_b | :x: |
| lcdon_to_ly1_a | :+1: |
| lcdon_to_ly1_b | :+1: |
| lcdon_to_ly2_a | :+1: |
| lcdon_to_ly2_b | :+1: |
| lcdon_to_ly3_a | :+1: |
| lcdon_to_ly3_b | :+1: |
| lcdon_to_lyc1_int | :+1: |
| lcdon_to_lyc2_int | :+1: |
| lcdon_to_lyc3_int | :+1: |
| lcdon_to_oam_int_l0 | :x: |
| lcdon_to_oam_int_l1 | :x: |
| lcdon_to_oam_int_l2 | :x: |
| lcdon_to_oam_unlock_a | :+1: |
| lcdon_to_oam_unlock_b | :+1: |
| lcdon_to_oam_unlock_c | :+1: |
| lcdon_to_oam_unlock_d | :x: |
| lcdon_to_stat0_a | :+1: |
| lcdon_to_stat0_b | :+1: |
| lcdon_to_stat0_c | :+1: |
| lcdon_to_stat0_d | :+1: |
| lcdon_to_stat1_a | :+1: |
| lcdon_to_stat1_b | :x: |
| lcdon_to_stat1_c | :+1: |
| lcdon_to_stat1_d | :x: |
| lcdon_to_stat1_e | :+1: |
| lcdon_to_stat2_a | :x: |
| lcdon_to_stat2_b | :+1: |
| lcdon_to_stat2_c | :+1: |
| lcdon_to_stat2_d | :+1: |
| lcdon_to_stat3_a | :+1: |
| lcdon_to_stat3_b | :+1: |
| lcdon_to_stat3_c | :+1: |
| lcdon_to_stat3_d | :+1: |
| lcdon_write_timing | :x: |
| line_144_oam_int_a | :+1: |
| line_144_oam_int_b | :x: |
| line_144_oam_int_c | :x: |
| line_144_oam_int_d | :x: |
| line_153_ly_a | :+1: |
| line_153_ly_b | :+1: |
| line_153_ly_c | :x: |
| line_153_ly_d | :+1: |
| line_153_ly_e | :x: |
| line_153_ly_f | :+1: |
| line_153_lyc_a | :+1: |
| line_153_lyc_b | :+1: |
| line_153_lyc_c | :x: |
| line_153_lyc_int_a | :+1: |
| line_153_lyc_int_b | :+1: |
| line_153_lyc0_int_inc_sled | :+1: |
| line_153_lyc0_stat_timing_a | :+1: |
| line_153_lyc0_stat_timing_b | :+1: |
| line_153_lyc0_stat_timing_c | :+1: |
| line_153_lyc0_stat_timing_d | :+1: |
| line_153_lyc0_stat_timing_e | :+1: |
| line_153_lyc0_stat_timing_f | :x: |
| line_153_lyc0_stat_timing_g | :+1: |
| line_153_lyc0_stat_timing_h | :x: |
| line_153_lyc0_stat_timing_i | :+1: |
| line_153_lyc0_stat_timing_j | :x: |
| line_153_lyc0_stat_timing_k | :+1: |
| line_153_lyc0_stat_timing_l | :+1: |
| line_153_lyc0_stat_timing_m | :x: |
| line_153_lyc0_stat_timing_n | :+1: |
| line_153_lyc153_stat_timing_a | :+1: |
| line_153_lyc153_stat_timing_b | :+1: |
| line_153_lyc153_stat_timing_c | :x: |
| line_153_lyc153_stat_timing_d | :+1: |
| line_153_lyc153_stat_timing_e | :x: |
| line_153_lyc153_stat_timing_f | :+1: |
| line_65_ly | :x: |
| ly_while_lcd_off | :x: |
| lyc_int_halt_a | :x: |
| lyc_int_halt_b | :+1: |
| lyc1_int_halt_a | :x: |
| lyc1_int_halt_b | :+1: |
| lyc1_int_if_edge_a | :+1: |
| lyc1_int_if_edge_b | :+1: |
| lyc1_int_if_edge_c | :+1: |
| lyc1_int_if_edge_d | :+1: |
| lyc1_int_nops_a | :+1: |
| lyc1_int_nops_b | :+1: |
| lyc1_write_timing_a | :+1: |
| lyc1_write_timing_b | :+1: |
| lyc1_write_timing_c | :+1: |
| lyc1_write_timing_d | :+1: |
| lyc2_int_halt_a | :x: |
| lyc2_int_halt_b | :+1: |
| mbc1_ram_banks | :+1: |
| mbc1_rom_banks | :x: |
| minimal | :x: |
| mode2_stat_int_to_oam_unlock | :x: |
| oam_int_halt_a | :x: |
| oam_int_halt_b | :+1: |
| oam_int_if_edge_a | :+1: |
| oam_int_if_edge_b | :x: |
| oam_int_if_edge_c | :+1: |
| oam_int_if_edge_d | :x: |
| oam_int_if_level_c | :+1: |
| oam_int_if_level_d | :x: |
| oam_int_inc_sled | :x: |
| oam_int_nops_a | :x: |
| oam_int_nops_b | :+1: |
| 000-oam_lock | :x: |
| oam_read_l0_a | :+1: |
| oam_read_l0_b | :+1: |
| oam_read_l0_c | :+1: |
| oam_read_l0_d | :x: |
| oam_read_l1_a | :+1: |
| oam_read_l1_b | :+1: |
| oam_read_l1_c | :+1: |
| oam_read_l1_d | :x: |
| oam_read_l1_e | :+1: |
| oam_read_l1_f | :+1: |
| oam_sprite_trashing | :x: |
| oam_write_l0_a | :+1: |
| oam_write_l0_b | :+1: |
| oam_write_l0_c | :+1: |
| oam_write_l0_d | :x: |
| oam_write_l0_e | :x: |
| oam_write_l1_a | :+1: |
| oam_write_l1_b | :+1: |
| oam_write_l1_c | :x: |
| oam_write_l1_d | :+1: |
| oam_write_l1_e | :+1: |
| oam_write_l1_f | :x: |
| poweron_bgp_000 | :+1: |
| poweron_div_000 | :x: |
| poweron_div_004 | :x: |
| poweron_div_005 | :x: |
| poweron_dma_000 | :x: |
| poweron_if_000 | :+1: |
| poweron_joy_000 | :+1: |
| poweron_lcdc_000 | :+1: |
| poweron_ly_000 | :+1: |
| poweron_ly_119 | :x: |
| poweron_ly_120 | :+1: |
| poweron_ly_233 | :x: |
| poweron_ly_234 | :+1: |
| poweron_lyc_000 | :+1: |
| poweron_oam_000 | :+1: |
| poweron_oam_005 | :+1: |
| poweron_oam_006 | :+1: |
| poweron_oam_069 | :+1: |
| poweron_oam_070 | :+1: |
| poweron_oam_119 | :x: |
| poweron_oam_120 | :+1: |
| poweron_oam_121 | :+1: |
| poweron_oam_183 | :+1: |
| poweron_oam_184 | :+1: |
| poweron_oam_233 | :x: |
| poweron_oam_234 | :+1: |
| poweron_oam_235 | :+1: |
| poweron_obp0_000 | :+1: |
| poweron_obp1_000 | :+1: |
| poweron_sb_000 | :+1: |
| poweron_sc_000 | :+1: |
| poweron_scx_000 | :+1: |
| poweron_scy_000 | :+1: |
| poweron_stat_000 | :+1: |
| poweron_stat_005 | :+1: |
| poweron_stat_006 | :x: |
| poweron_stat_007 | :+1: |
| poweron_stat_026 | :x: |
| poweron_stat_027 | :+1: |
| poweron_stat_069 | :x: |
| poweron_stat_070 | :+1: |
| poweron_stat_119 | :+1: |
| poweron_stat_120 | :x: |
| poweron_stat_121 | :+1: |
| poweron_stat_140 | :x: |
| poweron_stat_141 | :+1: |
| poweron_stat_183 | :x: |
| poweron_stat_184 | :+1: |
| poweron_stat_234 | :x: |
| poweron_stat_235 | :+1: |
| poweron_tac_000 | :+1: |
| poweron_tima_000 | :+1: |
| poweron_tma_000 | :+1: |
| poweron_vram_000 | :+1: |
| poweron_vram_025 | :+1: |
| poweron_vram_026 | :x: |
| poweron_vram_069 | :x: |
| poweron_vram_070 | :+1: |
| poweron_vram_139 | :+1: |
| poweron_vram_140 | :x: |
| poweron_vram_183 | :x: |
| poweron_vram_184 | :+1: |
| poweron_wx_000 | :+1: |
| poweron_wy_000 | :+1: |
| poweron | :x: |
| ppu_scx_vs_bgp | :x: |
| ppu_sprite_testbench | :x: |
| ppu_sprite0_scx0_a | :+1: |
| ppu_sprite0_scx0_b | :+1: |
| ppu_sprite0_scx1_a | :+1: |
| ppu_sprite0_scx1_b | :+1: |
| ppu_sprite0_scx2_a | :+1: |
| ppu_sprite0_scx2_b | :x: |
| ppu_sprite0_scx3_a | :+1: |
| ppu_sprite0_scx3_b | :x: |
| ppu_sprite0_scx4_a | :+1: |
| ppu_sprite0_scx4_b | :+1: |
| ppu_sprite0_scx5_a | :+1: |
| ppu_sprite0_scx5_b | :+1: |
| ppu_sprite0_scx6_a | :+1: |
| ppu_sprite0_scx6_b | :x: |
| ppu_sprite0_scx7_a | :+1: |
| ppu_sprite0_scx7_b | :x: |
| ppu_spritex_vs_scx | :x: |
| ppu_win_vs_wx | :x: |
| ppu_wx_early | :x: |
| 800-ppu-latch-scx | :x: |
| 801-ppu-latch-scy | :x: |
| sprite_0_a | :x: |
| sprite_0_b | :+1: |
| sprite_1_a | :x: |
| sprite_1_b | :+1: |
| sprite4_0_a | :x: |
| sprite4_0_b | :+1: |
| sprite4_1_a | :x: |
| sprite4_1_b | :+1: |
| sprite4_2_a | :x: |
| sprite4_2_b | :+1: |
| sprite4_3_a | :x: |
| sprite4_3_b | :+1: |
| sprite4_4_a | :x: |
| sprite4_4_b | :+1: |
| sprite4_5_a | :x: |
| sprite4_5_b | :+1: |
| sprite4_6_a | :x: |
| sprite4_6_b | :+1: |
| sprite4_7_a | :x: |
| sprite4_7_b | :+1: |
| stat_write_glitch_l0_a | :x: |
| stat_write_glitch_l0_b | :x: |
| stat_write_glitch_l0_c | :+1: |
| stat_write_glitch_l1_a | :+1: |
| stat_write_glitch_l1_b | :x: |
| stat_write_glitch_l1_c | :x: |
| stat_write_glitch_l1_d | :+1: |
| stat_write_glitch_l143_a | :+1: |
| stat_write_glitch_l143_b | :x: |
| stat_write_glitch_l143_c | :x: |
| stat_write_glitch_l143_d | :x: |
| stat_write_glitch_l154_a | :x: |
| stat_write_glitch_l154_b | :x: |
| stat_write_glitch_l154_c | :+1: |
| stat_write_glitch_l154_d | :x: |
| temp | :+1: |
| 802-ppu-latch-tileselect | :x: |
| 004-tima_boot_phase | :x: |
| 004-tima_cycle_timer | :x: |
| timer_div_phase_c | :+1: |
| timer_div_phase_d | :+1: |
| timer_tima_inc_256k_a | :+1: |
| timer_tima_inc_256k_b | :+1: |
| timer_tima_inc_256k_c | :+1: |
| timer_tima_inc_256k_d | :+1: |
| timer_tima_inc_256k_e | :+1: |
| timer_tima_inc_256k_f | :+1: |
| timer_tima_inc_256k_g | :+1: |
| timer_tima_inc_256k_h | :+1: |
| timer_tima_inc_256k_i | :+1: |
| timer_tima_inc_256k_j | :+1: |
| timer_tima_inc_256k_k | :+1: |
| timer_tima_inc_64k_a | :+1: |
| timer_tima_inc_64k_b | :+1: |
| timer_tima_inc_64k_c | :+1: |
| timer_tima_inc_64k_d | :+1: |
| timer_tima_phase_a | :x: |
| timer_tima_phase_b | :x: |
| timer_tima_phase_c | :x: |
| timer_tima_phase_d | :x: |
| timer_tima_phase_e | :x: |
| timer_tima_phase_f | :x: |
| timer_tima_phase_g | :x: |
| timer_tima_phase_h | :x: |
| timer_tima_phase_i | :x: |
| timer_tima_phase_j | :x: |
| timer_tima_reload_256k_a | :+1: |
| timer_tima_reload_256k_b | :+1: |
| timer_tima_reload_256k_c | :+1: |
| timer_tima_reload_256k_d | :+1: |
| timer_tima_reload_256k_e | :+1: |
| timer_tima_reload_256k_f | :+1: |
| timer_tima_reload_256k_g | :+1: |
| timer_tima_reload_256k_h | :+1: |
| timer_tima_reload_256k_i | :+1: |
| timer_tima_reload_256k_j | :+1: |
| timer_tima_reload_256k_k | :+1: |
| timer_tima_write_a | :+1: |
| timer_tima_write_b | :+1: |
| timer_tima_write_c | :+1: |
| timer_tima_write_d | :+1: |
| timer_tima_write_e | :+1: |
| timer_tima_write_f | :+1: |
| timer_tma_write_a | :+1: |
| timer_tma_write_b | :+1: |
| 500-scx-timing | :x: |
| toggle_lcdc | :x: |
| vblank_int_halt_a | :x: |
| vblank_int_halt_b | :+1: |
| vblank_int_if_a | :+1: |
| vblank_int_if_b | :x: |
| vblank_int_if_c | :+1: |
| vblank_int_if_d | :x: |
| vblank_int_inc_sled | :x: |
| vblank_int_nops_a | :x: |
| vblank_int_nops_b | :+1: |
| vblank2_int_halt_a | :x: |
| vblank2_int_halt_b | :+1: |
| vblank2_int_if_a | :+1: |
| vblank2_int_if_b | :x: |
| vblank2_int_if_c | :+1: |
| vblank2_int_if_d | :x: |
| vblank2_int_inc_sled | :x: |
| vblank2_int_nops_a | :x: |
| vblank2_int_nops_b | :+1: |
| 002-vram_locked | :x: |
| vram_read_l0_a | :+1: |
| vram_read_l0_b | :x: |
| vram_read_l0_c | :x: |
| vram_read_l0_d | :+1: |
| vram_read_l1_a | :+1: |
| vram_read_l1_b | :x: |
| vram_read_l1_c | :x: |
| vram_read_l1_d | :+1: |
| 001-vram_unlocked | :x: |
| vram_write_l0_a | :+1: |
| vram_write_l0_b | :x: |
| vram_write_l0_c | :x: |
| vram_write_l0_d | :+1: |
| vram_write_l1_a | :+1: |
| vram_write_l1_b | :x: |
| vram_write_l1_c | :x: |
| vram_write_l1_d | :+1: |
| wave_write_to_0xC003 | :x: |
| win0_a | :+1: |
| win0_b | :x: |
| win0_scx3_a | :+1: |
| win0_scx3_b | :+1: |
| win1_a | :+1: |
| win1_b | :x: |
| win10_a | :+1: |
| win10_b | :x: |
| win10_scx3_a | :+1: |
| win10_scx3_b | :x: |
| win11_a | :+1: |
| win11_b | :x: |
| win12_a | :+1: |
| win12_b | :x: |
| win13_a | :+1: |
| win13_b | :x: |
| win14_a | :+1: |
| win14_b | :x: |
| win15_a | :+1: |
| win15_b | :x: |
| win2_a | :+1: |
| win2_b | :+1: |
| win3_a | :+1: |
| win3_b | :+1: |
| win4_a | :+1: |
| win4_b | :+1: |
| win5_a | :+1: |
| win5_b | :+1: |
| win6_a | :x: |
| win6_b | :+1: |
| win7_a | :x: |
| win7_b | :+1: |
| win8_a | :+1: |
| win8_b | :x: |
| win9_a | :+1: |
| win9_b | :x: |
| 000-write_to_x8000 | :x: |


## Extra
These are valuable tests, they come in a single rom, so they were grouped into
a single table

| Test             | State |
| ---------------- | ----- |
| [rtc3test]       | :+1:  |
| [bullyGB] in DMG | :question:* |
| [bullyGB] in CGB | :question:* |

> \* previusly passed, but now it fails, it needs to be retested and fix the issues


[dmg_acid2]: https://github.com/mattcurrie/dmg-acid2
[cgb_acid2]: https://github.com/mattcurrie/cgb-acid2
[cgb_acid_hell]: https://github.com/mattcurrie/cgb-acid-hell
[blargg_tests]: https://gbdev.gg8.se/wiki/articles/Test_ROMs
[scribbltests]: https://github.com/Hacktix/scribbltests
[mooneye_tests]: https://github.com/Gekkio/mooneye-gb/tree/master/tests
[SameSuite]: https://github.com/LIJI32/SameSuite
[rtc3test]: https://github.com/aaaaaa123456789/rtc3test
[bullyGB]: https://github.com/Hacktix/BullyGB
[GBMicrotest]: https://github.com/aappleby/GBMicrotest

