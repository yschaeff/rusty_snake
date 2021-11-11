use std::{thread, time};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn odd(value:isize) -> bool {
    value&1 == 1
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
    End,
    Null,
}

impl Direction {
    fn invert(&self) -> Direction {
        match self {
            Direction::Left  => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up    => Direction::Down,
            Direction::Down  => Direction::Up,
            Direction::End   => Direction::End,
            Direction::Null  => Direction::Null,
        }
    }
    fn valid(&self) -> bool {
        match self {
            Direction::End   => false,
            Direction::Null  => false,
            _                => true,
        }
    }
    fn random(rng:&mut StdRng) -> Direction {
        match  rng.gen::<u32>()%4 {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            3 => Direction::Down,
            _ => panic!("you can't even get modulo to work dork!"),
        }
    }
}
impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Direction::Left  => write!(f, "🡸"),
            Direction::Right => write!(f, "🡺"),
            Direction::Up    => write!(f, "🡹"),
            Direction::Down  => write!(f, "🡻"),
            Direction::End   => write!(f, "•"),
            Direction::Null  => write!(f, " "),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Coordinate {
    x: isize,
    y: isize,
}

impl Coordinate {
    fn move_towards(&self, dir:Direction) -> Coordinate {
        match dir {
            Direction::Left  => Coordinate{x:self.x-1, y:self.y},
            Direction::Right => Coordinate{x:self.x+1, y:self.y},
            Direction::Up    => Coordinate{x:self.x,   y:self.y-1},
            Direction::Down  => Coordinate{x:self.x,   y:self.y+1},
            Direction::End   => Coordinate{x:self.x,   y:self.y},
            Direction::Null  => Coordinate{x:self.x,   y:self.y},
        }
    }
    fn random(&self, rng:&mut StdRng) -> Coordinate {
        let x = rng.gen::<usize>()%self.x as usize;
        let y = rng.gen::<usize>()%self.y as usize;
        Coordinate{x:x as isize, y:y as isize}
    }
    fn difference(&self, other:Coordinate) -> Coordinate {
        Coordinate{x:other.x-self.x, y:other.y-self.y}
    }
}
impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

/*
 * a Field is just a grid of directions
 */
struct Field {
    dimension: Coordinate,
    directions: Vec<Vec<Direction>>,
}

impl Field {
    fn init(dimension: Coordinate) -> Field {
        Field{
            dimension: dimension,
            directions: vec![vec![Direction::Null; dimension.x as usize]; dimension.y as usize],
        }
    }
    fn get(&self, position:Coordinate) -> Direction {
        self.directions[position.y as usize][position.x as usize]
    }
    fn set(&mut self, position:Coordinate, direction:Direction) {
        //println!("set to {} at {}", direction, position);
        self.directions[position.y as usize][position.x as usize] = direction;
    }
    fn next(&self, position:Coordinate) -> Coordinate {
        let direction = self.get(position);
        position.move_towards(direction)
    }
    fn valid(&self, position:Coordinate) -> bool {
        position.x >= 0 && position.y >= 0 && position.x < self.dimension.x && position.y < self.dimension.y
    }
    fn available(&self, position:Coordinate) -> bool {
        self.directions[position.y as usize][position.x as usize] == Direction::Null
    }
    fn random_available(&self, rng:&mut StdRng) -> Option<Coordinate> {
        let w = self.dimension.x;
        let h = self.dimension.y;
        let r = self.dimension.random(rng);

        for y in 0..h {
            for x in 0..w {
                let p = Coordinate{x: (x+r.x)%w, y: (y+r.y)%h};
                if self.available(p) {
                    return Some(p);
                }
            }
        }
        None
    }
    /* Follow chain backwards. Drop last segment, return its coordinates */
    fn drop_last(&mut self, start:Coordinate) -> Coordinate {
        let mut a = start;
        let mut b = self.next(a);
        while self.get(b) != Direction::End {
            a = b;
            b = self.next(a);
        }
        self.set(a, Direction::End);
        self.set(b, Direction::Null);
        b
    }
}

struct Game {
    head: Coordinate,
    apple: Coordinate,
    field: Field,
    apples: u32,
    moves: u32,
    rng: StdRng,
}

