#![no_main]
#![no_std]
#![allow(clippy::needless_range_loop)]

mod life;
use life::*;

//use embedded_hal::delay::DelayNs;
//use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::digital::InputPin;
use cortex_m_rt::entry;
use microbit::{
    //board::{Board, Buttons},
    board::Board,
    display::blocking::Display, 
    hal::{
        Rng as HwRng,
        timer::Timer,
    },
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
//use nanorand::{pcg64::Pcg64, Rng, SeedableRng};
use nanorand::{pcg64::Pcg64, Rng};

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
            board[row][col] = if board[row][col] == 1 {0} else {1};
        }
    }
}

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
    let hi = hardware_rng.random_u64() as u128;
    let lo = hardware_rng.random_u64() as u128;
    let seed: u128 = (hi << 64) | lo;
    let mut rng = nanorand::Pcg64::new_seed(seed);

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

    let mut state = Status::Normal;
    let mut board_state = Status::Normal;

    loop {
        //handles button A presses
        if button_a.is_low().unwrap() {
            rprintln!("A button acknowledged");
            randomize_board(& mut rng, & mut leds);
        }

        //handles button b presses
        let button_pressed = button_b.is_low().unwrap();
        match (button_pressed, state) {
            (true, Status::Normal) => { 
                rprintln!("B button acknowledged");
                invert_board(&mut leds);
                state = state.flip();
            }
            (_ , Status::Wait(ticks)) => {
                rprintln!("B button blocked");
                if ticks == 0 { state = state.flip(); }
                else { state = state.wait_one(); }
            }
            (false, Status::Normal) => {} 
        }

        //handles case of empty board
        if leds == EMPTY {
            rprintln!("Empty");
            match board_state {
                Status::Normal => { 
                    board_state = board_state.flip(); 
                }
                Status::Wait(ticks) => {
                    rprintln!("num ticks: {}", ticks);
                    if ticks == 0 {
                        board_state = board_state.flip();
                        randomize_board(& mut rng, & mut leds);
                    }
                    else { board_state = board_state.wait_one(); }
                }
            }
        }

        display.show(&mut timer, leds, FRAME);
        life(&mut leds);
        //timer.delay_ms(FRAME);
    }

}


