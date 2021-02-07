gb_tests!(
    brk; // clock until break

    dma_gbc_dma_cont for cgb,
    "SameSuite/dma/gbc_dma_cont.gb",
    0,
    11599733213654421168;

    dma_gdma_addr_mask for cgb,
    "SameSuite/dma/gdma_addr_mask.gb",
    0,
    9261950325417747531;

    dma_hdma_lcd_off for cgb,
    "SameSuite/dma/hdma_lcd_off.gb",
    0,
    1934747480547799326;

    dma_hdma_mode0 for cgb,
    "SameSuite/dma/hdma_mode0.gb",
    0,
    1934747480547799326;

    ppu_blocking_bgpi_increase for cgb,
    "SameSuite/ppu/blocking_bgpi_increase.gb",
    0,
    8677756512934466165;

    apu_div_write_trigger_10,
    "SameSuite/apu/div_write_trigger_10.gb",
    6901507070137769233,
    15470169245269049758;

    apu_div_write_trigger,
    "SameSuite/apu/div_write_trigger.gb",
    14744474223730903048,
    5536075953610796630;

    apu_div_write_trigger_volume for cgb,
    "SameSuite/apu/div_write_trigger_volume.gb",
    0,
    364719782163348138;

    apu_div_write_trigger_volume_10 for cgb,
    "SameSuite/apu/div_write_trigger_volume_10.gb",
    0,
    4353720675538315229;

    apu_div_trigger_volume_10 for cgb,
    "SameSuite/apu/div_trigger_volume_10.gb",
    0,
    4353720675538315229;
);