impl Game {
    fn init(width: usize, height: usize) -> Game {
        //let mut rng = rand::thread_rng();
        let mut rng = StdRng::seed_from_u64(42);
        let field_dimension = Coordinate{x:width as isize, y:height as isize};
        let mut field = Field::init(field_dimension);
        let head = field_dimension.random(&mut rng);
        //let head = Coordinate{x:2, y:5};
        let direction = Direction::End;
        println!("setting head {:?}", head);
        field.set(head, direction);
        let apple_opt = field.random_available(&mut rng);
        let apple = match apple_opt {
            Some(apple) => apple,
            None        => panic!("You goofed"),
        };

        Game{
            head,
            apple,
            field,
            apples: 0,
            moves: 0,
            rng,
        }
    }
    fn new_apple(&mut self) -> bool {
        let apple_opt = self.field.random_available(&mut self.rng);
        self.apple = match apple_opt {
            Some(apple) => apple,
            None        => return false,
        };
        return true;
    }
    fn draw(&self) {
        print!("   "); for i in 0..self.field.dimension.x { print!(" {} ", i%10); } println!("");
        print!("  ┏"); for _ in 0..self.field.dimension.x*3 { print!("━"); } println!("┓");
        for (y, row) in self.field.directions.iter().enumerate() {
            print!("{} ┃", y%10);
            for (x, dir) in row.iter().enumerate() {
                let pos = Coordinate{x:x as isize, y:y as isize};
                if pos == self.head {
                    print!(" # ");
                } else if pos == self.apple {
                    print!(" ø ");
                } else {
                    print!(" {} ", dir.invert());
                }
            }
            println!("┃");
        }
        print!("  ┗"); for _ in 0..self.field.dimension.x*3 { print!("━"); } println!("┛");
        println!("Apples: {}, Moves: {}, Moves/apple: {}", self.apples, self.moves, self.moves as f32 / self.apples as f32);
    }
}

trait Snake {
    fn init(&mut self, game:&Game);
    fn move_to(&self, game:&Game) -> Option<Direction>;
}

struct SillySnake;
impl Snake for SillySnake {
    fn init(&mut self, _game:&Game) {
    }
    fn move_to(&self, _game:&Game) -> Option<Direction> {
        let mut rng = StdRng::from_entropy();
        Some(Direction::random(&mut rng))
    }
}

struct GreedySnake;
impl Snake for GreedySnake {
    fn init(&mut self, _game:&Game) {
    }
    fn move_to(&self, game:&Game) -> Option<Direction> {
        let delta = game.head.difference(game.apple);
        Some(if (delta.x.abs() < delta.y.abs() || delta.y == 0) && delta.x != 0 {
        //if delta.x.abs() > delta.y.abs() {
            if delta.x > 0 { Direction::Right } else { Direction::Left }
        } else {
            if delta.y > 0 { Direction::Down } else { Direction::Up }
        })
    }
}
struct GreedyPickySnake;
impl GreedyPickySnake {
    fn prioritize(snake:Coordinate, apple:Coordinate) -> [Direction; 4] {
        let d1:Direction;
        let d2:Direction;
        let d3:Direction;
        let d4:Direction;

        let delta = snake.difference(apple);
        if (delta.x.abs() < delta.y.abs() || delta.y == 0) && delta.x != 0 {
            d1 = if delta.x >  0 { Direction::Right } else { Direction::Left };
            d2 = if delta.y >  0 { Direction::Down } else { Direction::Up };
            d3 = if delta.y <= 0 { Direction::Down } else { Direction::Up };
            d4 = if delta.x <= 0 { Direction::Right } else { Direction::Left };
        } else {
            d1 = if delta.y >  0 { Direction::Down } else { Direction::Up };
            d2 = if delta.x >  0 { Direction::Right } else { Direction::Left };
            d3 = if delta.x <= 0 { Direction::Right } else { Direction::Left };
            d4 = if delta.y <= 0 { Direction::Down } else { Direction::Up };
        }
        [d1, d2, d3, d4]
    }
    fn available(game:&Game, dir:Direction) -> bool {
        let pos = game.head.move_towards(dir);
        game.field.valid(pos) && game.field.available(pos)
    }
}
impl Snake for GreedyPickySnake {
    fn init(&mut self, _game:&Game) {
    }

    fn move_to(&self, game:&Game) -> Option<Direction> {
        let preferred = GreedyPickySnake::prioritize(game.head, game.apple).into_iter();
        let available = preferred.filter(|dir| GreedyPickySnake::available(game, *dir));
        for dir in available { //return first if list not empty
            return Some(dir);
        }
        None //Give up
    }
}

/* Almost a winning strategy. however at a cost. Expected moves per apple
 * works out to (w*h)/4 */
struct HamiltonianSnake;
impl Snake for HamiltonianSnake {
    fn init(&mut self, _game:&Game) {
    }

