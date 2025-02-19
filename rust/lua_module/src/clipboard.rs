use std::sync::{Arc, Mutex, OnceLock};

use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
};

static STATE: OnceLock<Arc<Mutex<State>>> = OnceLock::new();

#[derive(Default)]
struct State {
    refcount: i32,
    data: Vec<String>,
    shutdown: Option<clipboard_rs::WatcherShutdown>,
}

struct Manager {
    ctx: ClipboardContext,
}

impl Manager {
    pub fn new() -> Self {
        let ctx = ClipboardContext::new().unwrap();
        Self { ctx }
    }
}

impl ClipboardHandler for Manager {
    fn on_clipboard_change(&mut self) {
        if let Ok(s) = self.ctx.get_text() {
            let mut state = STATE.get().unwrap().lock().unwrap();
            if let Some(idx) = state.data.iter().position(|i| i == &s) {
                state.data.remove(idx);
            }
            state.data.insert(0, s);
            state.data.shrink_to(5);
        }
    }
}

pub fn init() {
    let mut state = STATE.get_or_init(|| Default::default()).lock().unwrap();
    if state.refcount == 0 {
        std::thread::spawn(|| {
            let mut watcher = ClipboardWatcherContext::new().unwrap();
            let shutdown = watcher.add_handler(Manager::new()).get_shutdown_channel();
            STATE.get().unwrap().lock().unwrap().shutdown = Some(shutdown);
            watcher.start_watch();
        });
    }
    state.refcount += 1;
}

pub fn fini() {
    let mut state = STATE.get().unwrap().lock().unwrap();
    if state.refcount == 0 {
        unreachable!();
    }
    state.refcount -= 1;
    if state.refcount == 0 {
        state.shutdown.take().unwrap().stop();
        state.data.clear();
    }
}

pub fn get() -> Vec<String> {
    STATE.get().unwrap().lock().unwrap().data.clone()
}
