use crate::{RenderState, Timings};
use ggez::glam::vec2;
use ggez::graphics::{Canvas, Color, DrawParam, Mesh, Text, TextFragment};
use ggez::{graphics, Context, GameError, GameResult};
use simulation::Simulation;

pub struct Renderer {
    ant_mesh: Mesh,
    pheromone_mesh: Mesh,
}

impl Renderer {
    pub fn new(ctx: &mut Context) -> Result<Renderer, GameError> {
        let ant_mesh = Mesh::new_ellipse(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            10.0,
            5.0,
            0.1,
            Color::BLACK,
        )?;

        let pheromone_mesh = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            1.0,
            0.1,
            Color::WHITE,
        )?;

        Ok(Renderer {
            ant_mesh,
            pheromone_mesh,
        })
    }

    pub fn draw(
        &mut self,
        simulation: &Simulation,
        timings: &Timings,
        render_state: &RenderState,
        canvas: &mut Canvas,
        ctx: &mut Context,
    ) -> GameResult {
        if render_state.draw_pheromones {
            self.draw_pheromones(simulation, canvas);
        }

        self.draw_ants(simulation, canvas);

        if render_state.draw_timings {
            self.draw_timings(simulation, timings, canvas, ctx);
        }

        Ok(())
    }
    fn draw_ants(&self, simulation: &Simulation, canvas: &mut Canvas) {
        for ant in simulation.ants() {
            let angle = ant.dir();
            let pos = ant.pos();

            canvas.draw(
                &self.ant_mesh,
                DrawParam::new().dest(vec2(pos.x, pos.y)).rotation(angle),
            );
        }
    }

    fn draw_pheromones(&self, simulation: &Simulation, canvas: &mut Canvas) {
        for pheromone in simulation.pheromones() {
            let pos = pheromone.pos();
            let scale = pheromone.size();
            let color = pheromone.color();
            let density = pheromone.density();

            canvas.draw(
                &self.pheromone_mesh,
                DrawParam::new()
                    .dest(vec2(pos.x, pos.y))
                    .scale(vec2(scale, scale))
                    .color(Color::new(color.0, color.1, color.2, density)),
            );
        }
    }

    fn draw_timings(
        &self,
        simulation: &Simulation,
        timings: &Timings,
        canvas: &mut Canvas,
        ctx: &mut Context,
    ) {
        let text = format!(
            "Stats:
fps: {}
render time: {:?} 
update time: {:?}
    ant update time: {:?}
    pheromone update time: {:?}
    pheromone spawn time: {:?}
    pheromone remove time: {:?}
            ",
            ctx.time.fps(),
            timings.render,
            timings.update,
            simulation.timings().ant_updates,
            simulation.timings().pheromone_updates,
            simulation.timings().pheromone_spawn,
            simulation.timings().pheromone_remove
        );

        let text = Text::new(TextFragment::new(text));
        canvas.draw(
            &text,
            DrawParam::new()
                .color(Color::BLACK)
                .dest(vec2(ctx.gfx.size().0 / -2., ctx.gfx.size().1 / -2.)),
        );
    }
}
