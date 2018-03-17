use nalgebra::{Vector2, Point2};
use scripting::{ScriptRuntime};
use {Error};

/// A template value, to be interpreted by components when created or updated.
#[derive(Debug, PartialEq, Clone)]
pub enum TemplateValue {
    /// A string text value.
    String(String),
    /// An integer numeric value.
    Integer(i32),
    /// A floating-point numeric value.
    Float(f32),
    /// An integer percentage value.
    Percentage(i32),
    /// A tuple of values.
    Tuple(Vec<TemplateValue>),
    /// A null value.
    Default,
    /// A script that will be evaluated by the scripting engine.
    ScriptValue(String),
    /// A script statement that will be executed by the scripting engine.
    /// For example, allowing more complex responses to UI events in the template.
    ScriptStatement(String),
}

impl TemplateValue {
    /// Gets the string content of this value, or returns an error.
    pub fn as_string(&self, runtime: &ScriptRuntime) -> Result<String, Error> {
        match *self {
            TemplateValue::String(ref value) => Ok(value.clone()),
            TemplateValue::ScriptValue(ref script) => runtime.eval_string(script),
            _ => Err("Value is not a string".into()),
        }
    }

    /// Gets the integer content of this value, or returns an error.
    pub fn as_integer(&self, runtime: &ScriptRuntime) -> Result<i32, Error> {
        match *self {
            TemplateValue::Integer(value) => Ok(value),
            TemplateValue::ScriptValue(ref script) => runtime.eval_integer(script),
            _ => Err("Value is not an integer".into()),
        }
    }

    /// Gets the floating point content of this value, or returns an error.
    pub fn as_float(&self, runtime: &ScriptRuntime) -> Result<f32, Error> {
        match *self {
            TemplateValue::Float(value) => Ok(value),
            TemplateValue::ScriptValue(ref script) => runtime.eval_float(script),
            _ => Err("Value is not a float".into()),
        }
    }

    pub fn as_vec(&self) -> Result<&Vec<TemplateValue>, Error> {
        if let TemplateValue::Tuple(ref values) = *self {
            Ok(values)
        } else {
            Err("Value is not a tuple".into())
        }
    }

    /// Gets the Size content of this value, which can be either an exact floating point value, or
    /// a percentage relative to the parent.
    pub fn as_coordinate(
        &self, runtime: &ScriptRuntime
    ) -> Result<Coordinate, Error> {
        match *self {
            TemplateValue::Float(value) => Ok(Coordinate::Exact(value)),
            TemplateValue::Percentage(value) =>
                Ok(Coordinate::RelativeToParent(value as f32 / 100.0)),
            TemplateValue::ScriptValue(ref script) =>
                Ok(Coordinate::Exact(runtime.eval_float(script)?)),
            _ => Err("Value is not a float or percentage".into()),
        }
    }

    pub fn as_coordinates(
        &self, runtime: &ScriptRuntime
    ) -> Result<Coordinates, Error> {
        if let TemplateValue::Tuple(ref values) = *self {
            if values.len() == 2 {
                let x = values[0].as_coordinate(runtime)
                    .map_err(|e| Error::new_value("Value 1", e))?;
                let y = values[1].as_coordinate(runtime)
                    .map_err(|e| Error::new_value("Value 2", e))?;

                Ok(Coordinates::new(x, y))
            } else {
                Err("Tuple is incorrect size".into())
            }
        } else {
            Err("Value is not a tuple".into())
        }
    }

    /// Gets the color content of this value, or returns an error.
    pub fn as_color(&self, runtime: &ScriptRuntime) -> Result<Color, Error> {
        if let TemplateValue::Tuple(ref values) = *self {
            let has_alpha = values.len() == 4;
            if values.len() == 3 || has_alpha {
                let red = values[0].as_integer(runtime)
                    .map_err(|e| Error::new_value("Value 1", e))?;
                let green = values[1].as_integer(runtime)
                    .map_err(|e| Error::new_value("Value 2", e))?;
                let blue = values[2].as_integer(runtime)
                    .map_err(|e| Error::new_value("Value 3", e))?;
                let alpha = if has_alpha {
                    let alpha = values[3].as_float(runtime)
                        .map_err(|e| Error::new_value("Value 4", e))?;
                    range_f(alpha, "Value 4", 0.0, 1.0)?;
                    (255.0 * alpha).round() as u8
                } else {
                    255
                };

                range_i(red, "Value 1", 0, 255)?;
                range_i(green, "Value 2", 0, 255)?;
                range_i(blue, "Value 3", 0, 255)?;

                Ok(Color::new_u8(red as u8, green as u8, blue as u8, alpha))
            } else {
                Err("Tuple is incorrect size".into())
            }
        } else {
            Err("Value is not a tuple".into())
        }
    }

    pub fn as_event_hook(&self, runtime: &ScriptRuntime) -> Result<EventHook, Error> {
        match *self {
            TemplateValue::String(ref value) => Ok(EventHook::Direct(value.clone())),
            TemplateValue::ScriptValue(ref script) =>
                Ok(EventHook::Direct(runtime.eval_string(script)?)),
            TemplateValue::ScriptStatement(ref script) =>
                Ok(EventHook::Script(script.clone())),
            _ => Err("Value is not a string or script statement".into()),
        }
    }
}

fn range_i(value: i32, err_id: &str, min: i32, max: i32) -> Result<(), String> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("{}: Out of range, valid range is {} to {}", err_id, min, max))
    }
}

fn range_f(value: f32, err_id: &str, min: f32, max: f32) -> Result<(), String> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("{}: Out of range, valid range is {} to {}", err_id, min, max))
    }
}

/// Re-export of palette's color for convenience so you don't have to add palette to your own
/// crate unless you need more complex color functionality.
pub type Color = ::palette::Srgba;

pub enum EventHook {
    Direct(String),
    Script(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Coordinate {
    Exact(f32),
    RelativeToParent(f32),
}

impl Coordinate {
    pub fn to_float(self, parent_container: f32) -> f32 {
        match self {
            Coordinate::Exact(value) => value,
            Coordinate::RelativeToParent(value) => parent_container * value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Coordinates {
    pub x: Coordinate,
    pub y: Coordinate,
}

impl Coordinates {
    pub fn new(x: Coordinate, y: Coordinate) -> Self {
        Coordinates {
            x, y,
        }
    }

    pub fn from_vector(value: Vector2<f32>) -> Self {
        Coordinates {
            x: Coordinate::Exact(value.x),
            y: Coordinate::Exact(value.y),
        }
    }

    pub fn from_point(value: Point2<f32>) -> Self {
        Coordinates {
            x: Coordinate::Exact(value.x),
            y: Coordinate::Exact(value.y),
        }
    }

    pub fn to_vector(&self, parent_container: Vector2<f32>) -> Vector2<f32> {
        Vector2::new(
            self.x.to_float(parent_container.x),
            self.y.to_float(parent_container.y),
        )
    }

    pub fn to_point(&self, parent_container: Vector2<f32>) -> Point2<f32> {
        Point2::from_coordinates(self.to_vector(parent_container))
    }
}
