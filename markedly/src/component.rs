use nalgebra::{Point2, Vector2};

use class::{ComponentClass};
use scripting::{ScriptRuntime};
use template::{ComponentTemplate, Style, TemplateValue, Attributes};
use {ComponentId, Error, Context, EventSink};

/// A component generated from a template, active in a UI.
pub struct Component {
    pub(crate) class: Box<ComponentClass>,
    pub(crate) style_class: Option<String>,
    pub(crate) event_sink: EventSink,
    pub(crate) needs_render_update: bool,

    pub(crate) children: Vec<ComponentId>,
    pub(crate) attributes: ComponentAttributes,

    template: ComponentTemplate,
}

impl Component {
    pub(crate) fn from_template(
        template: &ComponentTemplate,
        event_sink: EventSink,
        style: &Style,
        parent_size: Vector2<f32>,
        context: &Context,
    ) -> Result<Self, Error> {
        let runtime = &context.runtime;
        let attributes = Attributes::resolve(template, style, context)?;

        let class = context.classes.create(template, &attributes, runtime)?;
        let component_attributes = ComponentAttributes::load(parent_size, &attributes, runtime)?;

        Ok(Component {
            class,
            style_class: template.style_class.clone(),
            event_sink,
            needs_render_update: true,

            children: Vec::new(),
            attributes: component_attributes,

            // This seems very expensive to store, we should look at alternative solutions
            template: template.clone(),
        })
    }

    pub(crate) fn update_attributes(
        &mut self, style: &Style, context: &Context
    ) -> Result<(), Error> {
        let runtime = &context.runtime;
        let attributes = Attributes::resolve(&self.template, style, context)?;
        self.class.update_attributes(&attributes, runtime)?;
        self.needs_render_update = true;

        Ok(())
    }

    pub(crate) fn compute_position(
        &self, parent_size: Vector2<f32>
    ) -> Point2<f32> {
        let x = match self.attributes.docking.0 {
            Docking::Start =>
                self.attributes.position.x,
            Docking::End =>
                self.attributes.position.x +
                    parent_size.x - self.attributes.size.x,
        };
        let y = match self.attributes.docking.1 {
            Docking::Start =>
                self.attributes.position.y,
            Docking::End =>
                self.attributes.position.y +
                    parent_size.y - self.attributes.size.y,
        };

        Point2::new(x, y)
    }
}

/// Core attributes all components share.
pub struct ComponentAttributes {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub docking: (Docking, Docking),
}

impl ComponentAttributes {
    pub fn load(
        parent_size: Vector2<f32>, attributes: &Attributes, runtime: &ScriptRuntime
    ) -> Result<Self, Error> {
        Ok(ComponentAttributes {
            position: attributes.attribute(
                "position", |v| v.as_point(parent_size, runtime), Point2::new(0.0, 0.0)
            )?,
            size: attributes.attribute(
                "size", |v| v.as_vector(parent_size, runtime), parent_size
            )?,
            docking: attributes.attribute(
                "docking", |v| Docking::from_value(v, runtime), (Docking::Start, Docking::Start)
            )?,
        })
    }
}

#[derive(Copy, Clone)]
pub enum Docking {
    Start, End,
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
            "end" => Ok(Docking::End),
            _ => Err("Values must be either \"start\" or \"end\"".into())
        }
    }
}
