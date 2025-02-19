use mlua::prelude::*;
mod clipboard;

fn init_clipboard(_: &Lua, _: ()) -> LuaResult<()> {
    clipboard::init();
    Ok(())
}

fn fini_clipboard(_: &Lua, _: ()) -> LuaResult<()> {
    clipboard::fini();
    Ok(())
}

fn get_clipboard(_: &Lua, _: ()) -> LuaResult<Vec<String>> {
    Ok(clipboard::get())
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "C" {
    pub unsafe fn LoadLibraryA(filename: *const std::ffi::c_char);
}

#[mlua::lua_module]
fn lua_helper_wzv5(lua: &Lua) -> LuaResult<LuaTable> {
    // 增加引用计数，避免 dll 被卸载，因为用了全局变量，卸载会崩溃
    #[cfg(windows)]
    unsafe {
        LoadLibraryA(std::ffi::CString::new("lua_helper_wzv5.dll").unwrap().as_ptr());
    }
    let exports = lua.create_table()?;
    let clipboard_table = lua.create_table()?;
    clipboard_table.set("init", lua.create_function(init_clipboard)?)?;
    clipboard_table.set("fini", lua.create_function(fini_clipboard)?)?;
    clipboard_table.set("get", lua.create_function(get_clipboard)?)?;
    exports.set("clipboard", clipboard_table)?;
    Ok(exports)
}
