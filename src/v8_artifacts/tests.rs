use std::fs::File;
use std::path::Path;
use crate::base::reader::FileReader;
use crate::v8_artifacts::vfs_builder::{build_vfs, VfsEntry};

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

#[cfg(test)]
mod integration {
    use super::*;

    fn strip_utf8_bom(data: &[u8]) -> &[u8] {
        if data.starts_with(UTF8_BOM) { &data[3..] } else { data }
    }

    fn parse_and_list(path_str: &str) {
        let path = Path::new(path_str);
        if !path.exists() { println!("Skipping {}", path_str); return; }
        let file = File::open(path).expect("open");
        let file_size = file.metadata().unwrap().len();
        let reader = FileReader::new(file).expect("reader");
        let rows_map = crate::v8_artifacts::container::read_all_rows(reader, file_size).expect("read_all_rows");

        println!("\n=== Files in {} ===", path_str);
        let mut count = 0;
        for (id, data) in &rows_map {
            println!("[{}] {} - {} bytes", count, id, data.len());
            let data = strip_utf8_bom(data);
            if let Ok(s) = std::str::from_utf8(data) {
                let mut in_q = false;
                let mut qs = 0;
                for (i, ch) in s.char_indices() {
                    if ch == '"' {
                        if in_q { let c = &s[qs..i]; if c.len() >= 3 && c.len() <= 50 { println!("    >> '{}'", c); } }
                        in_q = !in_q; if in_q { qs = i + 1; }
                    }
                }
            }
            count += 1;
        }
        println!("Total items: {}", count);
        assert!(count > 0);
    }

    fn build_test_vfs(path_str: &str) -> Vec<VfsEntry> {
        let path = Path::new(path_str);
        if !path.exists() { panic!("Not found: {}", path_str); }
        let file = File::open(path).expect("open");
        let file_size = file.metadata().unwrap().len();
        let reader = FileReader::new(file).expect("reader");
        let rows_map = crate::v8_artifacts::container::read_all_rows(reader, file_size).expect("read_all_rows");
        build_vfs(rows_map).expect("build_vfs failed")
    }

    fn find_entry<'a>(entries: &'a [VfsEntry], name: &str) -> Option<&'a VfsEntry> {
        entries.iter().find(|e| e.name() == name)
    }

    fn print_vfs(entries: &[VfsEntry], indent: usize) {
        for entry in entries {
            let pfx = " ".repeat(indent);
            match entry {
                VfsEntry::Dir { name, children } => {
                    println!("{}{}/", pfx, name);
                    print_vfs(children, indent + 2);
                }
                VfsEntry::File { name, data, is_protected } => {
                    let p = if *is_protected { " [P]" } else { "" };
                    println!("{}{} ({} B){}", pfx, name, data.len(), p);
                }
            }
        }
    }

    // --- Raw listing tests ---

    #[test] fn test_parse_simple_epf() { parse_and_list("tests/epf/simple.epf"); }
    #[test] fn test_parse_with_form_epf() { parse_and_list("tests/epf/with_form.epf"); }
    #[test] fn test_parse_simple_erf() { parse_and_list("tests/erf/simple.erf"); }
    #[test] fn test_parse_1cv8_cf() { parse_and_list("tests/cf/1Cv8.cf"); }
    #[test] fn test_parse_with_module_epf() { parse_and_list("tests/epf/with_module.epf"); }
    #[test] fn test_parse_protected_epf() { parse_and_list("tests/epf/protected.epf"); }
    #[test] fn test_parse_cfe() { parse_and_list("tests/cfe/Ext1.cfe"); }

    // --- VFS structure tests (EPF/ERF) ---

    #[test]
    fn test_vfs_simple_epf() {
        let vfs = build_test_vfs("tests/epf/simple.epf");
        println!("\n=== VFS: simple.epf ==="); print_vfs(&vfs, 0);
        assert!(find_entry(&vfs, "Forms").is_none(), "no Forms");
        assert!(find_entry(&vfs, "Templates").is_none(), "no Templates");
    }

    #[test]
    fn test_vfs_with_form_epf() {
        let vfs = build_test_vfs("tests/epf/with_form.epf");
        println!("\n=== VFS: with_form.epf ==="); print_vfs(&vfs, 0);
        let forms = find_entry(&vfs, "Forms").expect("Forms dir");
        let children = forms.children().unwrap();
        assert!(!children.is_empty(), "Forms not empty");
        for form in children {
            assert!(form.is_dir());
            assert!(form.find_child("Module.bsl").is_some());
            println!("  Form: '{}'", form.name());
        }
    }

    #[test]
    fn test_vfs_simple_erf() {
        let vfs = build_test_vfs("tests/erf/simple.erf");
        println!("\n=== VFS: simple.erf ==="); print_vfs(&vfs, 0);
        assert!(find_entry(&vfs, "Forms").is_some(), "Forms exists");
        assert!(find_entry(&vfs, "Templates").is_some(), "Templates exists");
    }

    #[test]
    fn test_vfs_protected_epf() {
        let vfs = build_test_vfs("tests/epf/protected.epf");
        println!("\n=== VFS: protected.epf ==="); print_vfs(&vfs, 0);
        assert!(find_entry(&vfs, "Forms").is_none());
        assert!(find_entry(&vfs, "Templates").is_none());
    }

    #[test]
    fn test_vfs_with_module_epf() {
        let vfs = build_test_vfs("tests/epf/with_module.epf");
        println!("\n=== VFS: with_module.epf ==="); print_vfs(&vfs, 0);
        assert!(find_entry(&vfs, "ObjectModule.bsl").is_some(), "ObjectModule.bsl");
    }

    // --- VFS structure tests (CF/CFE) ---

    #[test]
    fn test_vfs_cf() {
        let vfs = build_test_vfs("tests/cf/1Cv8.cf");
        println!("\n=== VFS: 1Cv8.cf ==="); print_vfs(&vfs, 0);
        // CF should have at least one top-level group directory
        assert!(!vfs.is_empty(), "CF VFS should not be empty");
        // At least one entry should be a directory (type group)
        assert!(vfs.iter().any(|e| e.is_dir()), "CF should have directories");
    }

    #[test]
    fn test_vfs_cfe() {
    }
}
