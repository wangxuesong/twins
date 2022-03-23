use anyhow::Result;
use goblin::elf::Elf;
use id_tree::InsertBehavior::AsRoot;
use id_tree::{Node, Tree, TreeBuilder};

type DependencyTree = Tree<BinaryFile>;
type DependencyNode = Node<BinaryFile>;

#[derive(Debug)]
pub struct BinaryFile {
    pub path: String,
    pub is_root: bool,
    pub is_executable: bool,
}

impl BinaryFile {
    pub fn new(path: String) -> Result<BinaryFile> {
        let data = std::fs::read(path.clone())?;
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
            path,
            is_root: false,
            is_executable,
        };
        Ok(binary_file)
    }

    pub fn set_root(&mut self) {
        self.is_root = true;
    }
}

pub fn parse_binary_file(path: &str) -> Result<DependencyTree> {
    let mut tree = TreeBuilder::new().with_node_capacity(5).build();
    let root = BinaryFile {
        path: path.to_string(),
        is_root: true,
        is_executable: false,
    };
    tree.insert(DependencyNode::new(root), AsRoot)?;
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_server() {
        let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
        let server_path = data_path.join("bin/server");
        let tree = parse_binary_file(server_path.to_str().unwrap()).unwrap();
        let root_id = tree.root_node_id().unwrap();
        let root = tree.get(root_id).unwrap().data();
        assert_eq!(root.path, server_path.to_str().unwrap().to_string());
        assert_eq!(tree.height(), 1);
    }
}
