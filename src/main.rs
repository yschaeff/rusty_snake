/* How is the snake represented?
 *
 * The head of the snake is known (location).
 * Each segment of the snake stores the direction of the snake, pointing towards
 * its tail. The direction is chosen such that we can invert it easily.
 * 00: left
 * 11: right
 * 01: down
 * 10: up
 *
 * */

use std::{thread, time};

#[derive(Copy, Clone, Default)]
struct Position {
    x: isize,
    y: isize,
}

#[derive(Default)]
struct GameState {
    width:  usize,
    height: usize,
    head: Position,
    apple: Position,
    board: Vec<Vec<u32>>,
    moves: u32,
    length: u32,
}

const B_APPLE:u32 = 0b00000001;
const B_DIR  :u32 = 0b00000110;
const B_COUNT:u32 = 0b1111111111111000;

const LEFT :u32 = 0b000;
const RIGHT:u32 = 0b110;
const UP   :u32 = 0b100;
const DOWN :u32 = 0b010;

impl GameState {
    fn init(width: usize, height: usize) -> GameState {
        let x = rand::random::<usize>()%width;
        let y = rand::random::<usize>()%height;
        let mut state = GameState{
            width: width,
            height: height,
            board: vec![vec![0u32; width]; height],
            head: Position{x:x as isize, y:y as isize},
            length: 1,
            ..Default::default()
        };
        state.board[y][x] = RIGHT | (1<<3);
        state.place_random_apple();
        state
    }

    fn draw(&self) {
        for _ in 0..self.width*3+2 { print!("-"); } println!("");
        for (y, row) in self.board.iter().enumerate() {
            print!("|");
            for (x, cell) in row.iter().enumerate() {
                if cell & B_COUNT == 0 { //not a snake
                    if has(*cell, B_APPLE) {
                        print!(" ø ");
                    } else {
                        print!("   ");
                    }
                } else {
                    if (self.board[y][x] & B_COUNT)>>3 == 1 {
                        print!(" + ");
                    } else if (self.board[y][x] & B_COUNT)>>3 == 2 {
                        print!(" * ");
                    } else if x as isize == self.head.x && y as isize == self.head.y {
                        print!(" # ");
                    } else if has(*cell, B_APPLE) { //swallowed apple.
                        print!(" o ");
                    } else {
                        let dir = *cell & B_DIR;
                        match dir {
                            LEFT  => print!(" ← "),
                            RIGHT => print!(" → "),
                            UP    => print!(" ↑ "),
                            DOWN  => print!(" ↓ "),
                            _  => print!("error"),
                        }
                    }
                }
            }
            println!("|");
        }
        for _ in 0..self.width*3+2 { print!("-"); } println!("");
        println!("Apples: {}, Moves: {}, Moves/apple: {}", self.length, self.moves, self.moves as f32 / self.length as f32);
    }

    fn next(&self, dir: u32) -> Position {
        let (xn, yn) = match dir {
            LEFT  => (self.head.x-1, self.head.y),
            RIGHT => (self.head.x+1, self.head.y),
            UP    => (self.head.x,   self.head.y-1),
            DOWN  => (self.head.x,   self.head.y+1),
            _  => panic!("Not a valid direction {}", dir),
        };
        Position{x:xn, y:yn}
    }

    fn previous(&self, pos: Position) -> Position {
        let x = pos.x;
        let y = pos.y;
        let dir = !self.board[y as usize][x as usize] & B_DIR;
        match dir {
            LEFT  => Position{x: x-1, y: y},
            RIGHT => Position{x: x+1, y: y},
            UP    => Position{x: x,   y: y-1},
            DOWN  => Position{x: x,   y: y+1},
            _  => panic!("Not a valid direction {}", dir),
        }
    }

    fn place_random_apple(&mut self) {
        //TODO will get harder of snake longer

        let (x,y) = loop {
            let x = rand::random::<usize>()%self.width;
            let y = rand::random::<usize>()%self.height;
            if self.board[y][x] & B_COUNT == 0 {
                break (x, y)
            }
        };
        self.apple.x = x as isize;
        self.apple.y = y as isize;
        self.board[y][x] = B_APPLE;
    }

    fn in_bounds(&self, pos: Position) -> bool {
        pos.x < self.width as isize && pos.y < self.height as isize && pos.x >= 0 && pos.y >= 0
    }
}

fn has(cell:u32, flag:u32) -> bool {
    cell & flag == flag
}

#[allow(dead_code)]
fn snake_ai_straight(state: &GameState) -> u32 {
    let x = state.head.x;
    let y = state.head.y;
    state.board[y as usize][x as usize] & B_DIR
}
#[allow(dead_code)]
fn snake_ai_random(state: &GameState) -> u32 {
    rand::random::<u32>()%state.width as u32 & B_DIR
}
#[allow(dead_code)]
fn snake_ai_greedy(state: &GameState) -> u32 {
    let dx = state.apple.x as i32 - state.head.x as i32;
    let dy = state.apple.y as i32 - state.head.y as i32;
    return if dx.abs() > dy.abs() {
        if dx > 0 { RIGHT } else { LEFT }
    } else {
        if dy > 0 { DOWN } else { UP }
    };
}

#[allow(dead_code)]
fn snake_ai_greedy_avoid_self(state: &GameState) -> u32 {
    fn cost(state: &GameState, dir: u32) -> i32 {
        let pos = state.next(dir);
        if !state.in_bounds(pos) {
            return 999;
        }
        if (state.board[pos.y as usize][pos.x as usize] & B_COUNT) != 0 { return 999; }
        let dx = state.apple.x as i32 - pos.x as i32;
        let dy = state.apple.y as i32 - pos.y as i32;
        return dx.abs() + dy.abs();
    }

    let dirs = vec![LEFT, RIGHT, UP, DOWN];
    dirs.into_iter().min_by_key(|x| cost(&state, *x)).unwrap()
}

