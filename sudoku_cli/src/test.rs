
use super::*;
use std::fs;

#[test]
fn test_get_command() {
    assert_eq!(Command::Quit, get_command("quit"));
    assert_eq!(
        Command::Set {
            x: 1,
            y: 2,
            val: Some(3),
        },
        get_command("set(1,2,3)")
        );
    assert_eq!(Command::Clear { x: 1, y: 2 }, get_command("clear(1,2)"));
    assert_eq!(Command::Hint { x: 1, y: 2 }, get_command("hint(1,2)"));
    assert_eq!(Command::Check, get_command("check"));
    assert_eq!(Command::Solve(SolveType::Standard), get_command("solve"));
    assert_eq!(
        Command::Solve(SolveType::Standard),
        get_command("  SOLVE  ")
        );
}

#[test]
fn test_process_command() {
    let mut game_board = Board::default();
    assert_eq!(
        Action::Continue,
        process_command(
            Command::Set {
                x: 1,
                y: 2,
                val: Some(3),
            },
            &mut game_board,
            )
        );
    assert_eq!(
        Action::Quit,
        process_command(Command::Quit, &mut game_board)
        );
    assert_eq!(
        Action::Quit,
        process_command(get_command("quit"), &mut game_board)
        );
}

#[test]
fn test_change_square() {
    let mut game_board = Board::default();
    assert_eq!(None, game_board.get_val((0, 0)).unwrap());
    assert_eq!(None, game_board.get_val((1, 2)).unwrap());
    process_command(get_command("set(1,2,3)"), &mut game_board);
    assert_eq!(Some(3), game_board.get_val((1, 2)).unwrap());
    assert_ne!(None, game_board.get_val((1, 2)).unwrap());
    process_command(get_command("clear(1,2)"), &mut game_board);
    assert_eq!(None, game_board.get_val((1, 2)).unwrap());
}

#[ignore]
#[test]
fn test_file() {
    let mut game_board = Board::default();
    process_command(get_command("set(1,2,3)"), &mut game_board);
    process_command(get_command("save(memes.txt)"), &mut game_board);
    process_command(get_command("load(memes.txt)"), &mut game_board);
    fs::remove_file("memes.txt").unwrap();
}

#[ignore]
#[test]
fn test_solve_hard() {
    let mut game_board = Board::default();
    process_command(get_command("load(puzzles/hard.txt)"), &mut game_board);
    process_command(get_command("solve"), &mut game_board);
}
