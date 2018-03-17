extern crate ggez;
extern crate nalgebra;
extern crate markedly;
extern crate metrohash;

use std::path::{PathBuf};

use nalgebra::{Point2, Vector2};
use metrohash::{MetroHashMap};
use ggez::conf::{NumSamples};
use ggez::graphics::{self, DrawMode, Rect, Font, Text, Canvas, Matrix4, Mesh};
use ggez::{Context, GameError};

use markedly::render::{Renderer};
use markedly::template::{Color};
use markedly::{Error, ComponentId};

struct FontCache {
    path: PathBuf,
    sizes: MetroHashMap<u32, Font>,
}

/// A persistent resource cache for the ggez markedly renderer.
pub struct GgezCache {
    data: MetroHashMap<ComponentId, Canvas>,
    fonts: MetroHashMap<String, FontCache>,

    default_font: Option<String>,
    default_text_size: u32,
}

impl GgezCache {
    pub fn new() -> Self {
        GgezCache {
            data: MetroHashMap::default(),
            fonts: MetroHashMap::default(),

            default_font: None,
            default_text_size: 14,
        }
    }

    /// Adds a font to the cache by its path.
    /// This will not actually load the font until it's used with a specific size.
    pub fn add_font<S: Into<String>, P: Into<PathBuf>>(
        &mut self, name: S, location: P
    ) -> Result<(), Error> {
        let name = name.into();

        if self.default_font.is_none() {
            self.default_font = Some(name.clone());
        }

        if self.fonts.contains_key(&name) {
            return Err(Error::Resource {
                resource: Some(name),
                error: "Font already added to cache".into(),
            })
        }

        self.fonts.insert(name, FontCache {
            path: location.into(),
            sizes: MetroHashMap::default(),
        });

        Ok(())
    }
}

/// A markedly renderer for ggez, intended to be constructed every frame on-demand.
pub struct GgezRenderer<'a> {
    ctx: &'a mut Context,
    cache: &'a mut GgezCache,
    target_coordinates: Rect,
}

impl<'a> GgezRenderer<'a> {
    pub fn new(ctx: &'a mut Context, cache: &'a mut GgezCache) -> Self {
        let target_coordinates = graphics::get_screen_coordinates(ctx);
        GgezRenderer {
            ctx,
            cache,
            target_coordinates,
        }
    }

    fn render_to_component(&mut self, id: ComponentId) -> Result<(), Error> {
        let canvas = self.cache.data.get(&id).unwrap();
        graphics::set_canvas(self.ctx, Some(canvas));
        graphics::set_screen_coordinates(self.ctx, Rect::new(
            0.0, 0.0,
            canvas.get_image().width() as f32, canvas.get_image().height() as f32,
        )).map_err(egtm)?;
        graphics::apply_transformations(self.ctx).map_err(egtm)?;

        Ok(())
    }
}

impl<'a> Renderer for GgezRenderer<'a> {
    fn render_cache_to_target(&mut self, id: ComponentId) -> Result<(), Error> {
        graphics::set_canvas(self.ctx, None);
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
        graphics::draw(self.ctx, source_canvas, Point2::new(
            position.x.round(),
            position.y.round(),
        ), 0.0).map_err(egtm)?;

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
        text: &String, text_font: Option<&String>, text_size: Option<i32>,
        position: Point2<f32>, size: Vector2<f32>, color: Color,
    ) -> Result<(), Error> {
        self.render_to_component(id)?;

        // Try to find the font cache, use the default, or error if we can't find it
        let requested_font_name = text_font.or(self.cache.default_font.as_ref())
            .ok_or(Error::Resource {
                resource: None,
                error: "Could not fall back to default font, no fonts are loaded".into()
            })?;
        let font_cache = self.cache.fonts.get_mut(requested_font_name)
            .ok_or_else(|| Error::Resource {
                resource: Some(requested_font_name.clone()),
                error: "Font is not in cache".into()
            })?;

        // Find the cached size for this font, or generate a cache for that
        let text_size = text_size.map(|v| v as u32).unwrap_or(self.cache.default_text_size);
        if !font_cache.sizes.contains_key(&text_size) {
            let font = Font::new(self.ctx, &font_cache.path, text_size).map_err(egtm)?;
            font_cache.sizes.insert(text_size, font);
        }
        let font = font_cache.sizes.get(&text_size).unwrap();

        let text = Text::new(self.ctx, text, font).map_err(egtm)?;

        let x_offset = (size.x - text.width() as f32) * 0.5;
        let y_offset = (size.y - text.height() as f32) * 0.5;
        graphics::set_color(self.ctx, color_convert(color)).map_err(egtm)?;
        graphics::draw(self.ctx, &text, Point2::new(
            (position.x + x_offset).round(),
            (position.y + y_offset).round(),
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

/// Converts a ggez error to a markedly error.
pub fn egtm(e: GameError) -> Error {
    Error::Generic { error: Box::new(e) }
}

/// Converts a markedly error to a ggez error.
pub fn emtg(e: Error) -> GameError {
    GameError::UnknownError(format!("{:#?}", e))
}
