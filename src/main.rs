use std::io::prelude::*;

fn read_all<S: AsRef<std::path::Path>>(path: S) -> String {
    let mut content = String::new();
    let mut f = std::fs::File::open(path).unwrap();
    f.read_to_string(&mut content).unwrap();
    content
}

fn main() {
    let fname = std::env::args().nth(1).unwrap_or(String::from("example"));
    let content = read_all(fname);

    let maze = Maze::from_str(&content);

    let runner = Runner::new(maze,
                             door(&content),
                             Direction::Down);

    println!("Path = {}", runner.path());

    println!("Steps = {}", Runner::new(Maze::from_str(&content),
                                       door(&content),
                                       Direction::Down).count() + 1)
}

#[derive(Copy, Clone)]
enum Direction {
    Down,
    Left,
    Right,
    Up,
}

type Coord = i32;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Position(Coord, Coord);

impl Position {
    fn mv(&self, direction: Direction) -> Self {
        use Direction::*;
        let &Position(mut r, mut c) = self;
        match direction {
            Down => r += 1,
            Left => c -= 1,
            Right => c += 1,
            Up => r -= 1,
        };
        Position(r, c)
    }
}

type CellInner = char;

#[derive(Debug, Eq, PartialEq)]
struct Cell(Position, Option<CellInner>);

impl From<(Coord, Coord, Option<CellInner>)> for Cell {
    fn from(data: (Coord, Coord, Option<CellInner>)) -> Self {
        let (r, c, i) = data;
        Cell(Position(r, c), i)
    }
}

#[derive(Debug)]
struct Maze(std::collections::HashMap<Position, Option<CellInner>>);

impl Maze {
    fn from_iter<C: Into<Cell>, I: Iterator<Item=C>>(it: I) -> Self {
        let cell_tuple = |c: Cell| {
            let Cell(p, inner) = c;
            (p, inner)
        };

        Maze(it.map(|c| c.into()).map(cell_tuple).collect())

    }

    fn from_path<C: Into<Cell>>(v: Vec<C>) -> Self {
        Self::from_iter(v.into_iter())
    }

    fn from_str(s: &str) -> Self {
        Self::from_iter(
            s.lines().enumerate().flat_map(
            |(r, l)|
                l.chars()
                    .enumerate()
                    .filter(|&(_, brick)| brick != ' ')
                    .map(move|(c, brick)| (r as i32, c as i32, if brick.is_alphabetic() {Some(brick)} else {None}))
        )
        )
    }

    fn cell(&self, position: &Position) -> Option<Cell> {
        self.0.get(position).map(|inner|
            Cell(position.clone(), inner.clone())
        )
    }
}

struct Runner {
    maze: Maze,
    position: Position,
    direction: Direction,
}

impl Runner {
    fn new(maze: Maze, position: Position, direction: Direction) -> Self {
        Self {
            maze,
            position,
            direction,
        }
    }

    fn path(self) -> String {
        self.filter_map(
            |Cell(_, inner)| inner)
            .fold(String::new(), |mut s, c|
                {
                    s.push(c);
                    s
                })
    }

    fn directions(&self) -> &[Direction] {
        use Direction::*;
        match self.direction {
            Down => &[Down, Left, Right],
            Left => &[Left, Down, Up],
            Right => &[Right, Up, Down],
            Up => &[Up, Right, Left],
        }
    }
}

impl Iterator for Runner {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        self.directions().iter().filter_map(|&d|
            self.maze.cell(&self.position.mv(d))
                .map(|c| (c, d))
        ).nth(0).map (
            |(c, d)|
                {
                    self.direction = d;
                    self.position = c.0.clone();
                    c
                }
        )
    }
}

fn door(maze: &str) -> Position {
    Position(0, maze
        .lines()
        .nth(0)
        .unwrap().char_indices()
        .filter_map(|(c, brick)|
            if brick != ' ' {Some (c as i32)} else {None})
        .nth(0).unwrap())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn move_forward() {
        let maze = Maze::from_path(vec![(3, 5, None), (4, 5, None)]);

        let mut runner = Runner::new(maze,
                                     Position(3, 5),
                                     Direction::Down);

        assert_eq!(Cell(Position(4, 5), None), runner.next().unwrap());
        assert_eq!(None, runner.next());
    }

    #[test]
    fn collect() {
        let maze = Maze::from_path(
            vec![(1, 2, None), (1, 3, Some('A')), (1, 4, Some('B')),
                 (1, 5, None), (1, 6, Some('C'))]);

        let runner = Runner::new(maze,
                                 Position(1, 2),
                                 Direction::Right);

        assert_eq!("ABC", runner.path());
    }

    #[test]
    fn should_turn_left() {
        let maze = Maze::from_path(
            vec![(1, 3, None), (1, 2, None), (2, 2, None)]);

        let runner = Runner::new(maze,
                                 Position(1, 3),
                                 Direction::Left);

        assert_eq!(Cell(Position(2, 2), None),  runner.last().unwrap());
    }

    static MAZE : &'static str = "     |
     |  +--+
     A  |  C
 F---|----E|--+
     |  |  |  D
     +B-+  +--+
                ";
    #[test]
    fn integration() {
        let maze = Maze::from_str(MAZE);

        let runner = Runner::new(maze,
                                 door(MAZE),
                                 Direction::Down);

        assert_eq!("ABCDEF", runner.path());
    }

    #[test]
    fn count_steps() {
        let maze = Maze::from_str(MAZE);

        let runner = Runner::new(maze,
                                 door(MAZE),
                                 Direction::Down);

        assert_eq!(38, runner.count() + 1);

    }
}
