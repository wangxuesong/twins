use anyhow::Result;
use goblin::elf::Elf;
use id_tree::{Node, Tree};


type DependencyTree = Tree<BinaryFile>;
type DependencyNode = Node<BinaryFile>;

#[derive(Debug)]
pub struct BinaryFile {
    pub name: String,
    pub is_root: bool,
    pub is_executable: bool,
}

impl BinaryFile {
    pub fn new(name: String) -> Result<BinaryFile> {
        let data = std::fs::read(name.clone())?;
        let elf = Elf::parse(&data)?;
        let mut is_executable = false;
        for header in elf.program_headers.iter() {
            if header.is_executable() {
                is_executable = true;
            }
        }
        let mut dependencies = vec![];
        for dep in elf.libraries.iter() {
            let dep_name = dep.to_string();
            let file = BinaryFile::new(dep_name)?;
            dependencies.push(file);
        }

        let binary_file = BinaryFile {
            name,
            is_root: false,
            is_executable,
        };
        Ok(binary_file)
    }

    pub fn set_root(&mut self) {
        self.is_root = true;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