    fn move_to(&self, game:&Game) -> Option<Direction> {
        let x = game.head.x;
        let y = game.head.y;
        let w = game.field.dimension.x;
        let h = game.field.dimension.y;

        Some(if y == 0 { //go left
            if x > 0 { Direction::Left } else { Direction::Down }
        } else if x == w-1 { //last column
            if odd(x) { //straight up!
                Direction::Up
            } else { //zig(-zag)
                if odd(h - y) {
                    Direction::Up
                } else {
                    //CORNER case if w*h is odd
                    if y == 1 && odd(w) && odd(h) && game.apple.y == 0 { Direction::Up } else { Direction::Left }
                }
            }
        } else if x == w-2 && odd(w) { //last column
            if !odd(h - y) { Direction::Up } else { Direction::Right }
        } else if odd(x) {
            if y > 1 { Direction::Up } else { Direction::Right }
        } else {
            if y < h-1 { Direction::Down } else { Direction::Right }
        })
    }
}

fn choose_snake(k:u32) -> Box<dyn Snake> {
    match k {
        0 => Box::new(SillySnake{}),
        1 => Box::new(GreedySnake{}),
        2 => Box::new(GreedyPickySnake{}),
        3 => Box::new(HamiltonianSnake{}),
        _ => panic!("Never heard of such snake"),
    }
}

fn main() {
    const WIDTH:usize = 9;
    const HEIGHT:usize = 7;

    let mut game = Game::init(WIDTH, HEIGHT);
    let mut snake = choose_snake(3);
    snake.init(&game);

    game.draw();
    loop {
        let snake_dir = match snake.move_to(&game) {
            Some(dir) => dir,
            None => {
                println!("Snake forfeit.");
                break; }};
        if !snake_dir.valid() {
            println!("Snake is ejected because it speaks gibberish.");
            break;
        }
        let head = game.head.move_towards(snake_dir);

        if !game.field.valid(head) {
            println!("crashed in wall.");
            break;
        }
        if game.field.get(head) != Direction::End {
            if !game.field.available(head) {
                println!("ate snake");
                break;
            }
            game.field.set(head, snake_dir.invert());
            game.head = head; /* we *might* have overwritten tail */

            //are we on a apple now?
            let ate_apple = game.head == game.apple;
            if ate_apple {
                game.apples += 1;
                if !game.new_apple() {
                    println!("won, I think.");
                    break;
                }
            } else { //move tail
                let _dropped = game.field.drop_last(game.head);
            }
        } else {
            /* This is a corner case where we follow our tail closely. We
             * must be careful not to overwrite tail. On the flip side we
             * don't have to check for apples or collisions. */
            let _dropped = game.field.drop_last(game.head);
            game.field.set(head, snake_dir.invert());
            game.head = head; /* we *might* have overwritten tail */
        }

        thread::sleep(time::Duration::from_millis(50));
        game.moves += 1;
        print!("{}[2J", 27 as char);
        game.draw();
    }
    game.draw();
}




    //fn next(&self, dir: u32) -> Position {
        //let (xn, yn) = match dir {
            //LEFT  => (self.head.x-1, self.head.y),
            //RIGHT => (self.head.x+1, self.head.y),
            //UP    => (self.head.x,   self.head.y-1),
            //DOWN  => (self.head.x,   self.head.y+1),
            //_  => panic!("Not a valid direction {}", dir),
        //};
        //Position{x:xn, y:yn}
    //}

    //fn previous(&self, pos: Position) -> Position {
        //let x = pos.x;
        //let y = pos.y;
        //let dir = !self.board[y as usize][x as usize] & B_DIR;
        //match dir {
            //LEFT  => Position{x: x-1, y: y},
            //RIGHT => Position{x: x+1, y: y},
            //UP    => Position{x: x,   y: y-1},
            //DOWN  => Position{x: x,   y: y+1},
            //_  => panic!("Not a valid direction {}", dir),
        //}
    //}

    //fn place_random_apple(&mut self) {
        ////TODO will get harder of snake longer

        //let (x,y) = loop {
            //let x = rand::random::<usize>()%self.width;
            //let y = rand::random::<usize>()%self.height;
            //if self.board[y][x] & B_COUNT == 0 {
                //break (x, y)
            //}
        //};
        //self.apple.x = x as isize;
        //self.apple.y = y as isize;
        //self.board[y][x] = B_APPLE;
    //}

    //fn in_bounds(&self, pos: Position) -> bool {
        //pos.x < self.width as isize && pos.y < self.height as isize && pos.x >= 0 && pos.y >= 0
    //}
//}

//fn has(cell:u32, flag:u32) -> bool {
    //cell & flag == flag
//}

//#[allow(dead_code)]
//fn snake_ai_straight(state: &GameState) -> u32 {
    //let x = state.head.x;
    //let y = state.head.y;
    //state.board[y as usize][x as usize] & B_DIR
