use goblin::elf::*;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

#[test]
fn elf_class_test() {
    let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
    let fizz32_path = data_path.join("bin/fizz-buzz-glibc-64");
    let data = std::fs::read(fizz32_path).unwrap();
    let elf32 = Elf::parse(&*data).unwrap();
    assert_eq!(elf32.is_64, false);

    let fizz64_path = data_path.join("bin/fizz-buzz-glibc-64");
    let data = std::fs::read(fizz64_path).unwrap();
    let elf64 = Elf::parse(&*data).unwrap();
    assert_eq!(elf64.is_64, true);
}

#[test]
fn read_elf_linker_file_test() {
    let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
    let fizz32_path = data_path.join("bin/fizz-buzz-glibc-64");
    let data = std::fs::read(fizz32_path).unwrap();
    let elf32 = Elf::parse(&*data).unwrap();
    assert_eq!(elf32.is_64, false);
    let sections = elf32
        .program_headers
        .iter()
        .filter(|ph| ph.p_type == 3)
        .collect::<Vec<_>>();
    assert_eq!(sections.len(), 1);
    let header = sections[0];
    let file = std::fs::File::open(data_path.join("bin/fizz-buzz-glibc-64")).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = vec![0; header.p_filesz as usize];
    reader
        .seek(std::io::SeekFrom::Start(header.p_offset as u64))
        .unwrap();
    assert_eq!(reader.read(&mut buffer).unwrap(), buffer.len());
    assert_eq!(buffer[buffer.len() - 1], 0);
    let name = std::str::from_utf8(&buffer[0..buffer.len() - 1]).unwrap();
    assert_eq!(name, "/lib/ld-linux.so.2");
}

#[test]
fn find_direct_dependencies_test() {
    let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
    let fizz32_path = data_path.join("bin/fizz-buzz-glibc-64");
    let command_ldd = Command::new("ldd")
        .arg(fizz32_path.to_str().unwrap())
        .output()
        .unwrap();
    let stdout = String::from_utf8(command_ldd.stdout).unwrap();
    assert_eq!(stdout.contains("libc.so.6"), true);
}
