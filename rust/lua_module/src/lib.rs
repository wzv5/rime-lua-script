use mlua::prelude::*;
mod clipboard;
mod error;
mod suggest;

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

fn suggest(_: &Lua, (pinyin, providers): (Vec<String>, Vec<String>)) -> LuaResult<Vec<String>> {
    Ok(suggest::suggest(pinyin, providers))
}

#[mlua::lua_module]
fn lua_helper_wzv5(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    let clipboard_table = lua.create_table()?;
    clipboard_table.set("init", lua.create_function(init_clipboard)?)?;
    clipboard_table.set("fini", lua.create_function(fini_clipboard)?)?;
    clipboard_table.set("get", lua.create_function(get_clipboard)?)?;
    exports.set("clipboard", clipboard_table)?;
    let suggest_table = lua.create_table()?;
    suggest_table.set("suggest", lua.create_function(suggest)?)?;
    exports.set("suggest", suggest_table)?;
    Ok(exports)
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_instance: usize, reason: u32, _: usize) -> bool {
    // DLL_PROCESS_DETACH
    if reason == 0 {
        // clipboard_rs 库有问题，停止监听是异步的，会延迟至多 200 ms
        // 这里等待 300 ms，确保监听线程结束，才能安全卸载
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    true
}
