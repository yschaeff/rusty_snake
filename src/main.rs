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
    x: usize,
    y: usize,
}

#[derive(Default)]
struct GameState {
    width:  usize,
    height: usize,
    head: Position,
    apple: Position,
    board: Vec<Vec<u32>>,
    score: u32,
    moves: u32,
}

const B_APPLE:u32 = 0b00000001;
const B_DIR  :u32 = 0b00000110;
const B_COUNT:u32 = 0b1111111111111000;

const LEFT :u32 = 0b000;
const RIGHT:u32 = 0b110;
const UP   :u32 = 0b100;
const DOWN :u32 = 0b010;

fn board_init(width: usize, height: usize) -> GameState {
    let x = rand::random::<usize>()%width;
    let y = rand::random::<usize>()%height;
    let mut state = GameState{
        width: width,
        height: height,
        board: vec![vec![0u32; width]; height],
        head: Position{x:x, y:y},
        ..Default::default()
    };
    state.board[y][x] = RIGHT | (1<<3);
    place_random_apple(&mut state);
    state
}

fn place_random_apple(state: &mut GameState) {
    //TODO will get harder of snake longer

    //if state.board[0][state.width-1] & B_COUNT == 0 {
        //state.apple.x = state.width-1;
        //state.apple.y = 0;
        //state.board[0][state.width-1] = B_APPLE;
        //return state;
    //}

    let (x,y) = loop {
        let x = rand::random::<usize>()%state.width;
        let y = rand::random::<usize>()%state.height;
        if state.board[y][x] & B_COUNT == 0 {
            break (x, y)
        }
    };
    state.apple.x = x;
    state.apple.y = y;
    state.board[y][x] = B_APPLE;
}

fn has(cell:u32, flag:u32) -> bool {
    cell & flag == flag
}

fn next(state: &GameState, dir: u32) -> Position {
    let x = state.head.x as i32;
    let y = state.head.y as i32;
    let (xn, yn):(i32, i32) = match dir {
        LEFT  => (x-1, y),
        RIGHT => (x+1, y),
        UP    => (x,   y-1),
        DOWN  => (x,   y+1),
        _  => panic!("Not a valid direction {}", dir),
    };
    Position{x:xn as usize, y:yn as usize}
}

fn previous(state: &GameState, pos: Position) -> Position {
    let x = pos.x;
    let y = pos.y;
    let dir = !state.board[y][x] & B_DIR;
    match dir {
        LEFT  => Position{x: x-1, y: y},
        RIGHT => Position{x: x+1, y: y},
        UP    => Position{x: x,   y: y-1},
        DOWN  => Position{x: x,   y: y+1},
        _  => panic!("Not a valid direction {}", dir),
    }

}

fn in_bounds(state: &GameState, pos: Position) -> bool {
    pos.x < state.width && pos.y < state.height && (pos.x as isize) >= 0 && (pos.y as isize) >= 0
}