trait Odd {
    fn odd(&self) -> bool;
    fn even(&self) -> bool;
}

impl Odd for isize {
    fn odd(&self) -> bool {
        self&1 == 1
    }
    fn even(&self) -> bool {
        self&1 == 0
    }
}

struct Snake {
    path: Vec<Vec<u32>>,
}

impl Snake {
    fn make_path(&mut self, state: &GameState) {
        let mut x = 0;
        let mut y = 0;
        let w = state.width as isize;
        let h = state.height as isize;

        loop {
            let dir =
                if y == 0 { //go left
                    if x > 0 { LEFT } else { DOWN }
                } else if x == w-1 { //last column
                    if x.odd() { //straight up!
                        UP
                    } else { //zig(-zag)
                        if (h - y).odd() {
                            UP
                        } else {
                            //CORNER case if w*h is odd
                            //if y == 1 && w.odd() && h.odd() && state.apple.y == 0 { UP } else { LEFT }
                            LEFT
                        }
                    }
                } else if x == w-2 && w.odd() { //last column
                    if (h - y).even() { UP } else { RIGHT }
                } else if x.odd() {
                    if y > 1 { UP } else { RIGHT }
                } else {
                    if y < h-1 { DOWN } else { RIGHT }
                };
            self.path[y as usize][x as usize] = dir;
            match dir {
                LEFT  => {x=x-1; y=y-0},
                RIGHT => {x=x+1; y=y-0},
                UP    => {x=x-0; y=y-1},
                DOWN  => {x=x-0; y=y+1},
                _  => print!("error"),
            }
            if (x, y) == (0, 0) {
                break;
            }
        }
    }
    fn init(state: &GameState) -> Snake {
        let mut snake = Snake {
            path: vec![vec![0u32; state.width]; state.height],
        };
        snake.make_path(state);
        snake
    }
    fn next(&self, state:&GameState) -> u32 {
        self.path[state.head.y as usize][state.head.x as usize]
    }
}

#[allow(dead_code)]
fn snake_ai_hamiltonian(state: &GameState) -> u32 {
    let x = state.head.x;
    let y = state.head.y;
    let w = state.width as isize;
    let h = state.height as isize;

    if y == 0 { //go left
        if x > 0 { LEFT } else { DOWN }
    } else if x == w-1 { //last column
        if x.odd() { //straight up!
            UP
        } else { //zig(-zag)
            if (h - y).odd() {
                UP
            } else {
                //CORNER case if w*h is odd
                if y == 1 && w.odd() && h.odd() && state.apple.y == 0 { UP } else { LEFT }
            }
        }
    } else if x == w-2 && w.odd() { //last column
        if (h - y).even() { UP } else { RIGHT }
    } else if x.odd() {
        if y > 1 { UP } else { RIGHT }
    } else {
        if y < h-1 { DOWN } else { RIGHT }
    }
}

fn main() {
    const WIDTH:usize = 9;
    const HEIGHT:usize = 9;

    let mut state = GameState::init(WIDTH, HEIGHT);
    let mut snake = Snake::init(&state);
    state.draw();

    loop {
        //ask AI for move
        //let dir = snake_ai_straight(&state);
        //let dir = snake_ai_random(&state);
        //let dir = snake_ai_greedy(&state);
        //let dir = snake_ai_greedy_avoid_self(&state);
        //let dir = snake_ai_hamiltonian(&state);
        let dir = snake.next(&state);

        state.moves += 1;

        let new_pos = state.next(dir);
        if !state.in_bounds( new_pos) {
            println!("DEAD. Ran into a wall");
            break;
        }
        if (state.board[new_pos.y as usize][new_pos.x as usize] & B_COUNT)>>3 > 1 { //allow for very tip of the tail
            println!("DEAD. Ran into a snake");
            break;
        }

        //use of new_pos head is fine.
        let head = (state.board[state.head.y as usize][state.head.x as usize] & B_COUNT) | dir | (state.board[new_pos.y as usize][new_pos.x as usize] & B_APPLE);
        state.board[new_pos.y as usize][new_pos.x as usize] = head;

        let head = state.board[new_pos.y as usize][new_pos.x as usize];
        let ate_apple = has(head, B_APPLE);
        if ate_apple { //no need to move snake
            state.board[new_pos.y as usize][new_pos.x as usize] += 1<<3;
            state.length += 1;
            state.board[new_pos.y as usize][new_pos.x as usize] &= !B_APPLE; //clear apple
        } else { //Move snake forward head to tail
            let mut pos = state.head;
            loop {
                state.board[pos.y as usize][pos.x as usize] -= 1<<3;
                if (state.board[pos.y as usize][pos.x as usize] & B_COUNT) == 0 {
                    // This was the last bit of tail. we are done.
                    state.board[pos.y as usize][pos.x as usize] = 0;
                    break;
                }
                pos = state.previous(pos);
                // There is a corner case where the snake follows its own tail
                // at distance zero. We must detect the head to avoid getting
                // in to a loop.
                if pos.x == state.head.x && pos.y == state.head.y {
                    break;
                }
            }
        }
        state.head = new_pos;

        if ate_apple {// generate new apple.
            if state.length as usize != WIDTH*HEIGHT-1 {
                state.place_random_apple();
            } else {
                state.draw();
                println!("VICTORY. Ate last apple");
                break;
            }
        }

        thread::sleep(time::Duration::from_millis(60));
        print!("{}[2J", 27 as char);
        state.draw();
    }
}
