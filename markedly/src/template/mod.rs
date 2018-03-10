//! Templates parsed in from markup.

mod attributes;
mod component;
mod parse;
mod style;
mod template;
mod value;

pub(crate) use self::component::{TemplateAttribute};

pub use self::attributes::{Attributes};
pub use self::component::{ComponentTemplate};
pub use self::style::{Style};
pub use self::template::{Template};
pub use self::value::{TemplateValue, Color, EventHook};
