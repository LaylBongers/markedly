use std::io::{Read};

use pest::{Parser};

use template::parse::{self, TemplateParser, Rule};
use template::{ComponentTemplate};

/// A style template, used to define default values and style classes for use in templates.
#[derive(Debug)]
pub struct Style {
    pub components: Vec<ComponentTemplate>,
}

impl Style {
    /// Parses a style from a reader, such as a `File`.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, String> {
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();
        Self::from_str(&text)
    }

    /// Parses a style from a string.
    pub fn from_str(text: &str) -> Result<Self, String> {
        // Parse and extract the template pair
        let pairs = TemplateParser::parse(Rule::template, text)
            // This gives a pretty error to our caller
            .map_err(|e| format!("{}", e))?;
        let template_pair = pairs.into_iter().next().unwrap();

        let document = parse::parse_document(template_pair)?;

        Ok(Style {
            components: document,
        })
    }
}

#[cfg(test)]
mod test {
    use template::{Style};
    use {Value};

    #[test]
    fn it_parses_multiple_roots() {
        let result = Style::from_str("root1\nroot2\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let style = result.unwrap();
        assert_eq!(style.components.len(), 2);
        assert_eq!(style.components[0].class, "root1");
        assert_eq!(style.components[1].class, "root2");
    }
}
