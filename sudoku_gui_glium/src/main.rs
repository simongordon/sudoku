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
    let mut game_board = Board::default();
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

    window.set_lazy(true);
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            use graphics::*;

            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                clear(grey, g); // Grey background
                let text_image = Image::new_color(red);
                for col_num in 0..num_groups {
                    let x: f64 = (col_num as f64) * SQUARE_WIDTH;
                    for row_num in 0..num_groups {

                        let y: f64 = (row_num as f64) * SQUARE_WIDTH;

                        let full_width = SQUARE_WIDTH;
                        let smaller_width = SQUARE_WIDTH * 0.9;

                        Rectangle::new(black).draw([x, y, full_width, full_width], &c.draw_state, c.transform, g);
                        Rectangle::new(white).draw([x, y, smaller_width, smaller_width], &c.draw_state, c.transform, g);

                        let character = glyph_cache.character(34, 'a').unwrap();
                        text_image.draw(character.texture,
                                        &c.draw_state,
                                        c.transform.trans(x, y),
                                        g);
                    }
                }


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
