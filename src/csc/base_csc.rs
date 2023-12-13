//! Trait for CSCs.

use crate::{error::errors::SalObjResult, generics::start::Start, sal_enums::State};

pub trait BaseCSC {
    fn do_start(&mut self, data: Start) -> SalObjResult<()> {
        let new_state = self.get_current_state().start()?;

        self.configure(&data)?;

        self.set_summary_state(new_state);

        Ok(())
    }

    fn get_current_state(&self) -> State;

    fn set_summary_state(&mut self, new_state: State);

    fn configure(&mut self, data: &Start) -> SalObjResult<()>;
}