fn draw(state: &GameState) {
    for _ in 0..state.width*3+2 { print!("-"); } println!("");
    for (y, row) in (&state).board.iter().enumerate() {
        print!("|");
        for (x, cell) in row.iter().enumerate() {
            if cell & B_COUNT == 0 { //not a snake
                if has(*cell, B_APPLE) {
                    print!(" ø ");
                } else {
                    print!("   ");
                }
            } else {
                if (state.board[y][x] & B_COUNT)>>3 == 1 {
                    print!(" + ");
                } else if (state.board[y][x] & B_COUNT)>>3 == 2 {
                    print!(" * ");
                } else if x == state.head.x && y == state.head.y {
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
        print!("|");
        println!("");
    }
    for _ in 0..state.width*3+2 { print!("-"); } println!("");
    println!("Apples: {}, Moves: {}, Moves/apple: {}", state.score, state.moves, state.moves as f32 / state.score as f32);
}

#[allow(dead_code)]
fn snake_ai_straight(state: &GameState) -> u32 {
    let x = state.head.x;
    let y = state.head.y;
    state.board[y][x] & B_DIR
}
#[allow(dead_code)]
fn snake_ai_random(state: &GameState) -> u32 {
    let dir = rand::random::<u32>()%state.width as u32 & B_DIR;
    dir
}
#[allow(dead_code)]
fn snake_ai_greedy(state: &GameState) -> u32 {
    let dx = state.apple.x as i32 - state.head.x as i32;
    let dy = state.apple.y as i32 - state.head.y as i32;
    if dx.abs() > dy.abs() {
        if dx > 0 {
            RIGHT
        } else {
            LEFT
        }
    } else {
        if dy > 0 {
            DOWN
        } else {
            UP
        }
    }
}

#[allow(dead_code)]
fn snake_ai_greedy_avoid_self(state: &GameState) -> u32 {
    fn cost(state: &GameState, dir: u32) -> i32 {
        let pos = next(&state, dir);
        if !in_bounds(&state, pos) {
            return 999;
        }
        if (state.board[pos.y][pos.x] & B_COUNT) != 0 { return 999; }
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

impl Odd for usize {
    fn odd(&self) -> bool {
        self&1 == 1
    }
    fn even(&self) -> bool {
        self&1 == 0
    }
}

#[allow(dead_code)]
fn snake_ai_hamiltonian(state: &GameState) -> u32 {
    let x = state.head.x;
    let y = state.head.y;
    let w = state.width;
    let h = state.height;

    //fn even(n: usize) -> boolkkk

    if y == 0 { //go left
        if x > 0 {
            return LEFT;
        } else {
            return DOWN;
        }
    } else { //main
        if x == w-1 { //last column
            if x.odd() { //straight up!
                return UP;
            } else { //zig(-zag)
                if (h - y).odd() {
                    return UP;
                } else {
                    //CORNER case if w*h is odd
                    if y == 1 && w.odd() && h.odd() && state.apple.y == 0 {
                        return UP;
                    } else {
                        return LEFT;
                    }
                }
            }
        } else if x == w-2 && w.odd() { //last column
            if (h - y).even() {
                return UP;
            } else {
                return RIGHT;
            }
        } else if x.odd() {
            if y > 1 {
                return UP;
            } else {
                return RIGHT;
            }
        } else {
            if y < h-1 {
                return DOWN;
            } else {
                return RIGHT;
            }
        }
    }
}

fn main() {
    const WIDTH:usize = 7;
    const HEIGHT:usize = 5;

    let mut state = board_init(WIDTH, HEIGHT);
    draw(&state);

    loop {
        if state.score as usize == WIDTH*HEIGHT-1 {
            println!("VICTORY. Ate last apple");
            break;
        }
        state.moves += 1;
        //as AI for move
        //let dir = snake_ai_straight(&state);
        //let dir = snake_ai_random(&state);
        //let dir = snake_ai_greedy(&state);
        //let dir = snake_ai_greedy_avoid_self(&state);
        let dir = snake_ai_hamiltonian(&state);

        let new_pos = next(&state, dir);
        if !in_bounds(&state, new_pos) {
            println!("DEAD. Ran into a wall");
            break;
        }
        if (state.board[new_pos.y][new_pos.x] & B_COUNT)>>3 > 1 { //allow for very tip of the tail
            println!("DEAD. Ran into a snake");
            //println!("dir: {}", dir);
            break;
        }
        //use of new_pos head is fine.
        let head = (state.board[state.head.y][state.head.x] & B_COUNT) | dir | (state.board[new_pos.y][new_pos.x] & B_APPLE);
        state.board[new_pos.y][new_pos.x] = head;

        let head = state.board[new_pos.y][new_pos.x];
        if has(head, B_APPLE) {
            state.board[new_pos.y][new_pos.x] += 1<<3;
            //generate new apple.
            state.score += 1;
            if state.score as usize != WIDTH*HEIGHT-1 {
                place_random_apple(&mut state);
            }
            state.board[new_pos.y][new_pos.x] &= !B_APPLE; //clear apple
        } else { //decrement tail
            let mut pos = state.head;
            loop {
                state.board[pos.y][pos.x] -= 1<<3;
                //state.board[pos.y][pos.x] &= !B_APPLE; //clear apple
                if (state.board[pos.y][pos.x] & B_COUNT) == 0 {
                    state.board[pos.y][pos.x] = 0;
                    break;
                }
                pos = previous(&state, pos);
                if pos.x == state.head.x && pos.y == state.head.y {
                    //corner case where snake follows tail closely
                    break;
                }
            }
        }
        state.head = new_pos;

        thread::sleep(time::Duration::from_millis(50));
        print!("{}[2J", 27 as char);
        draw(&state);
    }
}
