use rlua::{Lua};

use scripting::{ScriptTable};
use {Error};

/// Keeps track of the scripting engine and data in it.
pub struct ScriptRuntime {
    lua: Lua,
}

impl ScriptRuntime {
    /// Creates a new runtime.
    pub fn new() -> Self {
        let lua = Lua::new();

        ScriptRuntime {
            lua,
        }
    }

    pub(crate) fn set_model(&self, model: &ScriptTable) -> Result<(), Error> {
        let globals = self.lua.globals();

        let model_table = model.to_lua_table(&self.lua)?;
        globals.set("model", model_table)?;

        Ok(())
    }

    pub(crate) fn eval_bool(&self, source: &str) -> Result<bool, Error> {
        let value = self.lua.eval(source, None)?;
        Ok(value)
    }

    pub(crate) fn eval_integer(&self, source: &str) -> Result<i32, Error> {
        let value = self.lua.eval(source, None)?;
        Ok(value)
    }

    pub(crate) fn eval_float(&self, source: &str) -> Result<f32, Error> {
        let value = self.lua.eval(source, None)?;
        Ok(value)
    }

    pub(crate) fn eval_string(&self, source: &str) -> Result<String, Error> {
        let value = self.lua.eval(source, None)?;
        Ok(value)
    }
}
