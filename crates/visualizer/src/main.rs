use crate::renderer::Renderer;
use clap::{arg, Parser};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use std::env;
use std::fs::File;
use std::io::BufReader;

use ggez::graphics::{self, Color, Rect};
use ggez::input::keyboard::KeyInput;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use neural_network::NeuralNetwork;
use simulation::timings::avg_duration::AvgDuration;
use simulation::{Simulation, NEURAL_NETWORK_INPUT_SIZE, NEURAL_NETWORK_OUTPUT_SIZE};
use std::time::{Instant};

mod renderer;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    path: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let neural_network = cli
        .path
        .map(|path| {
            println!("{}", path);

            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        })
        .or_else(|| {
            let mut network =
                NeuralNetwork::new(NEURAL_NETWORK_INPUT_SIZE, NEURAL_NETWORK_OUTPUT_SIZE);
            for _ in 0..20 {
                network.mutate();
            }

            Some(network)
        })
        .unwrap();

    let (mut ctx, event_loop) = ContextBuilder::new("ai ants", "ToBinio")
        .window_mode(WindowMode::default().resizable(true))
        .build()
        .expect("could not create ggez context!");

    let x: Vec<String> = env::args().collect();
    println!("{:?}", x);

    let my_game =
        SimulationVisualizer::new(&mut ctx, neural_network).expect("could not initialize game");

    event::run(ctx, event_loop, my_game);
}

struct SimulationVisualizer {
    simulation: Simulation,
    renderer: Renderer,
    render_state: RenderState,
    timings: Timings,
}

struct Timings {
    render: AvgDuration,
    update: AvgDuration,
}

struct RenderState {
    draw_timings: bool,
    draw_pheromones: bool,
    draw_rays: bool,
}

impl SimulationVisualizer {
    pub fn new(
        ctx: &mut Context,
        neural_network: NeuralNetwork,
    ) -> Result<SimulationVisualizer, GameError> {
        Ok(SimulationVisualizer {
            simulation: Simulation::new(neural_network),
            renderer: Renderer::new(ctx)?,
            render_state: RenderState {
                draw_timings: true,
                draw_pheromones: false,
                draw_rays: false,
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

        self.timings.update.add(&instant.elapsed());

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
        self.timings.render.add(&instant.elapsed());

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

                VirtualKeyCode::R => self.render_state.draw_rays = !self.render_state.draw_rays,
                _ => {}
            }
        }

        Ok(())
    }
}
