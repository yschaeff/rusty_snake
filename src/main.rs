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
    fn is_valid_direction(&self) -> bool {
        *self != Direction::End && *self != Direction::Null
    }
    fn random(rng:&mut StdRng) -> Direction {
        match rng.gen_range(0..4) {
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
        let x = rng.gen_range(0..self.x);
        let y = rng.gen_range(0..self.y);
        Coordinate{x, y}
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
    fn get_direction_at(&self, position:Coordinate) -> Direction {
        self.directions[position.y as usize][position.x as usize]
    }
    fn set_direction_at(&mut self, position:Coordinate, direction:Direction) {
        self.directions[position.y as usize][position.x as usize] = direction;
    }
    fn next(&self, position:Coordinate) -> Coordinate {
        let direction = self.get_direction_at(position);
        position.move_towards(direction)
    }
    fn coordinate_in_bounds(&self, position:Coordinate) -> bool {
        position.x >= 0 && position.y >= 0 && position.x < self.dimension.x && position.y < self.dimension.y
    }
    fn free_at(&self, position:Coordinate) -> bool {
        self.directions[position.y as usize][position.x as usize] == Direction::Null
    }
    fn random_available(&self, rng:&mut StdRng) -> Option<Coordinate> {
        let w = self.dimension.x;
        let h = self.dimension.y;
        let r = self.dimension.random(rng);

        for y in 0..h {
            for x in 0..w {
                let p = Coordinate{x: (x+r.x)%w, y: (y+r.y)%h};
                if self.free_at(p) {
                    return Some(p);
                }
            }
        }
        None
    }
    /* Follow chain backwards. Drop last segment, return its coordinates */
    fn drop_last_in_chain(&mut self, start:Coordinate) -> Coordinate {
        let (b, a) = self.find_last(start);
        self.set_direction_at(a, Direction::End);
        self.set_direction_at(b, Direction::Null);
        b
    }
    fn find_last(&self, start:Coordinate) -> (Coordinate, Coordinate) {
        let mut a = start;
        let mut b = self.next(a);
        while self.get_direction_at(b) != Direction::End {
            a = b;
            b = self.next(a);
        }
        (b, a)
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
        let mut rng = StdRng::seed_from_u64(42);
        let field_dimension = Coordinate{x:width as isize, y:height as isize};
        let mut field = Field::init(field_dimension);
        let head = field_dimension.random(&mut rng);
        let direction = Direction::End;
        println!("setting head {:?}", head);
        field.set_direction_at(head, direction);
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
    fn place_new_apple(&mut self) -> bool {
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
    fn choose_direction(&self, game:&Game) -> Option<Direction>;
}

struct SillySnake;
impl Snake for SillySnake {
    fn init(&mut self, _game:&Game) { }
    fn choose_direction(&self, _game:&Game) -> Option<Direction> {
        let mut rng = StdRng::from_entropy();
        Some(Direction::random(&mut rng))
    }
}

struct GreedySnake;
impl Snake for GreedySnake {
    fn init(&mut self, _game:&Game) { }
    fn choose_direction(&self, game:&Game) -> Option<Direction> {
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
        game.field.coordinate_in_bounds(pos) && game.field.free_at(pos)
    }
}
impl Snake for GreedyPickySnake {
    fn init(&mut self, _game:&Game) { }
    fn choose_direction(&self, game:&Game) -> Option<Direction> {
        let preferred = GreedyPickySnake::prioritize(game.head, game.apple).into_iter();
        let available = preferred.filter(|dir| GreedyPickySnake::available(game, *dir));
        for dir in available { //return first if list not empty
            return Some(dir);
        }
        None //Give up
    }
}

/* A winning strategy. However at a cost. Expected moves per apple
 * works out to (w*h)/4 */
struct HamiltonianSnake;
impl Snake for HamiltonianSnake {
    fn init(&mut self, _game:&Game) { }
    fn choose_direction(&self, game:&Game) -> Option<Direction> {
        Some(HamiltonianSnake::next_hamiltonian_direction(game, game.head, game.apple))
    }
}
impl HamiltonianSnake {
    fn next_hamiltonian_direction(game:&Game, head:Coordinate, target:Coordinate) -> Direction {
        let x = head.x;
        let y = head.y;
        let w = game.field.dimension.x;
        let h = game.field.dimension.y;

        if y == 0 {
            /* At the top row we go only left, then down */
            if x > 0 { Direction::Left } else { Direction::Down }
        } else if x == w-1 { //last column
            /* In the last column go straight up then left OR
             * wiggle upwards if column count (w) is odd. */
            if !odd(w) { //straight up!
                Direction::Up
            } else {
                if odd(h - y) {
                    Direction::Up
                } else {
                    /*CORNER case if w*h is odd reroute path trough top right corner */
                    if y == 1 && odd(w) && odd(h) && target.y == 0 { Direction::Up } else { Direction::Left }
                }
            }
        } else if x == w-2 && odd(w) { //1 before last column
            /* Wiggle back to last column  */
            if !odd(h - y) { Direction::Up } else { Direction::Right }
        } else if odd(x) {
            /* Down on odd lines */
            if y > 1 { Direction::Up } else { Direction::Right }
        } else {
            /* Up on even lines */
            if y < h-1 { Direction::Down } else { Direction::Right }
        }
    }
}

struct ImpatientHamiltonianSnake;
impl Snake for ImpatientHamiltonianSnake {
    fn init(&mut self, _game:&Game) { }
    /* propose greedy move, if after making that move can't follow
     * a Hamiltonian path to the apple reject. */
    fn choose_direction(&self, game:&Game) -> Option<Direction> {
        let preferred = GreedyPickySnake::prioritize(game.head, game.apple).into_iter();
        let available = preferred.filter(|dir| GreedyPickySnake::available(game, *dir));
        for dir in available { //return first if list not empty
            let pos = game.head.move_towards(dir);
            if ImpatientHamiltonianSnake::apple_on_path_to_tail(game, pos) {
                return Some(dir);
            }
            break;
        }
        Some(HamiltonianSnake::next_hamiltonian_direction(game, game.head, game.apple))
    }
}
impl ImpatientHamiltonianSnake {
    fn next_hamiltonian_move(game:&Game, head:Coordinate, target:Coordinate) -> Coordinate {
        let dir = HamiltonianSnake::next_hamiltonian_direction(game, head, target);
        head.move_towards(dir)
    }
    fn apple_on_path_to_tail(game:&Game, head:Coordinate) -> bool {
        let (tail, _) = game.field.find_last(game.head);
        let mut pos = head;
        let mut seen_apple = false;
        while pos != tail {
            if !game.field.free_at(pos) {
                return false;
            }
            if pos == game.apple {
                seen_apple = true;
            }
            pos = ImpatientHamiltonianSnake::next_hamiltonian_move(game, pos, tail);
        }
        seen_apple
    }
}

// NEXT calculate shortest path and validate with ham snake

fn choose_snake(k:u32) -> Box<dyn Snake> {
    match k {
        0 => Box::new(SillySnake{}),
        1 => Box::new(GreedySnake{}),
        2 => Box::new(GreedyPickySnake{}),
        3 => Box::new(HamiltonianSnake{}),
        4 => Box::new(ImpatientHamiltonianSnake{}),
        _ => panic!("Never heard of such snake"),
    }
}

fn main() {
    const WIDTH:usize = 5;
    const HEIGHT:usize = 5;

    let mut game = Game::init(WIDTH, HEIGHT);
    let mut snake = choose_snake(4); //Dynamic so we can get it as user input
    snake.init(&game);

    game.draw();
    loop {
        let snake_dir = match snake.choose_direction(&game) {
            Some(dir) => dir,
            None => {
                println!("Snake forfeit.");
                break; }};
        if !snake_dir.is_valid_direction() {
            println!("Snake is ejected because it speaks gibberish.");
            break;
        }
        let head = game.head.move_towards(snake_dir);

        if !game.field.coordinate_in_bounds(head) {
            println!("crashed in wall.");
            break;
        }
        if game.field.get_direction_at(head) != Direction::End {
            if !game.field.free_at(head) {
                println!("ate snake");
                break;
            }
            game.field.set_direction_at(head, snake_dir.invert());
            game.head = head;

            //are we on a apple now?
            let ate_apple = game.head == game.apple;
            if ate_apple {
                game.apples += 1;
                if !game.place_new_apple() {
                    println!("The Snake has won the game.");
                    break;
                }
            } else { //move tail
                let _dropped = game.field.drop_last_in_chain(game.head);
            }
        } else {
            /* This is a corner case where we follow our tail closely. We
             * must be careful not to overwrite tail. On the flip side we
             * don't have to check for apples or collisions. */
            let _dropped = game.field.drop_last_in_chain(game.head);
            game.field.set_direction_at(head, snake_dir.invert());
            game.head = head; /* we *might* have overwritten tail */
        }

        thread::sleep(time::Duration::from_millis(50));
        game.moves += 1;
        print!("{}[2J", 27 as char); //Clear screen
        game.draw();
    }
    game.draw();
}
