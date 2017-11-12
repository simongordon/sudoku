extern crate sudoku;

extern crate glium_graphics;
extern crate graphics;
extern crate piston;

use sudoku::board::*;
use sudoku::board::Position;

use glium_graphics::{Glium2d, GliumWindow, GlyphCache, OpenGL, TextureSettings};
use piston::input::*;
use piston::event_loop::EventLoop;
use piston::window::*;
use piston::window::WindowSettings;
use std::path::Path;

fn main() {
    let default_base_num = 3;
    let mut game_board = Board::from_base_num(default_base_num);
    game_board.set_val((3, 3), Some(3)).unwrap();
    game_board.set_val((4, 3), Some(7)).unwrap();
    game_board.set_val(80, Some(9)).unwrap();

    // Selected square index
    //let mut selector = 14;
    let mut selector = (0, 0);


    let opengl = OpenGL::V3_2;
    let (w, h) = (640, 480);
    let ref mut window: GliumWindow = WindowSettings::new("Sudoku", [w, h])
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let mut g2d = Glium2d::new(opengl, window);

    let mut glyph_cache = GlyphCache::new(
        Path::new("assets/FiraSans-Regular.ttf"),
        window.clone(),
        TextureSettings::new(),
    ).unwrap();

    let black = [0.0, 0.0, 0.0, 1.0];
    let white = [1.0, 1.0, 1.0, 1.0];
    let grey = [0.8, 0.8, 0.8, 1.0];
    // let red = [1.0, 0.0, 0.0, 1.0];
    let yellow = [1.0, 1.0, 0.0, 1.0];
    let orange = [1.0, 165.0 / 255.0, 0.0, 1.0];
    let thingo = 220.0 / 255.0;
    let light_grey = [thingo, thingo, thingo, 1.0];

    let mut show_hints = false;
    let mut show_same_groups = false;
    let mut show_same_nums = false;

    window.set_lazy(true);
    while let Some(e) = window.next() {
        let num_groups = game_board.side_length;
        let size = window.size();
        let w = size.width;
        let h = size.height;
        let smaller = if w < h { w } else { h };

        let square_width = (smaller as f64) / (num_groups as f64);

        if let Some(args) = e.render_args() {
            use graphics::*;
            //use graphics::text::*;

            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                clear(grey, g); // Grey background

                let smaller = smaller as f64;

                Rectangle::new(white).draw(
                    [0.0, 0.0, smaller, smaller],
                    &c.draw_state,
                    c.transform,
                    g,
                );

                let base_num = game_board.base_num;

                let (sel_col, sel_row) = selector;
                let sel_grid = selector.grid_num(base_num * base_num, base_num);

                let selector_val = {
                    if let Ok(val) = game_board.get_val(selector) {
                        val
                    } else {
                        None
                    }
                };

                for col_num in 0..num_groups {
                    let x: f64 = (col_num as f64) * square_width;
                    for row_num in 0..num_groups {
                        let y: f64 = (row_num as f64) * square_width;

                        let curr = (col_num, row_num);
                        let is_selector = curr == selector;

                        let grid_num = (col_num, row_num).grid_num(base_num * base_num, base_num);

                        let curr_val = {
                            if let Ok(val) = game_board.get_val(curr) {
                                val
                            } else {
                                None
                            }
                        };


                        let square_col = {
                            if is_selector {
                                Some(orange)
                            } else if show_same_nums && selector_val.is_some() &&
                                curr_val == selector_val
                            {
                                Some(yellow)
                            } else if show_same_groups &&
                                (col_num == sel_col || row_num == sel_row || grid_num == sel_grid)
                            {
                                Some(light_grey)
                            } else {
                                None
                            }
                        };

                        if let Some(col) = square_col {
                            Rectangle::new(col).draw(
                                [x, y, square_width, square_width],
                                &c.draw_state,
                                c.transform,
                                g,
                            );
                        }

                        let middle = square_width / 2.0;
                        let text_x = x + (middle / 2.0);
                        let text_y = y + (square_width * 0.7);
                        if let Some(val) = curr_val {
                            let square_val = format!("{}", val);
                            text::Text::new_color(black, 34)
                                .draw(
                                    &square_val,
                                    &mut glyph_cache,
                                    &c.draw_state,
                                    c.transform.trans(text_x, text_y),
                                    g,
                                )
                                .unwrap();
                        }
                    }
                }

                for group_num in 0..num_groups + 1 {
                    let is_thicc = (group_num) % base_num == 0;
                    let radius: f64 = if is_thicc { 5.0 } else { 2.0 };

                    //let x: f64 = (col_num as f64) * square_width;
                    //
                    let thingo: f64 = (group_num as f64) * square_width;

                    // Columns
                    let x_start = thingo;
                    let y_start = 0.0;
                    let x_end = thingo;
                    let y_end = smaller as f64;

                    Line::new(black, radius).draw(
                        [x_start, y_start, x_end, y_end],
                        &c.draw_state,
                        c.transform,
                        g,
                    );

                    // Rows
                    let x_start = 0.0;
                    let y_start = thingo;
                    let x_end = smaller as f64;
                    let y_end = thingo;

                    Line::new(black, radius).draw(
                        [x_start, y_start, x_end, y_end],
                        &c.draw_state,
                        c.transform,
                        g,
                    );
                }
            });

            target.finish().unwrap();
        }

        //if let Button::Keyboard(Some(key)) = e.press_args() {
        if let Some(arg) = e.press_args() {
            if let Button::Keyboard(key) = arg {
                let (mut col, mut row) = selector;
                match key {
                    Key::K | Key::Up => if row > 0 {
                        row -= 1;
                    },
                    Key::J | Key::Down => if row < num_groups - 1 {
                        row += 1;
                    },
                    Key::H | Key::Left => if col > 0 {
                        col -= 1;
                    },
                    Key::L | Key::Right => if col < num_groups - 1 {
                        col += 1;
                    },
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
                    Key::A => {
                        show_hints = !show_hints;
                    }
                    Key::G => {
                        show_same_groups = !show_same_groups;
                    }
                    Key::F => {
                        show_same_nums = !show_same_nums;
                    }
                    Key::Backspace => {
                        game_board.set_val(selector, None).unwrap();
                    }
                    Key::D1 => {
                        game_board.set_val(selector, Some(1)).unwrap();
                    }
                    Key::D2 => {
                        game_board.set_val(selector, Some(2)).unwrap();
                    }
                    Key::D3 => {
                        game_board.set_val(selector, Some(3)).unwrap();
                    }
                    Key::D4 => {
                        game_board.set_val(selector, Some(4)).unwrap();
                    }
                    Key::D5 => {
                        game_board.set_val(selector, Some(5)).unwrap();
                    }
                    Key::D6 => {
                        game_board.set_val(selector, Some(6)).unwrap();
                    }
                    Key::D7 => {
                        game_board.set_val(selector, Some(7)).unwrap();
                    }
                    Key::D8 => {
                        game_board.set_val(selector, Some(8)).unwrap();
                    }
                    Key::D9 => {
                        game_board.set_val(selector, Some(9)).unwrap();
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
