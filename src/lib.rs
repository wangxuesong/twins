use anyhow::Result;
use goblin::elf::Elf;
use id_tree::InsertBehavior::{AsRoot, UnderNode};
use id_tree::{Node, NodeId, Tree, TreeBuilder};
use std::path::{Path, PathBuf};

type DependencyTree = Tree<BinaryFile>;
type DependencyNode = Node<BinaryFile>;

#[derive(Debug)]
pub struct BinaryFile {
    pub path: String,
    pub interpreter: Option<String>,
    pub is_root: bool,
    pub is_executable: bool,
}

impl BinaryFile {
    pub fn new(path: String) -> Result<BinaryFile> {
        let binary_file = BinaryFile {
            path,
            interpreter: None,
            is_root: false,
            is_executable: false,
        };
        Ok(binary_file)
    }

    pub fn set_root(&mut self) {
        self.is_root = true;
    }
}

#[derive(Debug, Clone, Default)]
pub struct DependencyAnalyzer {}

impl DependencyAnalyzer {
    pub fn new() -> DependencyAnalyzer {
        DependencyAnalyzer {}
    }

    #[cfg(target_os="linux")]
    pub fn analyze(&self, path: impl AsRef<Path>) -> Result<DependencyTree> {
        let p = PathBuf::from(path.as_ref());
        let data = std::fs::read(p)?;
        let elf = Elf::parse(&data)?;
        let root = BinaryFile {
            path: String::from(path.as_ref().to_str().unwrap()),
            interpreter: elf.interpreter.map(|i| i.to_string()),
            is_root: true,
            is_executable: elf.program_headers.iter().any(|head| head.is_executable()),
        };
        let mut tree = TreeBuilder::new().build();

        let root_node = tree.insert(DependencyNode::new(root), AsRoot)?;

        for dep in elf.libraries.iter() {
            self.parse_dependency(&mut tree, &root_node, dep.to_string())?;
        }
        Ok(tree)
    }

    fn parse_dependency(
        &self,
        tree: &mut Tree<BinaryFile>,
        root_node: &NodeId,
        name: String,
    ) -> Result<NodeId> {
        let dep_path = PathBuf::from(name);
        let dep_file = BinaryFile {
            path: dep_path.to_string_lossy().to_string(),
            interpreter: None,
            is_root: false,
            is_executable: false,
        };
        let node_id = tree.insert(DependencyNode::new(dep_file), UnderNode(root_node))?;

        // let data = std::fs::read(dep_path.clone())?;
        // let elf = Elf::parse(&data)?;
        // for dep in elf.libraries.iter() {
        //     self.parse_dependency(tree, &node_id, dep.to_string())?;
        // }
        Ok(node_id)
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_fizz() {
        let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
        let server_path = data_path.join("bin/fizz-buzz-glibc-64");
        let analyzer = DependencyAnalyzer::new();
        let tree = analyzer.analyze(server_path.clone()).unwrap();
        // let tree = parse_binary_file(server_path.to_str().unwrap()).unwrap();
        let root_id = tree.root_node_id().unwrap();
        let root = tree.get(root_id).unwrap().data();
        assert_eq!(root.path, server_path.to_str().unwrap().to_string());
        assert_eq!(
            root.interpreter,
            Some("/lib64/ld-linux-x86-64.so.2".to_string())
        );
        assert_eq!(tree.height(), 2);
        let children = tree.children(root_id).unwrap();
        assert_eq!(children.clone().into_iter().count(), 1);
        let child = children.into_iter().next().unwrap().data();
        assert_eq!(child.path, "libc.so.6".to_string());
    }
}
