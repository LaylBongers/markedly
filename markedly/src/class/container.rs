use class::{ComponentClass, ComponentClassFactory, BackgroundAttributes};
use render::{Renderer};
use scripting::{ScriptRuntime};
use template::{Attributes};
use {Error, ComponentAttributes, ComponentId};

/// A container component class, functions as a generic container for other components.
pub struct ContainerClass {
    background: BackgroundAttributes,
}

impl ComponentClassFactory for ContainerClass {
    fn new(attributes: &Attributes, runtime: &ScriptRuntime) -> Result<Self, Error> {
        Ok(ContainerClass {
            background: BackgroundAttributes::load(attributes, runtime)?,
        })
    }
}

impl ComponentClass for ContainerClass {
    fn update_attributes(
        &mut self, attributes: &Attributes, runtime: &ScriptRuntime,
    ) -> Result<(), Error> {
        self.background = BackgroundAttributes::load(attributes, runtime)?;
        Ok(())
    }

    fn render(
        &self, id: ComponentId, attributes: &ComponentAttributes, renderer: &mut Renderer,
    ) -> Result<(), Error> {
        self.background.render(id, attributes, renderer, false)?;

        Ok(())
    }

    fn is_capturing_cursor(&self) -> bool {
        self.background.is_capturing_cursor()
    }
}
