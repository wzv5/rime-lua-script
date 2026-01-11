use mlua::{FromLua, Function, Lua, Value};

pub struct Rime<'a> {
    lua: &'a Lua,
}

impl<'a> Rime<'a> {
    pub fn new(lua: &'a Lua) -> Self {
        Self { lua }
    }

    pub fn log(&self) -> Log {
        self.lua.globals().get("log").unwrap()
    }
}

pub struct Log {
    info: Function,
    warning: Function,
    error: Function,
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
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let log = value
            .as_table()
            .ok_or(mlua::Error::RuntimeError("".into()))?;
        Ok(Self {
            info: log.get("info").unwrap(),
            warning: log.get("warning").unwrap(),
            error: log.get("error").unwrap(),
        })
    }
}
