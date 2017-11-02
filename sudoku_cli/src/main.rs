extern crate sudoku;
extern crate regex;
extern crate term_painter;


#[cfg(test)]
mod test;

use sudoku::board::*;
use sudoku::solver::*;
use sudoku::hintmap::HintMap;

use std::{env, io, time};
use std::io::Read;
use std::io::prelude::*;
use std::fs::File;

use regex::Regex;

use term_painter::ToStyle;
use term_painter::Color::*;

#[derive(Debug, PartialEq)]
enum Command {
    Quit,
    Help,
    Unrecognised,
    Set {
        x: Pos,
        y: Pos,
        val: Option<SquareType>,
    },
    Clear { x: Pos, y: Pos },
    Hint { x: Pos, y: Pos },
    HintAll,
    FromBase(usize),
    Sample,
    Reset,
    Check,
    Solve(SolveType),
    RandRow(usize),
    RandCol(usize),
    RandGrid(usize),
    RandDash,
    Generate,
    Save { file_name: String },
    Load { file_name: String },
    ShowStr,
    Other(String),
}

#[derive(Debug, PartialEq)]
enum SolveType {
    Standard,
    Ordered,
    Parallel,
    Search,
    SearchParallel,
    DFS(Option<i32>),
}


#[derive(Debug, PartialEq)]
enum Action {
    Quit,
    Continue,
    ContinueWithoutPrinting,
}

pub fn main() {
    let mut game_board = Board::default();

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        for i in 1..args.len() {
            let input = &args[i];
            println!("Loading from arguments: {}", input);
            let cmd: Command = get_command(input);
            match process_command(cmd, &mut game_board) {
                Action::Continue => {
                    game_board.print();
                }
                Action::ContinueWithoutPrinting => {
                    println!("");
                }
                Action::Quit => {
                    println!("QUIT");
                    return;
                }
            }
        }
    } else {
        println!("--------------");
        println!("--- {} ---", Magenta.paint("Sudoku"));
        println!("--------------");
        println!("");
        process_command(Command::Help, &mut game_board); //TODO: remove
        println!("");
        game_board.print();
    }


    'game_loop: loop {
        println!("Enter a command (or {})", Green.paint("help"));
        let mut input_string = String::new();
        io::stdin().read_line(&mut input_string).expect(
            "failed to read line",
        );
        println!("");

        for input in (&input_string).split("&&") {
            let cmd: Command = get_command(input);
            match process_command(cmd, &mut game_board) {
                Action::Continue => {
                    game_board.print();
                }
                Action::ContinueWithoutPrinting => {}
                Action::Quit => {
                    println!("Quitting...");
                    break 'game_loop;
                }
            }
            println!("");
        }
    }
}

