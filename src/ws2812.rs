use rp2040_hal::{
    gpio::bank0::Gpio0,
    gpio::{Pin, PinId, PinMode, ValidPinMode},
    pio::PIOExt,
    pio::{PIOBuilder, ShiftDirection, Tx, UninitStateMachine, PIO, SM0},
};

pub fn init<I, M, P>(
    _pin: Pin<I, M>,
    pio: &mut PIO<P>,
    sm: UninitStateMachine<(P, SM0)>,
    clock_freq: fugit::HertzU32,
) -> Tx<(P, SM0)>
where
    I: PinId,
    M: PinMode + ValidPinMode<I>,
    P: PIOExt,
{
    const CYCLES_PER_BIT: u32 = (10) as u32;
    const FREQ: u32 = 800_000;
    let program = pio_proc::pio_asm!(
        ".side_set 1",
        "",
        ".define public T1 2",
        ".define public T2 5",
        ".define public T3 3",
        "",
        ".wrap_target",
        "bitloop:",
        "    out x, 1       side 0 [T3 - 1] ; Side-set still takes place when instruction stalls",
        "    jmp !x do_zero side 1 [T1 - 1] ; Branch on the bit we shifted out. Positive pulse",
        "do_one:",
        "    jmp  bitloop   side 1 [T2 - 1] ; Continue driving high, for a long pulse",
        "do_zero:",
        "    nop            side 0 [T2 - 1] ; Or drive low, for a short pulse",
        ".wrap",
    );

    let div = clock_freq.to_Hz() / (FREQ * CYCLES_PER_BIT);
    let installed = pio.install(&program.program).unwrap();
    let (mut sm, _, tx) = PIOBuilder::from_program(installed)
        .side_set_pin_base(Gpio0::DYN.num)
        .autopull(true)
        .pull_threshold(24)
        .out_shift_direction(ShiftDirection::Right) // default is left
        .clock_divisor(div as f32)
        .build(sm);
    sm.set_pindirs([(Gpio0::DYN.num, rp2040_hal::pio::PinDir::Output)]);
    sm.start();
    tx
}
