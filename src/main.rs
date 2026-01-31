#![no_main]
#![no_std]

mod life;
use life::*;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use cortex_m_rt::entry;
use microbit::{
    board::{Board, Buttons},
    display::blocking::Display, 
    hal::{
        Rng as HwRng,
        //timer::{Timer, DelayNs},
        timer::Timer,
        gpio,
    },
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use nanorand::{pcg64::Pcg64, Rng, SeedableRng};

const FRAME: u32 = 500;
const WAIT_TIME: u16 = 5;
const EMPTY: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

fn randomize_board(rng: &mut Pcg64, board: &mut [[u8; 5]; 5]) {
    for row in 0 .. 5 {
        for col in 0 .. 5 {
            if rng.generate() {
                board[row][col] = 1;
            } else {
                board[row][col] = 0;
            }
        }
    }
}

fn invert_board(board: &mut [[u8; 5]; 5]) {
    for row in 0 .. 5 {
        for col in 0 .. 5 {
            board[row][col] = if board[row][col] == 1 {0} else {1};
        }
    }
}

#[derive(Clone, Copy)] //automatically implement traits Clone and Copy
enum Ignore {
    Accept,
    Block(u16),
}

impl Ignore {
    fn flip(self) -> Self {
        match self {
            Ignore::Accept => Ignore::Block(WAIT_TIME),
            Ignore::Block(_) => Ignore::Accept,
        }
    }

    fn wait_one(self) -> Self {
        match self {
            Ignore::Accept => Ignore::Accept,
            Ignore::Block(ticks) => Ignore::Block(ticks.max(1) - 1),
        }
    }
}
            


#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut rng = nanorand::Pcg64::new_seed(1);

    let mut button_a = board.buttons.button_a;
    let mut button_b = board.buttons.button_b;

    //get random bool
    //let b: bool = rng.generate();

    let mut leds = [
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 1, 0],
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0],
    ];

    //randomize_board(& mut rng, & mut leds);

    let mut state = Ignore::Accept;

    loop {
        if button_a.is_low().unwrap() {
            randomize_board(& mut rng, & mut leds);
        }
        let button_pressed = button_b.is_low().unwrap();
        match (button_pressed, state) {
            (true, Ignore::Accept) => { invert_board(&mut leds); }

            (_ , Ignore::Block(ticks)) => {
                if ticks == 0 { state = state.flip(); }
                else { state = state.wait_one(); }
            }
            (false, Ignore::Accept) => {} 
        }

        if leds == EMPTY {
            rprintln!("Empty")
        }

        display.show(&mut timer, leds, FRAME);
        life(&mut leds);
        //timer.delay_ms(FRAME);
    }

}


