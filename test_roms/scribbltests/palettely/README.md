# PaletteLY

The PaletteLY test is intended to test basic functionality of the BGP register and STAT/VBlank interrupts. It draws solid colored tiles to the background and changes the value of the BGP register every 8 scanlines using LY=LYC interrupts.

## Verified on:

* ✔ Gameboy Pocket (MGB 9638 D)
* ✔ Gameboy Color (CPU CGB D)

## Minimum Requirements

* **CPU:**
  * Functional Instructions
  * Basic Instruction Timing
  * Memory Access Timing **not** required
* **Interrupts:**
  * VBlank Interrupt
  * LYC=LY STAT Interrupt
* **PPU:**
  * LCDC Bits 0, 4 and 7
  * Functional LY Register
  * Functional Background Display
  * Functional Background Palette (BGP)

## Expected Output

![expected](./screenshots/expected.png)

## Common Error Outputs

### Broken LY=LYC Interrupts / BGP

![noint_nobgp](./screenshots/noint_nobgp.png)

This screen may occur if LY=LYC interrupts aren't fired (thereby not modifying the BGP register) or the BGP register is ignored when rendering frames.