//}
//#[allow(dead_code)]
//fn snake_ai_random(state: &GameState) -> u32 {
    //rand::random::<u32>()%state.width as u32 & B_DIR
//}
//#[allow(dead_code)]
//fn snake_ai_greedy(state: &GameState) -> u32 {
    //let dx = state.apple.x as i32 - state.head.x as i32;
    //let dy = state.apple.y as i32 - state.head.y as i32;
    //return if dx.abs() > dy.abs() {
        //if dx > 0 { RIGHT } else { LEFT }
    //} else {
        //if dy > 0 { DOWN } else { UP }
    //};
//}

//#[allow(dead_code)]
//fn snake_ai_greedy_avoid_self(state: &GameState) -> u32 {
    //fn cost(state: &GameState, dir: u32) -> i32 {
        //let pos = state.next(dir);
        //if !state.in_bounds(pos) {
            //return 999;
        //}
        //if (state.board[pos.y as usize][pos.x as usize] & B_COUNT) != 0 { return 999; }
        //let dx = state.apple.x as i32 - pos.x as i32;
        //let dy = state.apple.y as i32 - pos.y as i32;
        //return dx.abs() + dy.abs();
    //}

    //let dirs = vec![LEFT, RIGHT, UP, DOWN];
    //dirs.into_iter().min_by_key(|x| cost(&state, *x)).unwrap()
//}

//trait Odd {
    //fn odd(&self) -> bool;
    //fn even(&self) -> bool;
//}

//impl Odd for isize {
    //fn odd(&self) -> bool {
        //self&1 == 1
    //}
    //fn even(&self) -> bool {
        //self&1 == 0
    //}
//}

//#[allow(dead_code)]
//fn snake_ai_hamiltonian(state: &GameState) -> u32 {
    //let x = state.head.x;
    //let y = state.head.y;
    //let w = state.width as isize;
    //let h = state.height as isize;

    //if y == 0 { //go left
        //if x > 0 { LEFT } else { DOWN }
    //} else if x == w-1 { //last column
        //if x.odd() { //straight up!
            //UP
        //} else { //zig(-zag)
            //if (h - y).odd() {
                //UP
            //} else {
                ////CORNER case if w*h is odd
                //if y == 1 && w.odd() && h.odd() && state.apple.y == 0 { UP } else { LEFT }
            //}
        //}
    //} else if x == w-2 && w.odd() { //last column
        //if (h - y).even() { UP } else { RIGHT }
    //} else if x.odd() {
        //if y > 1 { UP } else { RIGHT }
    //} else {
        //if y < h-1 { DOWN } else { RIGHT }
    //}
//}

//struct Snake {
    //path: Vec<Vec<u32>>,
//}

//impl Snake {
    //fn make_path(&mut self, state: &GameState) {
        //let mut x = 0;
        //let mut y = 0;
        //let w = state.width as isize;
        //let h = state.height as isize;

        //loop {
            //let dir =
                //if y == 0 { //go left
                    //if x > 0 { LEFT } else { DOWN }
                //} else if x == w-1 { //last column
                    //if x.odd() { //straight up!
                        //UP
                    //} else { //zig(-zag)
                        //if (h - y).odd() {
                            //UP
                        //} else {
                            ////CORNER case if w*h is odd
                            ////if y == 1 && w.odd() && h.odd() && state.apple.y == 0 { UP } else { LEFT }
                            //LEFT
                        //}
                    //}
                //} else if x == w-2 && w.odd() { //last column
                    //if (h - y).even() { UP } else { RIGHT }
                //} else if x.odd() {
                    //if y > 1 { UP } else { RIGHT }
                //} else {
                    //if y < h-1 { DOWN } else { RIGHT }
                //};
            //self.path[y as usize][x as usize] = dir;
            //match dir {
                //LEFT  => {x=x-1; y=y-0},
                //RIGHT => {x=x+1; y=y-0},
                //UP    => {x=x-0; y=y-1},
                //DOWN  => {x=x-0; y=y+1},
                //_  => print!("error"),
            //}
            //if (x, y) == (0, 0) {
                //break;
            //}
        //}
    //}
    //fn init(state: &GameState) -> Snake {
        //let mut snake = Snake {
            //path: vec![vec![0u32; state.width]; state.height],
        //};
        //snake.make_path(state);
        //snake
    //}
    //fn next(&mut self, state:&GameState) -> u32 {
        //fn hamiltonian_distance(state:&GameState, dir:u32) -> i32 {
            //let pos = state.next(dir);
            //if !state.in_bounds(pos) {
                //return 2*(state.width+state.height) as i32;
            //}
            //if (state.board[pos.y as usize][pos.x as usize] & B_COUNT)>>3 > 1 { //allow for very tip of the tail
                //return 2*(state.width+state.height) as i32;
            //}
            //let x1 = state.head.x as i32;
            //let y1 = state.head.y as i32;
            //let x2 = state.apple.x as i32;
            //let y2 = state.apple.y as i32;
            //(x1-x2).abs() + (y1-y2).abs()
        //}
        //let dirs = vec![LEFT, RIGHT, UP, DOWN];
        //let desired_dir = dirs.into_iter().min_by_key(|x| hamiltonian_distance(&state, *x)).unwrap();
        ////figure out desired direction
        ////  -> direction not running into wall or snake, with lowest Hamiltonian
        ////  distance,
        ////figure out safe direction
        //let safe_dir = self.path[state.head.y as usize][state.head.x as usize];
        ////same? go!
        //if safe_dir == desired_dir {
            //return desired_dir;
        //}
        ////copy path
        //let mut alt_path = self.path.clone();
        ////isolate skipped path and make shortcut

            //// note desired pos
            //// follow & mark safe path until reach desired pos
            ////      then go to safe pos to close the loop
            //// 
        //let desired_pos = go_to(state.head, desired_dir);
        //let safe_pos = go_to(state.head, safe_dir);
        //let mut p = safe_pos;
        //let mut cutoff = vec![p];
        //while p != desired_pos {

        //}


        //// ____
        ////|  __|
        ////| |
        ////| |



        ////for each segment of skipped path:
        ////  Does it have a pll segment of main path?
        ////  is it free of snake?
        ////  make shortcut
        ////  return desired direction
        ////otherwise
        ////  return safe direction
        //return safe_dir;
    //}
