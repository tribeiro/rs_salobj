use salobj::{
    domain,
    sal_info::SalInfo,
    topics::{read_topic::ReadTopic, write_topic::WriteTopic},
};
use std::{cell::RefCell, rc::Rc};

#[test]
fn test_add_reader() {
    let domain = Rc::new(RefCell::new(domain::Domain::new()));
    let sal_info = SalInfo::new(domain, "Test", 1);

    let read_topic = ReadTopic::new(Rc::clone(&sal_info), "Test_scalars", 0, 100);

    sal_info.add_reader(read_topic);

    assert!(sal_info.has_read_topic("Test_scalars"));
}

#[test]
fn test_add_writer() {
    let domain = Rc::new(RefCell::new(domain::Domain::new()));
    let sal_info = SalInfo::new(domain, "Test", 1);

    let write_topic = WriteTopic::new(Rc::clone(&sal_info), "Test_scalars");

    sal_info.add_writer(write_topic);

    assert!(sal_info.has_write_topic("Test_scalars"));
}
