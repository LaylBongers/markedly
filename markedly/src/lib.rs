//! A markup based UI engine.

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate nalgebra;
extern crate palette;
extern crate metrohash;
extern crate lyon;
extern crate rlua;

pub mod class;
pub mod input;
pub mod render;
pub mod scripting;
pub mod template;

mod component;
mod error;
mod events;
mod ui;

use component::{Component};

pub use component::{ComponentAttributes};
pub use error::{Error};
pub use events::{EventSink};
pub use ui::{Ui, Context, ComponentId, Tree};
