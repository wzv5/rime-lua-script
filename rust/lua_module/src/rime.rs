use mlua::prelude::*;

pub struct RimeGlobal {
    pub log: Log,
}

impl RimeGlobal {
    pub fn new(lua: &Lua) -> LuaResult<Self> {
        Ok(Self {
            log: lua.globals().get("log")?,
        })
    }
}

pub struct Log {
    info: LuaFunction,
    warning: LuaFunction,
    error: LuaFunction,
}

impl Log {
    pub fn info(&self, s: &str) {
        self.info.call::<()>(s).unwrap();
    }

    pub fn warning(&self, s: &str) {
        self.warning.call::<()>(s).unwrap();
    }

    pub fn error(&self, s: &str) {
        self.error.call::<()>(s).unwrap();
    }
}

impl FromLua for Log {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let log = value.as_table().ok_or(LuaError::RuntimeError("".into()))?;
        Ok(Self {
            info: log.get("info")?,
            warning: log.get("warning")?,
            error: log.get("error")?,
        })
    }
}
