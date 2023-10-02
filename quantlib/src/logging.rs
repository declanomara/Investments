use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use std::error::Error;

pub fn configure_logger(logfile: &str) -> Result<(), Box<dyn Error>>{
    let log_pattern = "[{d(%Y-%m-%d %H:%M:%S)}][{l}] {m}{n}";
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build(logfile)?;

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder()
            .appender("logfile")
            .appender("stdout")
            .build(LevelFilter::Trace))?;

    log4rs::init_config(config)?;
    Ok(())
}