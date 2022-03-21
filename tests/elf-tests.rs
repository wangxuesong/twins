use goblin::elf::*;
use std::path::Path;

#[test]
fn elf_class_test() {
    let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
    let fizz64_path = data_path.join("bin/fizz-buzz-glibc-64");
    let data = std::fs::read(fizz64_path).unwrap();
    let elf64 = Elf::parse(&*data).unwrap();
    assert_eq!(elf64.is_64, true);
}

#[test]
fn read_elf_linker_file_test() {
    let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
    let fizz64_path = data_path.join("bin/fizz-buzz-glibc-64");
    let data = std::fs::read(fizz64_path).unwrap();
    let elf64 = Elf::parse(&*data).unwrap();
    assert_eq!(elf64.is_64, true);
    assert_eq!(elf64.interpreter, Some("/lib64/ld-linux-x86-64.so.2"));
}

#[test]
fn find_direct_dependencies_test() {
    let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
    let fizz64_path = data_path.join("bin/fizz-buzz-glibc-64");
    let data = std::fs::read(fizz64_path).unwrap();
    let elf64 = Elf::parse(&*data).unwrap();
    assert_eq!(elf64.is_64, true);
    assert_eq!(elf64.libraries.len(), 1);
    assert_eq!(elf64.libraries[0], "libc.so.6");
    assert_eq!(elf64.runpaths.len(), 0);

    let server_path = data_path.join("bin/server");
    let data = std::fs::read(server_path).unwrap();
    let elf_server = Elf::parse(&*data).unwrap();
    assert_eq!(elf_server.is_64, true);
    assert_eq!(elf_server.interpreter, Some("/lib64/ld-linux-x86-64.so.2"));
    assert_eq!(elf_server.libraries.len(), 4);
    assert_eq!(elf_server.libraries[0], "libcraft.so");
    assert_eq!(elf_server.libraries[1], "libpthread.so.0");
    assert_eq!(elf_server.libraries[2], "libdl.so.2");
    assert_eq!(elf_server.libraries[3], "libc.so.6");
}
