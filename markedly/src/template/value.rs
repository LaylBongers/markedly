use nalgebra::{Point2, Vector2};
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

    /// Gets the floating point content of this value, calculates a percentage floating point
    /// value, or returns an error.
    pub fn as_float_or_percentage(
        &self, percent_100: f32, runtime: &ScriptRuntime
    ) -> Result<f32, Error> {
        match *self {
            TemplateValue::Float(value) => Ok(value),
            TemplateValue::Percentage(value) => Ok((value as f32 / 100.0) * percent_100),
            TemplateValue::ScriptValue(ref script) => runtime.eval_float(script),
            _ => Err("Value is not a float or percentage".into()),
        }
    }

    pub fn as_vec(&self) -> Result<&Vec<TemplateValue>, Error> {
        if let TemplateValue::Tuple(ref values) = *self {
            Ok(values)
        } else {
            Err("Value is not a tuple".into())
        }
    }

    /// Gets the point content of this value, or returns an error.
    pub fn as_point(
        &self, percent_100: Vector2<f32>, runtime: &ScriptRuntime
    ) -> Result<Point2<f32>, Error> {
        self.as_vector(percent_100, runtime)
            .map(|v| Point2::from_coordinates(v))
    }

    /// Gets the vector content of this value, or returns an error.
    pub fn as_vector(
        &self, percent_100: Vector2<f32>, runtime: &ScriptRuntime
    ) -> Result<Vector2<f32>, Error> {
        if let TemplateValue::Tuple(ref values) = *self {
            if values.len() == 2 {
                let x = values[0].as_float_or_percentage(percent_100.x, runtime)
                    .map_err(|e| Error::new_value("Value 1", e))?;
                let y = values[1].as_float_or_percentage(percent_100.y, runtime)
                    .map_err(|e| Error::new_value("Value 2", e))?;

                Ok(Vector2::new(x, y))
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
