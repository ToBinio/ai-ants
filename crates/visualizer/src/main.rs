use crate::renderer::Renderer;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};

use ggez::graphics::{self, Color, Rect};
use ggez::input::keyboard::KeyInput;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use simulation::Simulation;
use std::time::{Duration, Instant};

mod renderer;

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("ai ants", "ToBinio")
        .window_mode(WindowMode::default().resizable(true))
        .build()
        .expect("could not create ggez context!");

    let my_game = SimulationVisualizer::new(&mut ctx).expect("could not initialize game");

    event::run(ctx, event_loop, my_game);
}

struct SimulationVisualizer {
    simulation: Simulation,
    renderer: Renderer,
    render_state: RenderState,
    timings: Timings,
}

struct Timings {
    render: Duration,
    update: Duration,
}

struct RenderState {
    draw_timings: bool,
    draw_pheromones: bool,
}

impl SimulationVisualizer {
    pub fn new(ctx: &mut Context) -> Result<SimulationVisualizer, GameError> {
        Ok(SimulationVisualizer {
            simulation: Simulation::default(),
            renderer: Renderer::new(ctx)?,
            render_state: RenderState {
                draw_timings: true,
                draw_pheromones: false,
            },
            timings: Timings {
                render: Default::default(),
                update: Default::default(),
            },
        })
    }
}

impl EventHandler for SimulationVisualizer {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let instant = Instant::now();

        while ctx.time.check_update_time(60) {
            self.simulation.step();
        }

        self.timings.update = instant.elapsed();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let instant = Instant::now();

        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let screen_size = ctx.gfx.size();
        canvas.set_screen_coordinates(Rect::new(
            -screen_size.0 / 2.,
            -screen_size.1 / 2.,
            screen_size.0,
            screen_size.1,
        ));

        self.renderer.draw(
            &self.simulation,
            &self.timings,
            &self.render_state,
            &mut canvas,
            ctx,
        )?;

        canvas.finish(ctx)?;
        self.timings.render = instant.elapsed();

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        if let Some(key) = input.keycode {
            match key {
                VirtualKeyCode::P => {
                    self.render_state.draw_pheromones = !self.render_state.draw_pheromones
                }
                VirtualKeyCode::S => {
                    self.render_state.draw_timings = !self.render_state.draw_timings
                }
                _ => {}
            }
        }

        Ok(())
    }
}