use salobj::{domain, sal_info::SalInfo, topics::read_topic::ReadTopic};
use std::{cell::RefCell, rc::Rc};

#[test]
fn test_add_reader() {
    let domain = Rc::new(RefCell::new(domain::Domain::new()));
    let sal_info = SalInfo::new(domain, "Test", 1);

    let read_topic = ReadTopic::new(Rc::clone(&sal_info), "Test_scalars", 0, 100);

    sal_info.add_reader(read_topic);

    assert!(sal_info.has_read_topic("Test_scalars"));
}