//}

//fn main() {
    //const WIDTH:usize = 9;
    //const HEIGHT:usize = 9;

    //let mut state = GameState::init(WIDTH, HEIGHT);
    //let mut snake = Snake::init(&state);
    //state.draw();

    //loop {
        ////ask AI for move
        ////let dir = snake_ai_straight(&state);
        ////let dir = snake_ai_random(&state);
        ////let dir = snake_ai_greedy(&state);
        ////let dir = snake_ai_greedy_avoid_self(&state);
        ////let dir = snake_ai_hamiltonian(&state);
        //let dir = snake.next(&state);

        //state.moves += 1;

        //let new_pos = state.next(dir);
        //if !state.in_bounds( new_pos) {
            //println!("DEAD. Ran into a wall");
            //break;
        //}
        //if (state.board[new_pos.y as usize][new_pos.x as usize] & B_COUNT)>>3 > 1 { //allow for very tip of the tail
            //println!("DEAD. Ran into a snake");
            //break;
        //}

        ////use of new_pos head is fine.
        //let head = (state.board[state.head.y as usize][state.head.x as usize] & B_COUNT) | dir | (state.board[new_pos.y as usize][new_pos.x as usize] & B_APPLE);
        //state.board[new_pos.y as usize][new_pos.x as usize] = head;

        //let head = state.board[new_pos.y as usize][new_pos.x as usize];
        //let ate_apple = has(head, B_APPLE);
        //if ate_apple { //no need to move snake
            //state.board[new_pos.y as usize][new_pos.x as usize] += 1<<3;
            //state.length += 1;
            //state.board[new_pos.y as usize][new_pos.x as usize] &= !B_APPLE; //clear apple
        //} else { //Move snake forward head to tail
            //let mut pos = state.head;
            //loop {
                //state.board[pos.y as usize][pos.x as usize] -= 1<<3;
                //if (state.board[pos.y as usize][pos.x as usize] & B_COUNT) == 0 {
                    //// This was the last bit of tail. we are done.
                    //state.board[pos.y as usize][pos.x as usize] = 0;
                    //break;
                //}
                //pos = state.previous(pos);
                //// There is a corner case where the snake follows its own tail
                //// at distance zero. We must detect the head to avoid getting
                //// in to a loop.
                //if pos.x == state.head.x && pos.y == state.head.y {
                    //break;
                //}
            //}
        //}
        //state.head = new_pos;

        //if ate_apple {// generate new apple.
            //if state.length as usize != WIDTH*HEIGHT-1 {
                //state.place_random_apple();
            //} else {
                //state.draw();
                //println!("VICTORY. Ate last apple");
                //break;
            //}
        //}

        //thread::sleep(time::Duration::from_millis(60));
        //print!("{}[2J", 27 as char);
        //state.draw();
    //}
//}
