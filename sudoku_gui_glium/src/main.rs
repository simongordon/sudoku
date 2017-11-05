extern crate sudoku;

extern crate graphics;
extern crate glium_graphics;
extern crate piston;

use sudoku::board::*;
use sudoku::solver::*;
use sudoku::hintmap::HintMap;

use glium_graphics::{
    Flip, Glium2d, GliumWindow, GlyphCache, OpenGL, Texture, TextureSettings
};
use piston::input::*;
use piston::event_loop::EventLoop;
use piston::window::WindowSettings;
use graphics::draw_state::Blend;
use std::path::Path;

use graphics::character::*;

const SQUARE_WIDTH: f64 = 50.0;

fn main() {
    let default_base_num = 3;
    let mut game_board = Board::from_base_num(default_base_num);
    game_board.set_val((3, 3), Some(3)).unwrap();
    game_board.set_val((4, 3), Some(7)).unwrap();
    game_board.set_val(80, Some(9)).unwrap();

    // Selected square index
    //let mut selector = 14;
    let mut selector = (0, 0);

    let num_groups = game_board.side_length;

    let opengl = OpenGL::V3_2;
    let (w, h) = (640, 480);
    let ref mut window: GliumWindow =
        WindowSettings::new("Sudoku", [w, h])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    let mut g2d = Glium2d::new(opengl, window);

    let mut glyph_cache = GlyphCache::new(
        Path::new("assets/FiraSans-Regular.ttf"),
        window.clone(),
        TextureSettings::new()
    ).unwrap();

    let black = [0.0, 0.0, 0.0, 1.0];
    let white = [1.0, 1.0, 1.0, 1.0];
    let grey = [0.8, 0.8, 0.8, 1.0];
    let red = [1.0, 0.0, 0.0, 1.0];
    let yellow = [1.0, 1.0, 0.0, 1.0];

    window.set_lazy(true);
    while let Some(e) = window.next() {
        let num_groups = game_board.side_length;
        let (w, h) = (640, 480);
        let smaller = if w < h {
            w
        }
        else {
            h
        };

        //let square_width = SQUARE_WIDTH;
        let square_width = (smaller as f64) / (num_groups as f64);

        if let Some(args) = e.render_args() {
            use graphics::*;
            //use graphics::text::*;

            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                clear(grey, g); // Grey background

                for col_num in 0..num_groups {
                    let x: f64 = (col_num as f64) * square_width;
                    for row_num in 0..num_groups {

                        let y: f64 = (row_num as f64) * square_width;

                        let full_width = square_width;
                        let smaller_width = square_width * 0.9;

                        let curr = (col_num, row_num).into_pos(num_groups);
                        let selector = selector.into_pos(num_groups);
                        
                        let chosen_col = if curr == selector {
                            yellow
                        }
                        else {
                            white
                        };

                        Rectangle::new(black).draw([x, y, full_width, full_width], &c.draw_state, c.transform, g);
                        Rectangle::new(chosen_col).draw([x, y, smaller_width, smaller_width], &c.draw_state, c.transform, g);

                        let middle = SQUARE_WIDTH / 2.0;
                        let text_x = x;
                        let text_y = y + middle;
                        if let Some(val) = (game_board.get_val((col_num, row_num)).unwrap() ) {
                            let square_val = format!("{}", val);
                            text::Text::new_color(red, 34).draw(&square_val, &mut glyph_cache, &c.draw_state, c.transform.trans(text_x, text_y), g);
                        }
                    }
                }

            });

            target.finish().unwrap();
        }

        //if let Button::Keyboard(Some(key)) = e.press_args() {
        if let Some(arg) = e.press_args() {
            if let Button::Keyboard(key) = arg {
                let (mut col, mut row) = selector;
                match key {
                    Key::K | Key::Up => {
                        if (row > 0) {
                            row -= 1;
                        }
                    }
                    Key::J | Key::Down => {
                        if (row < num_groups - 1) {
                            row += 1;
                        }
                    }
                    Key::H | Key::Left => {
                        if (col > 0) {
                            col -= 1;
                        }
                    }
                    Key::L | Key::Right => {
                        if (col < num_groups - 1) {
                            col += 1;
                        }
                    }
                    Key::S => {
                        game_board.solve_search_parallel();
                    }
                    Key::R => {
                        let base_num = game_board.base_num;
                        game_board = Board::from_base_num(base_num);
                    }
                    Key::N => {
                        let base_num = game_board.base_num;
                        game_board = Board::generate_new(base_num).unwrap();
                    }
                    Key::Plus | Key::RightBracket => {
                        let base_num = game_board.base_num;
                        game_board = Board::from_base_num(base_num + 1);
                        col = 0;
                        row = 0;
                    }
                    Key::Minus | Key::LeftBracket => {
                        let base_num = game_board.base_num;
                        println!("Base num: {}", base_num);
                        game_board = Board::from_base_num(base_num - 1);
                        col = 0;
                        row = 0;
                    }
                    Key::Backspace => {
                        game_board.set_val(selector, None).unwrap();
                    }
                    Key::D1 => {
                        game_board.set_val(selector, Some(1)).unwrap();
                    }
                    _ => {
                        println!("Another key pushed");
                    }
                }
                selector = (col, row);
            }
        }

    }
}
