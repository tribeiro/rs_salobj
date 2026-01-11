//! Utilities for command line interfaces.

/// Gerenal purpose LogLevel struct that can be used with clap.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