fn get_command(input: &str) -> Command {
    //Capture all words, numbers, etc.
    let captures: Vec<String> = Regex::new(r"([\w\.\\/]+)")
        .unwrap()
        .captures_iter(&input)
        .map(|caps| String::from(&caps[0]).to_lowercase())
        .collect();

    let num_args = captures.len();
    if num_args > 0 {
        match captures[0].as_ref() {
            "quit" | "exit" => Command::Quit,
            "reset" => Command::Reset,
            "solve" if num_args == 1 => Command::Solve(SolveType::Standard),
            "solve" if num_args > 1 => {
                if let Some(solve_type) = match captures[1].as_ref() {
                    "standad" | "normal" => Some(SolveType::Standard),
                    "ordered" => Some(SolveType::Ordered),
                    "parallel" => Some(SolveType::Parallel),
                    "search" if num_args > 2 => {
                        match captures[2].as_ref() {
                            "standard" => Some(SolveType::Search),
                            "parallel" => Some(SolveType::SearchParallel),
                            _ => None,
                        }
                    }
                    "search" => Some(SolveType::Search),
                    "dfs" => {
                        let max_depth = if num_args > 2 {
                            if let Ok(max_depth) = captures[2].parse::<i32>() {
                                Some(max_depth)
                            } else {
                                println!("Unable to parse depth.");
                                None
                            }
                        } else {
                            None
                        };
                        Some(SolveType::DFS(max_depth))
                    }
                    _ => None,
                }
                {
                    Command::Solve(solve_type)
                } else {
                    Command::Unrecognised
                }
            }
            //"rand" if num_args == 1 => {
            //}
            "rand" if num_args == 2 && captures[1] == String::from("dash") => Command::RandDash,
            "rand" if num_args == 3 => {
                //TODO: proper error checking
                let num = captures[2].parse::<usize>().unwrap();
                match captures[1].as_ref() {
                    "row" => Command::RandRow(num),
                    "col" => Command::RandCol(num),
                    "grid" => Command::RandGrid(num),
                    _ => Command::Unrecognised,
                }
            }
            "generate" | "new" => Command::Generate,
            "sample" => Command::Sample,
            "help" => Command::Help,
            "check" => Command::Check,
            "set" if num_args == 4 => {
                let x = captures[1].parse::<Pos>();
                let y = captures[2].parse::<Pos>();
                let val = captures[3].parse::<SquareType>();
                match (x, y, val) {
                    (Ok(x), Ok(y), Ok(val)) => Command::Set {
                        x,
                        y,
                        val: Some(val),
                    },
                    _ => Command::Unrecognised,
                }
            }
            "clear" if num_args == 2 && captures[1] == String::from("all") => Command::Reset,
            "clear" if num_args == 3 => {
                let x = captures[1].parse::<Pos>();
                let y = captures[2].parse::<Pos>();
                match (x, y) {
                    (Ok(x), Ok(y)) => Command::Clear { x, y },
                    _ => Command::Unrecognised,
                }
            }
            "hints" => Command::HintAll,
            "hint" if num_args == 2 && captures[1] == String::from("all") => Command::HintAll,
            "hint" if num_args == 3 => {
                let x = captures[1].parse::<Pos>();
                let y = captures[2].parse::<Pos>();
                match (x, y) {
                    (Ok(x), Ok(y)) => Command::Hint { x, y },
                    _ => Command::Unrecognised,
                }
            }
            "save" if num_args == 2 => {
                let file_name: String = String::from(captures[1].as_ref());
                Command::Save { file_name }
            }
            "load" if num_args == 2 => {
                let file_name: String = String::from(captures[1].as_ref());
                Command::Load { file_name }
            }
            "size" if num_args == 2 => {
                let size_name = &captures[1];
                let base_num = match size_name.as_ref() {
                    "small" => 2,
                    "normal" => 3,
                    "large" => 4,
                    "xlarge" => 5,
                    _ => 3,
                };
                Command::FromBase(base_num)
            }
            "base" if num_args == 2 => {
                if let Ok(base_num) = (&captures[1]).parse::<usize>() {
                    Command::FromBase(base_num)
                } else {
                    Command::Unrecognised
                }
            }
            "string" | "str" => Command::ShowStr,
            _ => Command::Other(input.to_string()),
            //_ => Command::Unrecognised,
        }
    } else {
        Command::Unrecognised
    }
}

