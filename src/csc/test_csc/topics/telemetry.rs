use super::{arrays::Arrays, scalars::Scalars};

/// Define all the telemetry types for the Test CSC.

#[derive(Default, Clone)]
pub enum TestTelemetry {
    #[default]
    None,
    Scalars(Scalars),
    Arrays(Arrays),
}
