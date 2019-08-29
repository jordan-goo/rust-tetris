extern crate termion;

use std::cmp::max;
use std::io::{stdin, stdout, Write};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};

type Pos = (i16, i16);
type TetPos = [Pos; 4];

struct Tetromino {
    rotation: i32,
    position: Box<Fn(Pos) -> TetPos>,
}

enum TetType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

fn gen(t: TetType) -> Tetromino {
    Tetromino {
        rotation: 0,
        position: Box::new(move |(x, y): Pos| -> TetPos {
            match t {
                TetType::O => [(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)],
                TetType::T => [(x, y), (x + 1, y), (x + 2, y), (x + 1, y + 1)],
                TetType::S => [(x, y), (x + 1, y), (x + 1, y + 1), (x + 2, y + 1)],
                TetType::Z => [(x, y), (x + 1, y), (x + 1, y - 1), (x + 2, y - 1)],
                TetType::J => [(x, y), (x, y - 1), (x + 1, y - 1), (x + 2, y - 1)],
                TetType::L => [(x, y), (x + 1, y), (x + 2, y), (x + 2, y + 1)],
                TetType::I => [(x, y), (x + 1, y), (x + 2, y), (x + 3, y)],
            }
        }),
    }
}

fn render_empty(x: i16, y: i16) {
    //cursor is 1 indexed!
    print!(
        "{}-",
        cursor::Goto(max(x + 1, 0) as u16, max(y + 1, 0) as u16)
    );
}

fn render_active(x: i16, y: i16) {
    print!(
        "{}{}#{}",
        cursor::Goto(max(x + 1, 0) as u16, max(y + 1, 0) as u16),
        color::Fg(color::Yellow),
        style::Reset
    );
}

fn render_locked(x: i16, y: i16) {
    print!(
        "{}{}#{}",
        cursor::Goto(max(x + 1, 0) as u16, max(y + 1, 0) as u16),
        color::Fg(color::Blue),
        style::Reset
    );
}

fn render(active: &[Pos], locked: &[Pos]) {
    //reset output
    print!("{}{}", clear::All, cursor::Goto(1, 1));

    for y in 0..H {
        for x in 0..W {
            render_empty(x as i16, y as i16);
        }
    }

    active.iter().for_each(|(x, y)| {
        render_active(*x, *y);
    });

    locked.iter().for_each(|(x, y)| {
        render_locked(*x, *y);
    });

    print!("{}{}", style::Reset, cursor::Goto(1, H + 2));
    //do i need this here?
    stdout().flush().unwrap();
}

const W: u16 = 16;
const H: u16 = 20;

fn main() {
    // Get the standard output stream and go to raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();
    // Use asynchronous stdin
    let mut stdin = termion::async_stdin().keys();
    //clears / resets display
    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

    let mut active_blocks: Vec<Pos> = vec![];
    let mut locked_blocks: Vec<Pos> = vec![];

    let p = gen(TetType::O);
    let current = (p.position)((0, 0));
    active_blocks.extend(&current);

    let mut tick = 1;

    loop {
        if tick > 500 {
            break;
        }
        if locked_blocks.iter().any(|&(_, y)| y <= 1) {
            //game over
            break;
        }

        let mut increment = (0, 0);
        if tick % 2 == 0 {
            //move blocks down half as fast as input
            increment.1 += 1;
        }

        tick += 1;
        render(&active_blocks, &locked_blocks);

        // Read input (if any)
        let input = stdin.next();

        // If a key was pressed
        if let Some(Ok(key)) = input {
            match key {
                Key::Char('q') => break,
                Key::Left => {
                    increment.0 += -1;
                }
                Key::Right => {
                    increment.0 += 1;
                }
                _ => {}
            }
        }

        //would be nice to have destructuring assignment here
        let (new_active_blocks, new_locked_blocks) =
            move_blocks(&active_blocks, &locked_blocks, increment);

        active_blocks = new_active_blocks;
        locked_blocks = new_locked_blocks;

        if active_blocks.is_empty() {
            active_blocks.extend(&current);
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn move_blocks(
    active: &Vec<Pos>,
    locked: &Vec<Pos>,
    (increment_x, increment_y): (i16, i16),
) -> (Vec<Pos>, Vec<Pos>) {
    let mut new_active: Vec<Pos> = active
        .iter()
        .map(|(x, y)| (x + increment_x, y + increment_y))
        .collect();
    let mut new_locked = locked.clone();

    if new_active
        .iter()
        .any(|&(x, y)| y >= H as i16 - 1 || locked.contains(&(x, y + 1)))
    {
        //moves all from active to locked, leaving active empty
        new_locked.append(&mut new_active);
        return (new_active, new_locked);
    }

    (new_active, new_locked)
}
