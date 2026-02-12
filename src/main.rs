#![no_main]
#![no_std]
#![allow(clippy::needless_range_loop)]

mod life;
use life::*;

use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{Rng as HwRng, timer::Timer},
};
use nanorand::{Rng, pcg64::Pcg64};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

const FRAME: u32 = 100;
const WAIT_TIME: u16 = 4;
//there is a cleaner function for determining if game is empty in life.rs
//but I didn't see it until after I was finished
const EMPTY: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

fn randomize_board(rng: &mut Pcg64, board: &mut [[u8; 5]; 5]) {
    for row in 0..5 {
        for col in 0..5 {
            if rng.generate() {
                board[row][col] = 1;
            } else {
                board[row][col] = 0;
            }
        }
    }
}

fn invert_board(board: &mut [[u8; 5]; 5]) {
    for row in 0..5 {
        for col in 0..5 {
            board[row][col] = if board[row][col] == 1 { 0 } else { 1 };
        }
    }
}

//Status gives us the ability to make things persist through the loops
//by using counters
#[derive(Clone, Copy)] //automatically implement traits Clone and Copy
enum Status {
    Normal,
    Wait(u16),
}

impl Status {
    fn flip(self) -> Self {
        match self {
            Status::Normal => Status::Wait(WAIT_TIME),
            Status::Wait(_) => Status::Normal,
        }
    }

    fn wait_one(self) -> Self {
        match self {
            Status::Normal => Status::Normal,
            Status::Wait(ticks) => Status::Wait(ticks.max(1) - 1),
        }
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut hardware_rng: HwRng = HwRng::new(board.RNG);
    //this gets two random u64 ints in the bottom half of the u128 bits
    //shuffles one to the top half and bitwise ORs them together for a single
    //random u128 bit to set the seed on nrg
    let hi = hardware_rng.random_u64() as u128;
    let lo = hardware_rng.random_u64() as u128;
    let seed: u128 = (hi << 64) | lo;
    let mut rng = nanorand::Pcg64::new_seed(seed);

    let mut button_a = board.buttons.button_a;
    let mut button_b = board.buttons.button_b;

    //I like the glider for the starting place instead of random
    let mut leds = if cfg!(feature = "start-glider") {
        [
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 1, 0],
            [0, 1, 1, 1, 0],
            [0, 0, 0, 0, 0],
        ]
    } else {
        [[0; 5]; 5]
    };

    //state blockes the b button,
    //board_state allows 5 frames of unlit LED matrix
    let mut state = Status::Normal;
    let mut board_state = Status::Normal;

    loop {
        //handles button A presses
        if button_a.is_low().unwrap() {
            rprintln!("A button acknowledged");
            randomize_board(&mut rng, &mut leds);
        }

        //handles button b presses
        let button_pressed = button_b.is_low().unwrap();
        match (button_pressed, state) {
            (true, Status::Normal) => {
                rprintln!("B button acknowledged");
                invert_board(&mut leds);
                state = state.flip();
            }
            (_, Status::Wait(ticks)) => {
                rprintln!("B button blocked");
                if ticks == 0 {
                    state = state.flip();
                } else {
                    state = state.wait_one();
                }
            }
            (false, Status::Normal) => {}
        }

        //handles case of empty board
        if leds == EMPTY {
            rprintln!("Empty");
            match board_state {
                Status::Normal => {
                    rprintln!("Starting count at 5");
                    board_state = board_state.flip();
                }
                Status::Wait(ticks) => {
                    rprintln!("num ticks: {}", ticks);
                    if ticks == 0 {
                        rprintln!("applying new pattern");
                        board_state = board_state.flip();
                        randomize_board(&mut rng, &mut leds);
                    } else {
                        board_state = board_state.wait_one();
                    }
                }
            }
        }

        display.show(&mut timer, leds, FRAME);
        life(&mut leds);
    }
}
