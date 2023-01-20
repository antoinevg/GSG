//! A simple logger for the `log` crate which can log to any object
//! implementing `Write`

#![allow(unused_imports, unused_mut, unused_variables)]

use crate::{hal, pac};

use log::{Level, LevelFilter, Metadata, Record};

use core::cell::RefCell;
use core::fmt::Write;

// - initialization -----------------------------------------------------------

static LOGGER: WriteLogger<hal::Serial> = WriteLogger {
    writer: RefCell::new(None),
    level: Level::Trace,
};

pub fn init(writer: hal::Serial) {
    LOGGER.writer.replace(Some(writer));

    // TODO we need support for atomics to use log::set_logger()
    unsafe { log::set_logger_racy(&LOGGER) }
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap();
}

// - implementation -----------------------------------------------------------

/// WriteLogger
pub struct WriteLogger<W>
where
    W: Write + Send,
{
    pub writer: RefCell<Option<W>>,
    pub level: Level,
}

impl<W> log::Log for WriteLogger<W>
where
    W: Write + Send,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        riscv::interrupt::free(|| match self.writer.borrow_mut().as_mut() {
            Some(writer) => {
                writeln!(writer, "{} - {}", record.level(), record.args())
                    .expect("Logger failed to write to device");
                //unsafe { riscv::asm::delay(6_000_000) };
            }
            None => {
                panic!("Logger has not been initialized");
            }
        })
    }

    fn flush(&self) {}
}

// TODO add support for critical-section crate
// TODO implement a riscv::interrupt::Mutex
unsafe impl<W: Write + Send> Sync for WriteLogger<W> {}
