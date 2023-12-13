use clap::Parser;
use salobj::{csc::test_csc::test_csc::TestCSC, utils::cli::LogLevel};
use simple_logger::SimpleLogger;

/// Put the CSC in a particular state.
#[derive(Parser)]
struct Cli {
    /// Component index
    #[clap(short = 'i', long = "index", default_value = "0")]
    index: isize,

    #[arg(value_enum, long = "log-level", default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

impl Cli {
    pub fn get_component_index(&self) -> isize {
        self.index
    }

    pub fn get_log_level(&self) -> &LogLevel {
        &self.log_level
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let cli = Cli::parse();

    match cli.get_log_level() {
        LogLevel::Trace => log::set_max_level(log::LevelFilter::Trace),
        LogLevel::Debug => log::set_max_level(log::LevelFilter::Debug),
        LogLevel::Info => log::set_max_level(log::LevelFilter::Info),
        LogLevel::Warn => log::set_max_level(log::LevelFilter::Warn),
        LogLevel::Error => log::set_max_level(log::LevelFilter::Error),
    };

    log::info!("Running Test CSC with index {}.", cli.get_component_index(),);

    let mut test_csc = TestCSC::new(cli.get_component_index()).unwrap();

    log::info!("Starting CSC.");
    test_csc.start().await;

    log::info!("Running CSC.");
    let _ = test_csc.run().await;

    log::info!("Done...");
}
