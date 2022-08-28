use std::{
    cell::RefCell,
    process,
    rc::{Rc, Weak},
};
use whoami;

use crate::sal_info::SalInfo;

pub struct Domain {
    sal_info_set: RefCell<Vec<Weak<SalInfo>>>,
    origin: u32,
    identity: Option<String>,
}

impl Domain {
    /// Create a new instance of Domain,
    pub fn new() -> Domain {
        Domain {
            sal_info_set: RefCell::new(Vec::new()),
            origin: process::id(),
            identity: None,
        }
    }

    pub fn new_rc() -> Rc<RefCell<Domain>> {
        Rc::new(RefCell::new(Domain::new()))
    }

    /// Return the default identify.
    pub fn get_default_identity(&self) -> String {
        let username = whoami::username();
        let hostname = whoami::hostname();
        format!("{username}@{hostname}")
    }

    pub fn get_origin(&self) -> u32 {
        self.origin
    }

    pub fn get_identity(&self) -> String {
        match &self.identity {
            Some(identity) => identity.to_owned(),
            None => self.get_default_identity(),
        }
    }

    /// Add the specified salinfo to the internal registry.
    ///
    /// The input pointer is downgraded to a week reference and added to a
    /// vector list.
    pub fn add_salinfo(&self, sal_info: &Rc<SalInfo>) {
        self.sal_info_set.borrow_mut().push(Rc::downgrade(sal_info));
    }

    /// Remove the specified salinfo to the internal registry.
    ///
    /// Objects are removed using `swap_remove`, which is O(1) but does not
    /// preserve ordering.
    pub fn remove_salinfo(&self, sal_info: &Rc<SalInfo>) -> bool {
        match self
            .sal_info_set
            .borrow()
            .iter()
            .position(|x| x.ptr_eq(&Rc::downgrade(sal_info)))
        {
            Some(index) => {
                self.sal_info_set.borrow_mut().swap_remove(index);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Domain;

    #[test]
    fn get_default_identity() {
        let domain = Domain::new();

        let default_identity = domain.get_default_identity();

        assert!(default_identity.contains("@"))
    }
}
