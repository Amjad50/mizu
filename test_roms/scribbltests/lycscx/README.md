# LYCSCX

The LYCSXC test is intended to test basic functionality of SCX and LY=LYC STAT interrupts. It draws vertical lines onto the screen and changes SCX every 8 scanlines using LY=LYC interrupts.

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
  * Horizontal Scrolling (SCX Register)

## Expected Output

![expected](./screenshots/expected.png)

## Common Error Outputs

### Broken LY=LYC Interrupts / SCX

![noint_noscx](./screenshots/noint_noscx.png)

This screen may occur if background scrolling doesn't work correctly or LY=LYC interrupts aren't fired (thereby not incrementing the SCX register).