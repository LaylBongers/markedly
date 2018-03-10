use std::collections::{HashMap};

use render::{Renderer};
use scripting::{ScriptRuntime};
use template::{ComponentTemplate, Attributes};
use {EventSink, ComponentAttributes, Error, ComponentId};

/// The class of a component, defines specific appearance and functionality in response to user
/// input.
pub trait ComponentClass {
    fn update_attributes(
        &mut self, attributes: &Attributes, runtime: &ScriptRuntime,
    ) -> Result<(), Error>;

    /// Renders the component.
    fn render(
        &self, id: ComponentId, attributes: &ComponentAttributes, renderer: &mut Renderer,
    ) -> Result<(), Error>;

    /// Returns if this component class captures cursor events or not. Does not affect children.
    fn is_capturing_cursor(&self) -> bool { false }

    /// Called when the cursor starts hovering over this component.
    /// Returns if the component should be marked for render update.
    fn hover_start_event(&mut self, _event_sink: &mut EventSink) -> bool { false }

    /// Called when the cursor stops hovering over this component.
    /// Returns if the component should be marked for render update.
    fn hover_end_event(&mut self, _event_sink: &mut EventSink) -> bool { false }

    /// Called when the component is clicked or tapped.
    fn pressed_event(&mut self, _event_sink: &mut EventSink) {}
}


/// A registry of component class factories.
pub struct ComponentClasses {
    factories: HashMap<String, Box<
        Fn(&Attributes, &ScriptRuntime) -> Result<Box<ComponentClass>, Error>
    >>,
}

impl ComponentClasses {
    /// Creates a new registry.
    pub fn new() -> Self {
        ComponentClasses {
            factories: HashMap::new(),
        }
    }

    /// Registers a component class by name.
    pub fn register<F: ComponentClassFactory>(
        &mut self, class: &str
    ) {
        self.factories.insert(class.into(), Box::new(|attributes, runtime| {
            let class = F::new(attributes, runtime)?;
            Ok(Box::new(class))
        }));
    }

    /// Creates a new boxed instance of the component class requested in the template.
    pub fn create(
        &self, template: &ComponentTemplate, attributes: &Attributes, runtime: &ScriptRuntime,
    ) -> Result<Box<ComponentClass>, Error> {
        let component_class = self.factories
            .get(&template.class)
            .ok_or(format!("Component class \"{}\" was not registered", template.class))?
            (attributes, runtime)?;

        Ok(component_class)
    }
}

/// A factory trait to allow component classes to define their factory function.
pub trait ComponentClassFactory: Sized + ComponentClass + 'static {
    /// Creates a new instance of the component class.
    fn new(attributes: &Attributes, runtime: &ScriptRuntime) -> Result<Self, Error>;
}
