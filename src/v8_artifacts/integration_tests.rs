use std::fs::File;
use std::path::Path;
use crate::base::reader::FileReader;
use crate::v8_artifacts::cf::Container;

#[cfg(test)]
mod integration {
    use super::*;

    fn parse_and_list(path_str: &str) {
        let path = Path::new(path_str);
        if !path.exists() {
            println!("Skipping {}, file not found", path_str);
            return;
        }
        
        let file = File::open(path).expect(&format!("Failed to open {}", path_str));
        let reader = FileReader::new(file).expect("Failed to create reader");
        let mut container = Container::new(reader).expect("Failed to parse container");
        
        println!("Files in {}:", path_str);
        let mut count = 0;
        for row_res in container.rows() {
            let row = row_res.expect("Failed to read row");
            if count < 10 {
                println!("  - {} ({} bytes, packed: {})", row.id, row.data.len(), row.is_packed);
            }
            count += 1;
        }
        println!("Total items: {}", count);
        assert!(count > 0, "No files found in container");
    }

    #[test]
    fn test_parse_simple_epf() {
        parse_and_list("tests/epf/simple.epf");
    }

    #[test]
    fn test_parse_with_form_epf() {
        parse_and_list("tests/epf/with_form.epf");
    }

    #[test]
    fn test_parse_simple_erf() {
        parse_and_list("tests/erf/simple.erf");
    }

    #[test]
    fn test_parse_1cv8_cf() {
        parse_and_list("tests/cf/1Cv8.cf");
    }

    #[test]
    fn test_parse_with_module_epf() {
        parse_and_list("tests/epf/with_module.epf");
    }

    #[test]
    fn test_parse_protected_epf() {
        parse_and_list("tests/epf/protected.epf");
    }
}
