//! Implement methods basic state machine methods for [sal_enums::State].

use crate::{
    error::errors::{SalObjError, SalObjResult},
    sal_enums::State,
};

impl State {
    pub fn start(self) -> SalObjResult<State> {
        if self == State::Standby {
            Ok(State::Disabled)
        } else {
            Err(SalObjError::new(&format!(
                "Cannot perform state transition {self} -> Disable"
            )))
        }
    }

    pub fn enable(self) -> SalObjResult<State> {
        if self == State::Disabled {
            Ok(State::Enabled)
        } else {
            Err(SalObjError::new(&format!(
                "Cannot perform state transition {self} -> Enabled"
            )))
        }
    }

    pub fn disable(self) -> SalObjResult<State> {
        if self == State::Enabled {
            Ok(State::Disabled)
        } else {
            Err(SalObjError::new(&format!(
                "Cannot perform state transition {self} -> Disabled"
            )))
        }
    }

    pub fn standby(self) -> SalObjResult<State> {
        if self == State::Disabled || self == State::Fault {
            Ok(State::Standby)
        } else {
            Err(SalObjError::new(&format!(
                "Cannot perform state transition {self} -> Standby"
            )))
        }
    }

    pub fn enter_control(self) -> SalObjResult<State> {
        if self == State::Standby {
            Ok(State::Offline)
        } else {
            Err(SalObjError::new(&format!(
                "Cannot perform state transition {self} -> Offline"
            )))
        }
    }

    pub fn exit_control(self) -> SalObjResult<State> {
        if self == State::Offline {
            Ok(State::Standby)
        } else {
            Err(SalObjError::new(&format!(
                "Cannot perform state transition {self} -> Standby"
            )))
        }
    }
}
