use crate::renderer::Renderer;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::glam::{vec2, Vec2};
use ggez::graphics::{self, Color, Rect};
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
    timings: Timings,
}

struct Timings {
    render: Duration,
    update: Duration,
}

impl SimulationVisualizer {
    pub fn new(ctx: &mut Context) -> Result<SimulationVisualizer, GameError> {
        Ok(SimulationVisualizer {
            simulation: Simulation::new(),
            renderer: Renderer::new(ctx)?,
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

        self.renderer
            .draw(&self.simulation, &self.timings, &mut canvas, ctx)?;

        canvas.finish(ctx)?;
        self.timings.render = instant.elapsed();

        Ok(())
    }
}
