use crate::{RenderState, Timings};
use ggez::glam::vec2;
use ggez::graphics::{Canvas, Color, DrawParam, InstanceArray, Mesh, Text, TextFragment};
use ggez::{graphics, Context, GameError, GameResult};
use itertools::izip;
use simulation::ants::{Ants, ANT_SEE_DISTANCE};
use simulation::{Simulation, ANT_HILL_RADIUS, FOOD_SIZE, GAME_SIZE};
use std::f32::consts::PI;

pub struct Renderer {
    ant_mesh: Mesh,
    ant_hill_mesh: Mesh,
    pheromone_mesh: Mesh,
    food_mesh: Mesh,
    map_mesh: Mesh,
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
            Color::WHITE,
        )?;

        let ant_hill_mesh = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            ANT_HILL_RADIUS,
            0.1,
            Color::new(0.8, 0.7, 0.1, 1.),
        )?;

        let pheromone_mesh = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            1.0,
            0.1,
            Color::WHITE,
        )?;

        let food_mesh = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            FOOD_SIZE,
            0.1,
            Color::GREEN,
        )?;

        let map_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: -GAME_SIZE,
                y: -GAME_SIZE,
                w: GAME_SIZE * 2.0,
                h: GAME_SIZE * 2.0,
            },
            Color::new(0.6, 0.4, 0.1, 1.),
        )?;

        Ok(Renderer {
            ant_mesh,
            ant_hill_mesh,
            pheromone_mesh,
            food_mesh,
            map_mesh,
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
        canvas.draw(&self.map_mesh, DrawParam::from(vec2(0., 0.)));

        if render_state.draw_pheromones {
            self.draw_pheromones(simulation, canvas, ctx);
        }

        if render_state.draw_rays {
            self.draw_rays(simulation, canvas, ctx);
        }

        self.draw_ants(simulation, canvas, ctx);
        self.draw_food(simulation, canvas, ctx);

        canvas.draw(&self.ant_hill_mesh, DrawParam::from(vec2(0., 0.)));

        if render_state.draw_timings {
            self.draw_timings(simulation, timings, canvas, ctx);
        }

        Ok(())
    }
    fn draw_ants(&self, simulation: &Simulation, canvas: &mut Canvas, ctx: &mut Context) {
        let mut instances = InstanceArray::new(&ctx.gfx, None);

        for ((pos, dir), carries) in simulation
            .ants()
            .positions
            .iter()
            .zip(simulation.ants().dirs.iter())
            .zip(simulation.ants().caries_foods.iter())
        {
            let color = if *carries { Color::GREEN } else { Color::BLACK };

            instances.push(
                DrawParam::new()
                    .dest(vec2(pos.x, pos.y))
                    .rotation(*dir)
                    .color(color),
            );
        }
        canvas.draw_instanced_mesh(self.ant_mesh.clone(), &instances, DrawParam::new());
    }

    fn draw_pheromones(&self, simulation: &Simulation, canvas: &mut Canvas, ctx: &mut Context) {
        let mut instances = InstanceArray::new(&ctx.gfx, None);

        let pheromones = simulation.pheromones();

        for (index, size) in pheromones
            .sizes
            .iter()
            .enumerate()
            .filter(|(_, size)| size.is_some())
        {
            let scale = size.unwrap();
            //todo extract to function
            let density = 5. / (scale * scale * PI);

            let index_range = simulation.ants().positions.len() * index
                ..simulation.ants().positions.len() * (index + 1);

            for index in index_range {
                let color = pheromones.colors[index];
                let pos = pheromones.positions[index];

                instances.push(
                    DrawParam::new()
                        .dest(vec2(pos.x, pos.y))
                        .scale(vec2(scale, scale))
                        .color(Color::new(color.0, color.1, color.2, density)),
                );
            }
        }

        canvas.draw_instanced_mesh(self.pheromone_mesh.clone(), &instances, DrawParam::new());
    }

    fn draw_food(&self, simulation: &Simulation, canvas: &mut Canvas, ctx: &mut Context) {
        let mut instances = InstanceArray::new(&ctx.gfx, None);

        for food in simulation.foods() {
            let pos = food.pos();

            instances.push(DrawParam::new().dest(vec2(pos.x, pos.y)));
        }

        canvas.draw_instanced_mesh(self.food_mesh.clone(), &instances, DrawParam::new());
    }

    fn draw_rays(&self, simulation: &Simulation, canvas: &mut Canvas, ctx: &mut Context) {
        let mb = &mut graphics::MeshBuilder::new();

        for (pos, dir, rays) in izip!(
            &simulation.ants().positions,
            &simulation.ants().dirs,
            &simulation.ants().rays
        ) {
            for (direction, ray) in izip!(Ants::get_ray_directions(*dir), rays) {
                let point = *pos + direction * ANT_SEE_DISTANCE;
                let color = if *ray == -1. {
                    Color::YELLOW
                } else {
                    Color::GREEN
                };

                mb.line(&[vec2(pos.x, pos.y), vec2(point.x, point.y)], 5., color)
                    .unwrap();
            }
        }

        canvas.draw(&Mesh::from_data(ctx, mb.build()), DrawParam::new());
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
steps: {}
render time: {:?}
update time: {:?}
    ant update time: {:?}
    ant rays update time: {:?}
    neural network update time: {:?}
    keep ants update time: {:?}
    pheromone update time: {:?}
    pheromone spawn time: {:?}
    pheromone remove time: {:?}
    pick up food time: {:?}
    drop off food time: {:?}
            ",
            ctx.time.fps(),
            simulation.stats().step_count,
            timings.render,
            timings.update,
            simulation.timings().ant_updates,
            simulation.timings().see_food,
            simulation.timings().neural_network_updates,
            simulation.timings().keep_ants,
            simulation.timings().pheromone_updates,
            simulation.timings().pheromone_spawn,
            simulation.timings().pheromone_remove,
            simulation.timings().pick_up_food,
            simulation.timings().drop_of_food
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
