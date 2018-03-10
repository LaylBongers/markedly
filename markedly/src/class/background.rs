use nalgebra::{Point2};
use lyon::math::rect;
use lyon::tessellation as lt;

use render::{Renderer};
use scripting::{ScriptRuntime};
use template::{Attributes, Color};
use {Error, ComponentAttributes, ComponentId};

pub struct BackgroundAttributes {
    color: Option<Color>,
    color_hovering: Option<Color>,
    border_radius: f32,
}

impl BackgroundAttributes {
    pub fn load(attributes: &Attributes, runtime: &ScriptRuntime) -> Result<Self, Error> {
        Ok(BackgroundAttributes {
            color: attributes.attribute_optional("color", |v| v.as_color(runtime))?,
            color_hovering: attributes.attribute_optional(
                "color-hovering", |v| v.as_color(runtime)
            )?,
            border_radius: attributes.attribute("border-radius", |v| v.as_float(runtime), 0.0)?,
        })
    }

    pub fn render(
        &self, id: ComponentId, attributes: &ComponentAttributes, renderer: &mut Renderer,
        hovering: bool,
    ) -> Result<(), Error> {
        let current_color = if hovering && self.color_hovering.is_some() {
            self.color_hovering
        } else {
            self.color
        };

        if let Some(color) = current_color {
            if self.border_radius == 0.0 {
                renderer.rectangle(id, Point2::new(0.0, 0.0), attributes.size, color)?;
            } else {
                // Generate the rounded rectangle
                let mut geometry = lt::VertexBuffers::new();
                let options = lt::FillOptions::tolerance(0.1);
                lt::basic_shapes::fill_rounded_rectangle(
                    &rect(0.0, 0.0, attributes.size.x, attributes.size.y),
                    &lt::basic_shapes::BorderRadii {
                        top_left: self.border_radius,
                        top_right: self.border_radius,
                        bottom_left: self.border_radius,
                        bottom_right: self.border_radius,
                    },
                    &options,
                    &mut lt::geometry_builder::simple_builder(&mut geometry),
                );

                // Send it over to the renderer
                let vertices: Vec<_> = geometry.vertices.into_iter()
                    .map(|v| Point2::new(v.position.x, v.position.y)).collect();
                renderer.vertices(
                    id,
                    &vertices,
                    &geometry.indices,
                    color,
                )?;
            }
        }

        Ok(())
    }

    pub fn is_capturing_cursor(&self) -> bool {
        self.color.is_some()
    }
}
