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

const SQUARE_WIDTH: f64 = 50.0;

fn main() {
    let mut game_board = Board::default();
    let num_groups = game_board.side_length;


    let opengl = OpenGL::V3_2;
    let (w, h) = (640, 480);
    let ref mut window: GliumWindow =
        WindowSettings::new("Sudoku", [w, h])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    let mut g2d = Glium2d::new(opengl, window);

    window.set_lazy(true);
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            use graphics::*;

            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                for col_num in 0..num_groups {
                    let x: f64 = (col_num as f64) * SQUARE_WIDTH;
                    for row_num in 0..num_groups {

                        let y: f64 = (row_num as f64) * SQUARE_WIDTH;
                        
                        let black = [0.0, 0.0, 0.0, 1.0];
                        let white = [1.0, 1.0, 1.0, 1.0];

                        let full_width = SQUARE_WIDTH;
                        let smaller_width = SQUARE_WIDTH * 0.9;

                        Rectangle::new(black)
                            .draw([x, y, full_width, full_width], &c.draw_state, c.transform, g);

                        Rectangle::new(white)
                            .draw([x, y, smaller_width, smaller_width], &c.draw_state, c.transform, g);
                    }
                }

                // Grey background
                clear([0.8, 0.8, 0.8, 1.0], g);

            });

            target.finish().unwrap();
        }

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            println!("A");
        }

        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            println!("S");
        }
    }
}
