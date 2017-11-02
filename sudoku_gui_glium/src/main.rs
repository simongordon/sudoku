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

    println!("Press A to change blending");
    println!("Press S to change clip inside/out");

    let opengl = OpenGL::V3_2;
    let (w, h) = (640, 480);
    let ref mut window: GliumWindow =
        WindowSettings::new("Sudoku", [w, h])
        .exit_on_esc(true).opengl(opengl).build().unwrap();

    let mut blend = Blend::Alpha;
    let mut clip_inside = true;

    let mut g2d = Glium2d::new(opengl, window);

    //let mut glyph_cache = GlyphCache::new(
        //Path::new("assets/FiraSans-Regular.ttf"),
        //window.clone(),
        //TextureSettings::new()
    //).unwrap();

    window.set_lazy(true);
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            use graphics::*;

            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                for i in 0..num_groups {
                    for j in 0..num_groups {
                        let x: f64 = (i as f64) * SQUARE_WIDTH;
                        let y: f64 = (j as f64) * SQUARE_WIDTH;
                        Rectangle::new([1.0, 0.0, 0.0, 1.0])
                            .draw([x, y, SQUARE_WIDTH, SQUARE_WIDTH], &c.draw_state, c.transform, g);

                        Rectangle::new([2.0, 1.0, 1.0, 1.0])
                            .draw([x, y, SQUARE_WIDTH / 2.0, SQUARE_WIDTH / 2.0], &c.draw_state, c.transform, g);

                        //text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw(
                            //"Hello glium_graphics!",
                            //&mut glyph_cache,
                            //&DrawState::default(),
                            //c.transform.trans(10.0, 100.0),
                            //g
                        //).unwrap();
                    }
                }
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);

                clear([0.8, 0.8, 0.8, 1.0], g);
                g.clear_stencil(0);
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);

                let draw_state = c.draw_state.blend(blend);
                Rectangle::new([0.5, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0], &draw_state, c.transform, g);

                Rectangle::new([0.5, 2.0, 0.0, 0.3])
                    .draw([70.0, 70.0, 100.0, 100.0], &draw_state, c.transform, g);

                let transform = c.transform.trans(100.0, 100.0);
                // Compute clip rectangle from upper left corner.
                let (clip_x, clip_y, clip_w, clip_h) = (100, 100, 100, 100);
                let (clip_x, clip_y, clip_w, clip_h) =
                    (clip_x, c.viewport.unwrap().draw_size[1] - clip_y - clip_h, clip_w, clip_h);
                //let clipped = c.draw_state.scissor([clip_x, clip_y, clip_w, clip_h]);
                //Image::new().draw(&rust_logo, &clipped, transform, g);
//
                //let transform = c.transform.trans(200.0, 200.0);
                //Ellipse::new([1.0, 0.0, 0.0, 1.0])
                    //.draw([0.0, 0.0, 50.0, 50.0], &DrawState::new_clip(), transform, g);
                //Image::new().draw(&rust_logo,
                    //&(if clip_inside { DrawState::new_inside() }
                        //else { DrawState::new_outside() }),
                    //transform, g);
            });

            target.finish().unwrap();
        }

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            blend = match blend {
                Blend::Alpha => Blend::Add,
                Blend::Add => Blend::Multiply,
                Blend::Multiply => Blend::Invert,
                Blend::Invert => Blend::Alpha,
            };
            println!("Changed blending to {:?}", blend);
        }

        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            clip_inside = !clip_inside;
            if clip_inside {
                println!("Changed to clip inside");
            } else {
                println!("Changed to clip outside");
            }
        }
    }
}
