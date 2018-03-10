extern crate ggez;
extern crate nalgebra;
extern crate markedly;
extern crate metrohash;

use ggez::conf::{NumSamples};
use ggez::graphics::{self, DrawMode, Rect, Font, Text, Canvas, Matrix4, Mesh};
use ggez::{Context, GameError};
use nalgebra::{Point2, Vector2};
use metrohash::{MetroHashMap};

use markedly::render::{Renderer};
use markedly::template::{Color};
use markedly::{Error, ComponentId};

pub struct GgezComponentCache {
    data: MetroHashMap<ComponentId, Canvas>,
}

impl GgezComponentCache {
    pub fn new() -> Self {
        GgezComponentCache {
            data: MetroHashMap::default(),
        }
    }
}

pub struct GgezRenderer<'a> {
    ctx: &'a mut Context,
    cache: &'a mut GgezComponentCache,
    font: &'a Font,
    target_coordinates: Rect,
}

impl<'a> GgezRenderer<'a> {
    pub fn new(ctx: &'a mut Context, cache: &'a mut GgezComponentCache, font: &'a Font) -> Self {
        let target_coordinates = graphics::get_screen_coordinates(ctx);
        GgezRenderer {
            ctx,
            cache,
            font,
            target_coordinates,
        }
    }

    fn render_to_component(&mut self, id: ComponentId) -> Result<(), Error> {
        let canvas = self.cache.data.get(&id).unwrap();
        graphics::set_canvas(self.ctx, Some(canvas));
        graphics::set_projection(self.ctx, Matrix4::new_orthographic(
            0.0, canvas.get_image().width() as f32,
            0.0, canvas.get_image().height() as f32,
            -1.0, 1.0
        ));
        graphics::apply_transformations(self.ctx).map_err(egtm)?;

        Ok(())
    }
}

impl<'a> Renderer for GgezRenderer<'a> {
    fn render_cache_to_target(&mut self, id: ComponentId) -> Result<(), Error> {
        graphics::set_canvas(self.ctx, None);
        // TODO: Test if we can remove this line
        graphics::set_screen_coordinates(self.ctx, self.target_coordinates).map_err(egtm)?;
        graphics::apply_transformations(self.ctx).map_err(egtm)?;

        let canvas = self.cache.data.get(&id).unwrap();
        graphics::set_color(self.ctx, (255, 255, 255, 255).into()).map_err(egtm)?;
        graphics::draw(self.ctx, canvas, Point2::new(0.0, 0.0), 0.0).map_err(egtm)?;

        Ok(())
    }

    fn create_resize_cache(
        &mut self, id: ComponentId, size: Vector2<u32>
    ) -> Result<bool, Error> {
        // If we have a cached canvas and it's of the right size, we only have to clear
        if let Some(canvas) = self.cache.data.get(&id) {
            if canvas.get_image().width() == size.x &&
                canvas.get_image().height() == size.y {
                return Ok(false)
            }
        }

        // We don't have what we need so create a new canvas
        let canvas = Canvas::new(self.ctx, size.x, size.y, NumSamples::One).map_err(egtm)?;
        self.cache.data.insert(id, canvas);

        Ok(true)
    }

    fn clear_cache(&mut self, id: ComponentId) -> Result<(), Error> {
        let canvas = self.cache.data.get(&id).unwrap();
        graphics::set_canvas(self.ctx, Some(canvas));
        graphics::set_background_color(self.ctx, (255, 255, 255, 0).into());
        graphics::clear(self.ctx);

        Ok(())
    }

    fn render_cache(
        &mut self, id: ComponentId,
        source_id: ComponentId, position: Point2<f32>
    ) -> Result<(), Error> {
        self.render_to_component(id)?;

        let source_canvas = self.cache.data.get(&source_id).unwrap();
        graphics::set_color(self.ctx, (255, 255, 255, 255).into()).map_err(egtm)?;
        graphics::draw(self.ctx, source_canvas, position, 0.0).map_err(egtm)?;

        Ok(())
    }

    fn rectangle(
        &mut self, id: ComponentId,
        position: Point2<f32>, size: Vector2<f32>, color: Color,
    ) -> Result<(), Error> {
        self.render_to_component(id)?;

        graphics::set_color(self.ctx, color_convert(color)).map_err(egtm)?;

        graphics::rectangle(self.ctx, DrawMode::Fill, Rect::new(
            position.x, position.y,
            size.x, size.y,
        )).map_err(egtm)?;

        Ok(())
    }

    fn text(
        &mut self, id: ComponentId,
        text: &str, position: Point2<f32>, size: Vector2<f32>, color: Color,
    ) -> Result<(), Error> {
        self.render_to_component(id)?;

        graphics::set_color(self.ctx, color_convert(color)).map_err(egtm)?;

        let text = Text::new(self.ctx, text, self.font).map_err(egtm)?;

        let x_offset = ((size.x - text.width() as f32) * 0.5).round();
        let y_offset = ((size.y - text.height() as f32) * 0.5).round();
        graphics::set_color(self.ctx, (0, 0, 0, 200).into()).map_err(egtm)?;
        graphics::draw(self.ctx, &text, Point2::new(
            position.x + x_offset,
            position.y + y_offset,
        ), 0.0).map_err(egtm)?;

        Ok(())
    }

    fn vertices(
        &mut self, id: ComponentId,
        vertices: &[Point2<f32>], indices: &[u16], color: Color,
    ) -> Result<(), Error> {
        self.render_to_component(id)?;

        graphics::set_color(self.ctx, color_convert(color)).map_err(egtm)?;

        // Convert the vertices+indices to triangles and then a mesh
        let mut flattened_vertices = Vec::new();
        for index in indices {
            flattened_vertices.push(vertices[*index as usize]);
        }
        let mesh = Mesh::from_triangles(self.ctx, &flattened_vertices).map_err(egtm)?;

        graphics::draw(self.ctx, &mesh, Point2::new(0.0, 0.0), 0.0).map_err(egtm)?;

        Ok(())
    }
}

fn color_convert(color: Color) -> ::ggez::graphics::Color {
    ::ggez::graphics::Color::new(color.red, color.green, color.blue, color.alpha)
}

fn egtm(e: GameError) -> Error {
    Error::Generic { error: Box::new(e) }
}