fn process_command(command: Command, game_board: &mut Board) -> Action {
    match command {
        Command::Quit => Action::Quit,
        Command::Help => {
            println!("---------");
            println!("{}", Magenta.paint("Help menu"));
            println!("---------");

            println!("This Sudoku CLI Game was created by Simon Gordon for CAB401.");
            println!("");

            let commands = [
                ("set [COL] [ROW] [VAL]", "Set a value."),
                ("clear [COL] [ROW]", "Clear a value."),
                ("check", "Check the board is valid/solved."),
                ("hint [COL] [ROW]", "Get hints for a square."),
                ("hint all", "Display all hints for the board."),
                (
                    "base [NUM]",
                    "Set the base number for the puzzle. Default is 3 (for 9*9 board).",
                ),
                ("reset", "Reset the board to default dimensions."),
                ("sample", "Load the sample puzzle."),
                ("solve", "Solve the puzzle (simple mode)."),
                (
                    "solve parallel",
                    "Solve the puzzle (multithreaded simple mode).",
                ),
                ("solve search", "Recursively solve the puzzle."),
                (
                    "solve search parallel",
                    "Recursively solve the puzzle (multithreaded).",
                ),
                (
                    "compare",
                    concat!(
                        "Compare the execution time of sequential and parallel solve algorithms for the given board.",
                        " Does not affect active puzzle."
                    ),
                ),
                ("generate", "Generate a new puzzle."),
                // ("")
                ("save [FILE_PATH]", "Save a puzzle."),
                ("load [FILE_PATH]", "Load a puzzle."),
                (
                    "string",
                    "Display the string representation of this puzzle.",
                ),
                ("help", "Display help menu."),
                ("exit", "Quit the game."),
            ];

            for &(cmd, description) in commands.iter() {
                println!("> {:23}-> {}", cmd, description);
            }

            println!("");
            println!("Other notes:");
            println!(
                "-You can also chain together multiple commands by using double ampersands, like so:"
            );
            println!("");
            println!("    set 1 2 3 && clear 4 3 && solve && quit");
            println!("");
            println!(
                "-Another option is to pass a range of initial commands as arguments to the program:"
            );
            println!("");
            println!("    sudoku.exe \"base 4\" compare quit>>logs/somelog.txt");
            println!("");
            Action::ContinueWithoutPrinting
        }
        Command::Set { x, y, val } => {
            let result = game_board.set_val((x, y), val);
            if !result.is_ok() {
                println!("Error trying to set value. Index or value may be out of range.");
            }
            Action::Continue
        }
        Command::Clear { x, y } => {
            let result = game_board.set_val((x, y), None);
            if !result.is_ok() {
                println!("Error trying to clear value. Index may be out of range.");
            }
            Action::Continue
        }
        Command::Hint { x, y } => {
            let hmap = HintMap::from_board(&game_board);
            println!("{:?}", hmap.get_hints((x, y)));
            Action::Continue
        }
        Command::HintAll => {
            let hmap = HintMap::from_board(&game_board);
            let width = game_board.side_length;
            println!("(x, y) [hints]");
            for index in 0..game_board.num_squares {
                if let Ok(val) = game_board.get_val(index) {
                    if val.is_none() {
                        let hints = hmap.get_hints(index);
                        let coords = index.into_coord(width);
                        println!("({}, {}) {:?}", coords.0, coords.1, hints);
                    }
                }
            }
            Action::Continue
        }
        Command::Sample => {
            *game_board = Board::from_string(UNSOLVED_PUZZLE_STR);
            Action::Continue
        }
        Command::Reset => {
            *game_board = Board::default();
            Action::Continue
        }
        Command::FromBase(base_num) => {
            *game_board = Board::from_base_num(base_num);
            Action::Continue
        }
        Command::RandRow(row_num) => {
            game_board.set_row_rand(row_num);
            Action::Continue
        }
        Command::RandCol(col_num) => {
            game_board.set_col_rand(col_num);
            Action::Continue
        }
        Command::RandGrid(grid_num) => {
            game_board.set_grid_rand(grid_num);
            Action::Continue
        }
        Command::RandDash => {
            let base_num = game_board.base_num;
            *game_board = Board::from_base_num(base_num);
            for i in 0..game_board.side_length {
                let grid_num = game_board.get_grid_num((i, i));
                game_board.set_grid_rand(grid_num);
            }
            Action::Continue
        }
        Command::Check => {
            let msg = match game_board.check_status() {
                BoardStatus::Valid => "Board is valid",
                BoardStatus::Invalid => "Board is invalid",
                BoardStatus::Solved => "Board is solved",
            };
            println!("{}", msg);
            Action::Continue
        }
        Command::Solve(solve_type) => {
            println!("Solving...");
            let now = time::SystemTime::now();
            let solved = match solve_type {
                SolveType::Standard => game_board.solve_standard().is_solved(),
                SolveType::Ordered => game_board.solve_ordered().is_solved(),
                SolveType::Parallel => game_board.solve_parallel().is_solved(),
                SolveType::Search => game_board.solve_search(),
                SolveType::SearchParallel => game_board.solve_search_parallel(),
                SolveType::DFS(max_depth) => game_board.solve_search_dfs(max_depth).is_solved(),
            };
            if solved {
                println!("Successfully solved!");
            } else {
                println!("Unable to solve.");
            }
            let elapsed = now.elapsed().expect("Error retrieving time");
            println!("Took {} seconds.", elapsed.as_secs());
            Action::Continue
        }
        Command::Generate => {
            println!("Generating...");
            let now = time::SystemTime::now();
            if let Ok(new_board) = Board::generate_new(game_board.base_num) {
                *game_board = new_board;
                println!("Generated successfully!");
            } else {
                println!("Generating failed.");
            }
            let elapsed = now.elapsed().expect("Error retrieving time");
            println!("Took {} seconds", elapsed.as_secs());
            Action::Continue
        }
        Command::Save { file_name } => {
            println!("Saving to {}", file_name);
            let mut output_file = match File::open(file_name.clone()) {
                Ok(file) => file,
                Err(_) => File::create(file_name.clone()).unwrap(),
            };
            let puzzle_str = game_board.to_string();
            match output_file.write(puzzle_str.as_bytes()) {
                Ok(_) => println!("Saved successfully!"),
                Err(_) => println!("Failed to write."),
            }
            Action::Continue
        }
        Command::Load { file_name } => {
            println!("Loading from {}", file_name);
            if let Ok(mut input_file) = File::open(file_name) {
                let mut buffer = String::new();
                match input_file.read_to_string(&mut buffer) {
                    Ok(_) => println!("Loaded successfully!"),
                    Err(_) => println!("Failed to read."),
                }
                *game_board = Board::from_string(&buffer);
            } else {
                println!("Unable to open");
            }
            Action::Continue
        }
        Command::ShowStr => {
            println!("{}", game_board.to_string());
            Action::Continue
        }

        //Lets me be sneaky and add extra commands on the fly
        Command::Other(input) => {
            match input.trim().as_ref() {
                "reduce" => {
                    game_board.reduce();
                    Action::Continue
                }
                "mirror" => {
                    game_board._mirror();
                    Action::Continue
                }
                "rotate" => {
                    game_board._rotate();
                    Action::Continue
                }
                "compare" => {
                    let mut board_clone = game_board.clone();
                    let now = time::SystemTime::now();
                    let solved = board_clone.solve_search_parallel();
                    let parallel_elapsed = now.elapsed().expect("Error retrieving time");
                    let parallel_status = if solved { "success" } else { "failed" };
                    println!(
                        "parallel done ({}, {}) {}",
                        parallel_status,
                        parallel_elapsed.as_secs(),
                        board_clone.to_string()
                    );
                    board_clone.print();

                    let mut board_clone = game_board.clone();
                    let now = time::SystemTime::now();
                    let solved = board_clone.solve_search();
                    let sequential_elapsed = now.elapsed().expect("Error retrieving time");
                    let sequential_status = if solved { "success" } else { "failed" };
                    println!(
                        "sequential done ({}, {}) {}",
                        sequential_status,
                        sequential_elapsed.as_secs(),
                        board_clone.to_string()
                    );
                    board_clone.print();

                    println!("------------------------");
                    println!(
                        "Parallel:   {} ({})",
                        parallel_elapsed.as_secs(),
                        parallel_status
                    );
                    println!(
                        "Sequential: {} ({})",
                        sequential_elapsed.as_secs(),
                        sequential_status
                    );
                    println!("------------------------");
                    Action::ContinueWithoutPrinting
                }
                _ => {
                    println!("Command not recognised.");
                    Action::Continue
                }
            }
        }
        Command::Unrecognised => {
            println!("Command not recognised.");
            Action::Continue
        }
    }
}

