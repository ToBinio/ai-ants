use ggez::glam::{vec2, Vec2};
use ggez::graphics::{Canvas, Color, DrawParam, Drawable, Mesh, MeshBuilder, Text, TextFragment};
use ggez::input::mouse::CursorIcon::Default;
use ggez::{graphics, Context, GameError, GameResult};
use simulation::Simulation;

pub struct Renderer {
    ant_mesh: Mesh,
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

        Ok(Renderer { ant_mesh })
    }

    pub fn draw(
        &mut self,
        simulation: &Simulation,
        canvas: &mut Canvas,
        ctx: &mut Context,
    ) -> GameResult {
        for ant in simulation.ants() {
            let angle = ant.dir();
            let pos = ant.pos();

            canvas.draw(
                &self.ant_mesh,
                DrawParam::new().dest(vec2(pos.x, pos.y)).rotation(angle),
            );
        }

        Ok(())
    }
}
