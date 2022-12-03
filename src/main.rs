#![no_std]
#![no_main]

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use embedded_hal::digital::v2::OutputPin;
use fugit::ExtU32;
use hal::pio::PIOExt;
use hal::Timer;
use panic_halt as _;
use seeeduino_xiao_rp2040::entry;
use seeeduino_xiao_rp2040::hal;
use seeeduino_xiao_rp2040::hal::pac;
use seeeduino_xiao_rp2040::hal::prelude::*;

mod ws2812;

#[entry]
fn main() -> ! {
    const ON: u32 = 0x05u32;
    const OF: u32 = 0x00u32;
    const WHITE: u32 = ((ON / 3) << 24) | ((ON / 3) << 16) | ((ON / 3) << 8);
    const RED: u32 = (ON << 24) | (OF << 16) | (OF << 8);
    const GREEN: u32 = (OF << 24) | (ON << 16) | (OF << 8);
    const BLUE: u32 = (OF << 24) | (OF << 16) | (ON << 8);
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

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut count_down = timer.count_down();

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut tx = ws2812::init(
        pins.neopixel_data.into_mode(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
    );

    // Turn on Neopixel RGB LED
    let mut neopixel_power = pins.neopixel_power.into_push_pull_output();
    neopixel_power.set_high().unwrap();

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
        count_down.start(60u32.micros());
        let _ = nb::block!(count_down.wait());
        while !tx.write(BLUE) {
            cortex_m::asm::nop();
        }
        delay.delay_ms(500);

        // Set USER LED to red
        led_blue_pin.set_high().unwrap();
        led_red_pin.set_low().unwrap();
        led_green_pin.set_high().unwrap();
        // Set RGB LED to red
        count_down.start(60u32.micros());
        let _ = nb::block!(count_down.wait());
        tx.write(RED);
        delay.delay_ms(500);

        // Set USER LED to green
        led_blue_pin.set_high().unwrap();
        led_red_pin.set_high().unwrap();
        led_green_pin.set_low().unwrap();
        // Set RGB LED to green
        count_down.start(60u32.micros());
        let _ = nb::block!(count_down.wait());
        tx.write(GREEN);
        delay.delay_ms(500);

        // Set USER LED to white
        led_blue_pin.set_low().unwrap();
        led_red_pin.set_low().unwrap();
        led_green_pin.set_low().unwrap();
        // Set RGB LED to white
        count_down.start(60u32.micros());
        let _ = nb::block!(count_down.wait());
        tx.write(WHITE);
        delay.delay_ms(500);
    }
}
