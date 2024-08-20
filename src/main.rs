use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const WIDTH: usize = 21;
const HEIGHT: usize = 11;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Wall,
    Path,
    Start,
    Goal,
    Player,
}

struct Player {
    x: usize,
    y: usize,
}


fn main() {
    let mut maze = vec![vec![Cell::Wall; WIDTH]; HEIGHT];
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    generate_maze(&mut maze, 1, 1, seed);
    set_start_and_goal(&mut maze, seed);

    // for i in 0..WIDTH {
    //     maze[0][i] = Cell::Wall;
    //
    // }
    // for i in 0..HEIGHT {
    //     maze[i][0] = Cell::Wall;
    // }

    // maze[1][0] = Cell::Start;

    let mut player = Player {x: 1, y: 1};
    maze[player.y][player.x] = Cell::Player;

    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = termion::async_stdin();
    let mut keys = stdin.keys();

    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(100);

    for row in &mut maze {
        row.push(Cell::Path);
    }
    maze.push(vec![Cell::Path; WIDTH]);

    print!("{}", termion::clear::All);
    print_maze(&mut stdout, &maze);
    write!(stdout, "{}移動: w(上), s(下), a(左), d(右), q(終了)", Goto(1, HEIGHT as u16 + 2)).unwrap();
    stdout.flush().unwrap();

    loop {
        if last_update.elapsed() >= update_interval {
            if maze[player.y][player.x] == Cell::Goal {
                write!(stdout, "{}ゴールに到達しました！おめでとうございます！", Goto(1, HEIGHT as u16 + 3)).unwrap();
                break;
            }
            last_update = Instant::now();
        }

        if let Some(Ok(key)) = keys.next() {
            match key {
                Key::Char('w') => move_player(&mut maze, &mut player, 0, -1),
                Key::Char('s') => move_player(&mut maze, &mut player, 0, 1),
                Key::Char('a') => move_player(&mut maze, &mut player, -1, 0),
                Key::Char('d') => move_player(&mut maze, &mut player, 1, 0),
                Key::Char('q') => return,
                _ => continue,
            }
            print!("{}", termion::clear::All);
            print_maze(&mut stdout, &maze);
            write!(stdout, "{}移動: w(上), s(下), a(左), d(右), q(終了)", Goto(1, HEIGHT as u16 + 2)).unwrap();
            stdout.flush().unwrap();
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn generate_maze(maze: &mut Vec<Vec<Cell>>, x: usize, y: usize, seed: u64) {
    let mut rng = seed;
    let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    let mut shuffled = directions.clone();

    for i in (1..4).rev() {
        rng = (rng * 1103515245 + 12345) % 2_u64.pow(31);
        let j = (rng % (i + 1) as u64) as usize;
        shuffled.swap(i, j);
    }

    maze[y][x] = Cell::Path;

    for (dx, dy) in shuffled.iter() {
        let nx = x as i32 + dx * 2;
        let ny = y as i32 + dy * 2;
        let new_x = x as i32 + dx;
        let new_y = y as i32 + dy;
        if nx > 0 && nx < WIDTH as i32 && ny > 0 && ny < HEIGHT as i32 && maze[ny as usize][nx as usize] == Cell::Wall {
            maze[new_y as usize][new_x as usize] = Cell::Path;
            generate_maze(maze, nx as usize, ny as usize, rng);
        }
    }
}

fn print_maze<W: Write>(stdout: &mut W, maze: &Vec<Vec<Cell>>) {
    for (y, row) in maze.iter().enumerate() {
        write!(stdout, "{}", Goto(1, y as u16 +  1)).unwrap();
        for &cell in row {
             match cell {
                 Cell::Wall => print!("■"),
                 Cell::Path => print!(" "),
                 Cell::Start => print!("S"),
                 Cell::Goal => print!("G"),
                 Cell::Player => print!("●"),
             }
        }
    }
}

fn move_player(maze: &mut Vec<Vec<Cell>>, player: &mut Player, dx: i32, dy: i32) {
    let new_x = player.x as i32 + dx;
    let new_y = player.y as i32 + dy;

    let current_x = player.x;
    let current_y = player.y;

    if new_x >= 0 && new_x < WIDTH as i32 && new_y >= 0 && new_y < HEIGHT as i32 {
        let new_cell = maze[new_y as usize][new_x as usize];
        if new_cell != Cell::Wall {
            if ![Cell::Start, Cell::Goal].contains(&maze[player.y][player.x]) {
                maze[player.y][player.x] = Cell::Path;
            }
            player.y = new_y as usize;
            player.x = new_x as usize;
            if maze[player.y][player.x] != Cell::Goal {
                maze[player.y][player.x] = Cell::Player;
            } else if current_y == HEIGHT - 2 && maze[current_y + 1][player.x] == Cell::Goal {
                maze[player.y + 1][player.x] = Cell::Player;
            } else if current_x == WIDTH - 2 && maze[player.y][current_x + 1] == Cell::Goal {
                maze[player.y][player.x + 1] = Cell::Player;
            }
        }
    }
}

fn set_start_and_goal(maze: &mut Vec<Vec<Cell>>, seed: u64) {
    let mut rng = seed;
    let mut max_distance = 0;
    let start_pos = (0, 1);
    let mut goal_pos = (1, 1);

    for y in 1..HEIGHT - 1 {
        for x in 1..WIDTH - 1 {
            if maze[y][x] == Cell::Path {
                let distance = (x - 1) + (y - 1);
                if distance > max_distance {
                    max_distance = distance;
                    goal_pos = (x, y);
                } else if distance == max_distance {
                    rng = (rng * 1103515245 + 12345) % 2147483648;
                    if rng % 2 == 0 {
                        goal_pos = (x, y);
                    }
                }
            }
        }
    }

    if goal_pos.0 == WIDTH - 2 {
        goal_pos = (goal_pos.0 + 1, goal_pos.1);
    } else if goal_pos.1 == HEIGHT - 2 {
        goal_pos = (goal_pos.0, goal_pos.1 + 1);
    }

    maze[start_pos.1][start_pos.0] = Cell::Start;
    maze[goal_pos.1][goal_pos.0] = Cell::Goal;
}
