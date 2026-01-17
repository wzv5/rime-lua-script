use crate::GLOBAL;

struct RimeLogger;

impl log::Log for RimeLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let s = format!(
            "{}({}): {}\r\n",
            record.file().unwrap_or("<unknown>"),
            record.line().unwrap_or(0),
            record.args()
        );
        let g = GLOBAL.read().unwrap();
        if let Some(g) = g.as_ref() {
            match record.level() {
                log::Level::Error => g.rime.log.error(&s),
                log::Level::Warn => g.rime.log.warning(&s),
                _ => g.rime.log.info(&s),
            };
        }
    }

    fn flush(&self) {}
}

static LOGGER: RimeLogger = RimeLogger;

pub fn init() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}
