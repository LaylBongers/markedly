use std::collections::{HashMap};

use template::{ComponentTemplate, Style, TemplateValue};
use {Error, Context};

/// A generated attribute bundle for a component, used by the component and its class to receive
/// data from templates and styles.
pub struct Attributes {
    attributes: HashMap<String, TemplateValue>,
    component_class: String,
    component_line: usize,
}

impl Attributes {
    /// Resolves the final attributes of the current component from its template and the style.
    pub fn resolve(
        template: &ComponentTemplate, style: &Style, context: &Context,
    ) -> Result<Self, Error> {
        let mut attributes = HashMap::new();

        // Attributes should always be added, and thus overwritten, in the sequence they were in in
        // the template

        // Add any styles from the stylesheet
        for component in &style.components {
            if component.class == template.class {
                for attribute in &component.attributes {
                    if attribute.check_conditional(&context.runtime)? {
                        attributes.insert(attribute.key.clone(), attribute.value.clone());
                    }
                }
            }
        }

        // Overwrite any style resolved attributes with this component's set attributes
        for attribute in &template.attributes {
            if attribute.check_conditional(&context.runtime)? {
                attributes.insert(attribute.key.clone(), attribute.value.clone());
            }
        }

        Ok(Attributes {
            component_class: template.class.clone(),
            component_line: template.line,
            attributes,
        })
    }

    pub fn attribute<O, F: FnOnce(&TemplateValue) -> Result<O, Error>>(
        &self, key: &str, map: F, default: O
    ) -> Result<O, Error> {
        self.attributes.get(key)
            .map(map)
            .unwrap_or(Ok(default))
            // Error reporting here is done by what component is being resolved, rather than
            // where the attribute came from, for example a style file. Both of these are relevant
            // information for resolving the error, so this needs to be changed to both.
            .map_err(|error| Error::Attribute {
                component: self.component_class.clone(),
                line: self.component_line,
                field: key.into(),
                inner: Box::new(error),
            })
    }

    pub fn attribute_optional<O, F: FnOnce(&TemplateValue) -> Result<O, Error>>(
        &self, key: &str, map: F,
    ) -> Result<Option<O>, Error> {
        self.attributes.get(key)
            .map(|value| {
                if *value == TemplateValue::Default {
                    Ok(None)
                } else {
                    map(value).map(|v| Some(v))
                }
            })
            .unwrap_or(Ok(None))
            // Error reporting here is done by what component is being resolved, rather than
            // where the attribute came from, for example a style file. Both of these are relevant
            // information for resolving the error, so this needs to be changed to both.
            .map_err(|error| Error::Attribute {
                component: self.component_class.clone(),
                line: self.component_line,
                field: key.into(),
                inner: Box::new(error),
            })
    }
}
