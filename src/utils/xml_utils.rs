use crate::error::errors::NoInterfaceFileError;
use std::env;
use std::fs;
use std::path::Path;

/// Read interface definition xml file into a string.
///
/// # Panics
///
/// If environment variable TS_XML_DIR does not exists.
///
/// If the directory pointed by TS_XML_DIR does not exists.
pub fn read_xml_interface(interface: &str) -> Result<String, NoInterfaceFileError> {
    match env::var("TS_XML_DIR") {
        Ok(val) => {
            let interfaces_dir = Path::new(&val);

            assert!(
                interfaces_dir.exists(),
                "{}",
                format!("Interface directory {val} does not exists.")
            );

            let sal_subsystems_path = interfaces_dir.join(interface);
            if sal_subsystems_path.exists() {
                return Ok(fs::read_to_string(sal_subsystems_path).unwrap());
            } else {
                return Err(NoInterfaceFileError::new(&format!(
                    "No interface file for {interface}"
                )));
            }
        }
        Err(_) => panic!("Required environment variable 'TS_XML_DIR' not set."),
    }
}

/// Utility method to unwrap the result of `read_xml_interface`.
///
/// This method will either return the string part of the result, filtered out
/// of entries that are undesired or an empty string in case of
/// NoInterfaceFileError.
pub fn unwrap_xml_interface(xml_interface: Result<String, NoInterfaceFileError>) -> String {
    match xml_interface {
        Ok(xml_interface) => xml_interface
            .lines()
            .filter(|line| !line.contains("?xml"))
            .filter(|line| !line.contains("Enumeration"))
            .map(|line| format!("{}\n", line.to_string()))
            .collect(),
        Err(_) => String::new(),
    }
}

/// Returns default sal index.
pub fn get_default_sal_index() -> i64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_xml_interface() {
        let interface = read_xml_interface("SALSubsystems.xml").unwrap();

        assert!(interface.len() > 0)
    }

    #[test]
    #[should_panic(expected = "No interface file for SALInexistent.xml")]
    fn test_read_inexistent_interface() {
        read_xml_interface("SALInexistent.xml").unwrap();
    }

    #[test]
    fn test_unwrap_xml_interface() {
        let interface = unwrap_xml_interface(read_xml_interface("SALSubsystems.xml"));

        assert!(interface.len() > 0)
    }

    #[test]
    fn test_unwrap_inexistent_interface() {
        let interface = unwrap_xml_interface(read_xml_interface("SALInexistent.xml"));

        assert_eq!(interface.len(), 0)
    }
}
