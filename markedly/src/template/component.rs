use scripting::{ScriptRuntime};
use template::{TemplateValue};
use {Error};

/// A template for a component.
#[derive(Clone, Debug)]
pub struct ComponentTemplate {
    /// The component class this component has.
    pub(crate) class: String,
    /// The style class this component has.
    pub(crate) style_class: Option<String>,
    /// The attributes given to this component.
    pub(crate) attributes: Vec<TemplateAttribute>,
    /// The children of this component.
    pub(crate) children: Vec<ComponentTemplate>,
    /// The line this component is at in the source markup.
    pub(crate) line: usize,
}

/// An attribute-value-conditional combination in a component template.
#[derive(Clone, Debug)]
pub(crate) struct TemplateAttribute {
    pub key: String,
    pub value: TemplateValue,
    pub script_conditional: Option<String>,
}

impl TemplateAttribute {
    pub(crate) fn check_conditional(&self, runtime: &ScriptRuntime) -> Result<bool, Error> {
        if let Some(ref script) = self.script_conditional {
            runtime.eval_bool(script)
        } else {
            Ok(true)
        }
    }
}