trait Printable {
    fn print(&self);
}

impl Printable for Board {
    fn print(&self) {
        let max_digits = self.max_value.to_string().len();
        let spaces = String::from_utf8(vec![b' '; max_digits]).unwrap();
        let max_width = (self.side_length * (2 + max_digits)) + (self.base_num - 1);
        let divider = spaces.clone() + "|" + &String::from_utf8(vec![b'-'; max_width]).unwrap() +
            "-|";

        print!("{}  ", spaces);
        for col in 0..self.side_length {
            print!(" {1:>0$} ", max_digits, col);
            if col % self.base_num == (self.base_num - 1) && col != (self.side_length - 1) {
                print!(" ");
            }
        }
        println!("");
        println!("{}", divider);

        for row in 0..self.side_length {
            print!("{1:>0$}| ", max_digits, row);
            for col in 0..self.side_length {
                print!(
                    " {1:>0$} ",
                    max_digits,
                    match self.get_val((col, row)).unwrap() {
                        Some(num) => num.to_string(),
                        None => "_".to_string(),
                    }
                );
                if col % self.base_num == (self.base_num - 1) && col != (self.side_length - 1) {
                    print!(":");
                }
            }
            print!("|");
            println!("");
            if row % self.base_num == (self.base_num - 1) && row != (self.side_length - 1) {
                println!("{}", divider);
            }
        }
        println!("{}", divider);
        println!("");
    }
}

