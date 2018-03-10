use std::collections::{HashMap};
use rlua::{Lua, Table};
use {Error};

/// Tracks values to be converted to a model for use by the scripting language.
pub struct ScriptTable {
    values: HashMap<String, ScriptValue>,
}

impl ScriptTable {
    /// Creates a new empty model.
    pub fn new() -> Self {
        ScriptTable {
            values: HashMap::new(),
        }
    }

    pub(crate) fn to_lua_table<'l>(&self, lua: &'l Lua) -> Result<Table<'l>, Error> {
        let model_table = lua.create_table()?;

        for (key, value) in &self.values {
            match *value {
                ScriptValue::Bool(value) => model_table.set(key.as_str(), value)?,
                ScriptValue::String(ref value) => model_table.set(key.as_str(), value.as_str())?,
            }
        }

        Ok(model_table)
    }

    /// Sets the field with given key in the model to the given value.
    pub fn set<V: Into<ScriptValue>>(&mut self, key: &str, value: V) {
        self.values.insert(key.into(), value.into());
    }
}

/// A generic value stored in the model.
pub enum ScriptValue {
    Bool(bool),
    String(String),
}

impl From<bool> for ScriptValue {
    fn from(value: bool) -> Self {
        ScriptValue::Bool(value)
    }
}

impl From<String> for ScriptValue {
    fn from(value: String) -> Self {
        ScriptValue::String(value)
    }
}
