use log::{Record, LevelFilter, Metadata, SetLoggerError};
use std::sync::Once;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;
static INIT: Once = Once::new();

pub fn init() -> Result<(), SetLoggerError> {
    INIT.call_once(|| {
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(LevelFilter::Info);
    });
    Ok(())
}

pub fn builder() -> Builder {
    Builder
}

pub struct Builder;
impl Builder {
    pub fn is_test(self, _is_test: bool) -> Self { self }
    pub fn try_init(self) -> Result<(), SetLoggerError> { init() }
}
