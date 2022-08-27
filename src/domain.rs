use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use whoami;

use crate::sal_info::SalInfo;

pub struct Domain {
    sal_info_set: RefCell<Vec<Weak<SalInfo>>>,
}

impl Domain {
    /// Create a new instance of Domain,
    pub fn new() -> Domain {
        Domain {
            sal_info_set: RefCell::new(Vec::new()),
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
