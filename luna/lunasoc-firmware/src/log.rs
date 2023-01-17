//! A simple logger for the `log` crate which can log to any object
//! implementing `Write`

#![allow(unused_imports, unused_mut, unused_variables)]

use crate::{hal, pac};

use log::{Level, LevelFilter, Metadata, Record};

use core::fmt::Write;

// - initialization -----------------------------------------------------------

static LOGGER: WriteLogger = WriteLogger {
    level: Level::Debug,
};

pub fn init(writer: hal::Serial) {
    unsafe {
        log::set_logger_racy(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Debug))
            .unwrap();
    }
}

// - implementation -----------------------------------------------------------

/// WriteLogger
pub struct WriteLogger
//<W>
//where
//    W: Write
{
    //pub writer: Option<W>,
    pub level: Level,
}

//static GLOBAL_LOGGER: Logger<hal::Serial> = Logger {
//    writer: None,
//    level: LevelFilter::Error
//};

impl log::Log for WriteLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        //let peripherals = unsafe { pac::Peripherals::steal() };
        //let mut serial = hal::Serial::new(peripherals.UART);
        let mut serial = unsafe { hal::Serial::summon() };
        writeln!(serial, "{} - {}", record.level(), record.args()).unwrap();
    }

    fn flush(&self) {}
}
