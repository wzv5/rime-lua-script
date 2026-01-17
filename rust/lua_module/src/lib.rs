#![allow(unused_variables, dead_code)]

use std::sync::{Arc, LazyLock, RwLock};

use mlua::prelude::*;

#[macro_use]
extern crate log;

mod clipboard;
mod error;
mod logger;
mod rime;
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

fn new_suggest(_: &Lua, _: ()) -> LuaResult<suggest::Suggest> {
    suggest::Suggest::new()
}

fn pinyin_match(
    _: &Lua,
    (text, pinyin, lookup_fn): (String, String, LuaFunction),
) -> LuaResult<bool> {
    let lookup = |c: char| lookup_fn.call::<String>(c).unwrap();
    let text = text.chars().collect::<Vec<_>>();
    let pinyin = pinyin.split(" ").collect::<Vec<_>>();
    if text.len() != pinyin.len() {
        return Ok(false);
    }
    let mut chars = text.into_iter();
    for py in pinyin {
        let c = chars.next().unwrap();
        if lookup(c).split(" ").any(|i| i == py) == false {
            return Ok(false);
        }
    }
    return Ok(true);
}

struct Global {
    lua: WeakLua,
    rime: rime::RimeGlobal,
}

// 绕过 rust 的全局变量限制，其实是 mlua 的限制，但 mlua 不允许 module 开启 send 特性，
// 正常情况下 rime 不会出现多线程并发调用，所以姑且认为安全
unsafe impl Sync for Global {}
unsafe impl Send for Global {}

static GLOBAL: LazyLock<Arc<RwLock<Option<Global>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

// 绑定到当前 lua 模块上，用来接收 drop 回调，从而精准控制生命周期
struct DropNotifier;

impl Drop for DropNotifier {
    fn drop(&mut self) {
        *GLOBAL.write().unwrap() = None;
    }
}

impl LuaUserData for DropNotifier {}

#[mlua::lua_module]
fn lua_helper_wzv5(lua: &Lua) -> LuaResult<LuaTable> {
    *GLOBAL.write().unwrap() = Some(Global {
        lua: lua.weak(),
        rime: rime::RimeGlobal::new(lua).unwrap(),
    });
    logger::init();
    let exports = lua.create_table()?;
    exports.set("_drop_notifier", DropNotifier)?;
    let clipboard_table = lua.create_table()?;
    clipboard_table.set("init", lua.create_function(init_clipboard)?)?;
    clipboard_table.set("fini", lua.create_function(fini_clipboard)?)?;
    clipboard_table.set("get", lua.create_function(get_clipboard)?)?;
    exports.set("clipboard", clipboard_table)?;
    let suggest_table = lua.create_table()?;
    suggest_table.set("new", lua.create_function(new_suggest)?)?;
    exports.set("suggest", suggest_table)?;
    exports.set("pinyin_match", lua.create_function(pinyin_match)?)?;
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
