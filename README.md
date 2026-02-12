
# Game of Life

Garrett Reiter

The Game of Life program uses the rules of Jon Conway's "Game of Life" to manipulate
a randomly generated 5x5 LED matrix over 100ms frames.

I made a simple state machine to detect and delay input from button B, which inverts the
LEDS from on to off and vice versa.

The A button randomized which LEDs are turned on.

How it went:

I've struggled to find relevant information in the Rust docs, for example,
https://docs.rs/microbit-v2/latest/microbit/board/struct.Buttons.html
In the board Button docs, I don't see `.is_low()` documented. I just used it 
because that's what's in our book.
