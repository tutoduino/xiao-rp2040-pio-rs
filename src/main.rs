#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use hal::pio::PIOExt;
use panic_halt as _;
use seeeduino_xiao_rp2040::entry;
use seeeduino_xiao_rp2040::hal;
use seeeduino_xiao_rp2040::hal::pac;
use seeeduino_xiao_rp2040::hal::prelude::*;

mod ws2812;

#[entry]
fn main() -> ! {
    const WHITE: u32 = 0x808080; // lower brightness, 0xff max brightness (blinding)
    const RED: u32 = 0x008000;
    const BLUE: u32 = 0x800000;
    const GREEN: u32 = 0x000080;
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        seeeduino_xiao_rp2040::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = seeeduino_xiao_rp2040::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut tx = ws2812::init(
        pins.neopixel_data,
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
    );

    // Configure the USER LED pins to operate as a push-pull output
    let mut led_blue_pin = pins.led_blue.into_push_pull_output();
    let mut led_green_pin = pins.led_green.into_push_pull_output();
    let mut led_red_pin = pins.led_red.into_push_pull_output();

    loop {
        // Set USER LED to blue
        led_blue_pin.set_low().unwrap();
        led_red_pin.set_high().unwrap();
        led_green_pin.set_high().unwrap();
        // Set RGB LED to blue
        tx.write(BLUE);
        delay.delay_ms(500);

        // Set USER LED to red
        led_blue_pin.set_high().unwrap();
        led_red_pin.set_low().unwrap();
        led_green_pin.set_high().unwrap();
        // Set RGB LED to red
        tx.write(RED);
        delay.delay_ms(500);

        // Set USER LED to green
        led_blue_pin.set_high().unwrap();
        led_red_pin.set_high().unwrap();
        led_green_pin.set_low().unwrap();
        // Set RGB LED to green
        tx.write(GREEN);
        delay.delay_ms(500);

        // Set USER LED to white
        led_blue_pin.set_low().unwrap();
        led_red_pin.set_low().unwrap();
        led_green_pin.set_low().unwrap();
        // Set RGB LED to white
        tx.write(WHITE);
        delay.delay_ms(500);
    }
}
