use nalgebra::{Point2, Vector2};

use class::{ComponentClass};
use render::{Renderer};
use scripting::{ScriptRuntime};
use template::{ComponentTemplate, Style, TemplateValue, Attributes, Coordinates};
use {ComponentId, Error, Context, EventSink};

/// A component generated from a template, active in a UI.
pub struct Component {
    class: Box<ComponentClass>,
    style_class: Option<String>,

    event_sink: EventSink,
    needs_rendering: bool,

    children: Vec<ComponentId>,
    attributes: ComponentAttributes,

    template: ComponentTemplate,
}

impl Component {
    pub(crate) fn from_template(
        template: &ComponentTemplate,
        event_sink: EventSink,
        style: &Style,
        context: &Context,
    ) -> Result<Self, Error> {
        let runtime = &context.runtime;
        let attributes = Attributes::resolve(template, style, context)?;

        let class = context.classes.create(template, &attributes, runtime)?;
        let component_attributes = ComponentAttributes::load(&attributes, runtime)?;

        Ok(Component {
            class,
            style_class: template.style_class.clone(),

            event_sink,
            needs_rendering: true,

            children: Vec::new(),
            attributes: component_attributes,

            // This seems very expensive to store, we should look at alternative solutions
            template: template.clone(),
        })
    }

    pub fn class(&self) -> &ComponentClass {
        self.class.as_ref()
    }

    pub fn style_class(&self) -> Option<&String> {
        self.style_class.as_ref()
    }

    pub fn needs_rendering(&self) -> bool {
        self.needs_rendering
    }

    pub(crate) fn mark_rendered(&mut self) {
        self.needs_rendering = false;
    }

    pub fn children(&self) -> &Vec<ComponentId> {
        &self.children
    }

    pub(crate) fn add_child(&mut self, id: ComponentId) {
        self.children.push(id);
    }

    pub fn attributes(&self) -> &ComponentAttributes {
        &self.attributes
    }

    pub(crate) fn render(
        &self, id: ComponentId, computed_size: Vector2<f32>, renderer: &mut Renderer,
    ) -> Result<(), Error> {
        self.class.render(id, &self.attributes, computed_size, renderer)
    }

    pub(crate) fn raise_hover_start_event(&mut self) {
        self.needs_rendering |= self.class.hover_start_event(&mut self.event_sink);
    }

    pub(crate) fn raise_hover_end_event(&mut self) {
        self.needs_rendering |= self.class.hover_end_event(&mut self.event_sink);
    }

    pub(crate) fn raise_pressed_event(&mut self) {
        self.class.pressed_event(&mut self.event_sink);
    }

    pub(crate) fn update_attributes(
        &mut self, style: &Style, context: &Context
    ) -> Result<(), Error> {
        let runtime = &context.runtime;
        let attributes = Attributes::resolve(&self.template, style, context)?;
        self.class.update_attributes(&attributes, runtime)?;
        self.needs_rendering = true;

        Ok(())
    }
}

/// Core attributes all components share.
pub struct ComponentAttributes {
    pub position: Option<Coordinates>,
    pub size: Option<Coordinates>,
    pub docking: (Docking, Docking),
    pub margin: f32,
}

impl ComponentAttributes {
    pub fn load(
        attributes: &Attributes, runtime: &ScriptRuntime
    ) -> Result<Self, Error> {
        Ok(ComponentAttributes {
            position: attributes.attribute_optional(
                "position", |v| v.as_coordinates(runtime),
            )?,
            size: attributes.attribute_optional(
                "size", |v| v.as_coordinates(runtime),
            )?,
            docking: attributes.attribute(
                "docking", |v| Docking::from_value(v, runtime), (Docking::Start, Docking::Start),
            )?,
            margin: attributes.attribute(
                "margin", |v| v.as_float(runtime), 0.0,
            )?,
        })
    }

    pub(crate) fn compute_size(&self, parent_size: Vector2<f32>) -> Vector2<f32> {
        self.size
            .map(|v| v.to_vector(parent_size))
            .unwrap_or(parent_size)
    }

    pub(crate) fn compute_position(
        &self, parent_size: Vector2<f32>, parent_flow: &mut ComponentFlow
    ) -> Point2<f32> {
        let size = self.compute_size(parent_size);

        if let Some(position) = self.position {
            let position = position.to_point(parent_size);

            // If we have a position, we need to use that
            let x = match self.docking.0 {
                Docking::Start =>
                    position.x,
                Docking::Middle =>
                    position.x + (parent_size.x - size.x)*0.5,
                Docking::End =>
                    position.x + parent_size.x - size.x,
            };
            let y = match self.docking.1 {
                Docking::Start =>
                    position.y,
                Docking::Middle =>
                    position.y + (parent_size.y - size.y)*0.5,
                Docking::End =>
                    position.y + parent_size.y - size.y,
            };

            Point2::new(x, y)
        } else {
            // If we don't have a position, we need to automatically calculate it
            parent_flow.position(size, self.margin)
        }
    }
}

#[derive(Copy, Clone)]
pub enum Docking {
    Start, Middle, End,
}

impl Docking {
    pub fn from_value(
        value: &TemplateValue, runtime: &ScriptRuntime
    ) -> Result<(Self, Self), Error> {
        let vec = value.as_vec()?;

        if vec.len() != 2 {
            return Err("Tuple is incorrect size".into())
        }

        Ok((
            Self::from_value_individual(&vec[0], runtime)?,
            Self::from_value_individual(&vec[1], runtime)?,
        ))
    }

    fn from_value_individual(
        value: &TemplateValue, runtime: &ScriptRuntime
    ) -> Result<Self, Error> {
        match value.as_string(runtime)?.as_str() {
            "start" => Ok(Docking::Start),
            "middle" => Ok(Docking::Middle),
            "end" => Ok(Docking::End),
            _ => Err("Values must be either \"start\" or \"end\"".into())
        }
    }
}

pub struct ComponentFlow {
    limits: Vector2<f32>,
    pointer: Point2<f32>,
    pointer_margin: f32,
    next_line: f32,
}

impl ComponentFlow {
    pub fn new(limits: Vector2<f32>) -> Self {
        ComponentFlow {
            limits,
            pointer: Point2::new(0.0, 0.0),
            pointer_margin: 0.0,
            next_line: 0.0,
        }
    }

    pub fn position(&mut self, size: Vector2<f32>, margin: f32) -> Point2<f32> {
        // TODO: This function is a perfect unit testing candidate
        // TODO: Vertical margin is incorrect right now, instead of correctly overlapping line
        //  margins, it just uses the current component's margin on top. This needs to be changed
        //  to instead properly calculate lines at a time before rendering.

        // The total margin is always the maximum, margins overloap
        // These margins are by how much this component needs to be offset
        let max_x_margin = self.pointer_margin.max(margin);

        // Make sure the next position in this line doesn't overflow the line
        // If it does, go to the next line
        let next_x = self.pointer.x + max_x_margin;
        let position = if next_x + size.x <= self.limits.x {
            Point2::new(next_x, self.pointer.y + margin)
        } else {
            Point2::new(margin, self.next_line + margin)
        };

        self.pointer = position + Vector2::new(size.x, -margin);
        self.pointer_margin = margin;
        self.next_line = (position.y + size.y).max(self.next_line);

        position
    }
}
