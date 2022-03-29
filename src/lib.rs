use anyhow::Result;
use goblin::elf::Elf;
use id_tree::InsertBehavior::{AsRoot, UnderNode};
use id_tree::{Node, NodeId, Tree, TreeBuilder};
use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};

type DependencyTree = Tree<BinaryFile>;
type DependencyNode = Node<BinaryFile>;

pub struct BinaryFile {
    pub name: String,
    pub real_path: Option<String>,
    pub interpreter: Option<String>,
    pub is_root: bool,
    pub is_executable: bool,
}

impl BinaryFile {
    pub fn new(path: String) -> Result<BinaryFile> {
        let binary_file = BinaryFile {
            name: path,
            real_path: None,
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

impl fmt::Display for BinaryFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Debug for BinaryFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.real_path.as_ref().unwrap_or(&"None".to_string()),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct DependencyAnalyzer {
    default_ld_paths: Vec<String>,
    interpreter: String,
}

impl DependencyAnalyzer {
    pub fn new() -> DependencyAnalyzer {
        DependencyAnalyzer {
            default_ld_paths: vec![
                "/lib64/".to_string(),
                "/lib/".to_string(),
                "/lib/x86_64-linux-gnu/".to_string(),
                "/usr/lib64/".to_string(),
                "/usr/lib/".to_string(),
                "/usr/lib/x86_64-linux-gnu/".to_string(),
            ],
            interpreter: "".to_string(),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn analyze(&mut self, path: impl AsRef<Path>) -> Result<DependencyTree> {
        let p = PathBuf::from(path.as_ref());
        let data = std::fs::read(p)?;
        let elf = Elf::parse(&data)?;
        let root = BinaryFile {
            name: String::from(path.as_ref().to_str().unwrap()),
            real_path: None,
            interpreter: elf.interpreter.map(|i| i.to_string()),
            is_root: true,
            is_executable: elf.program_headers.iter().any(|head| head.is_executable()),
        };
        self.interpreter = root.interpreter.clone().unwrap_or_default();
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
        let dep_file = self.find_library(dep_path)?;
        let path = dep_file.real_path.clone();
        let node_id = tree.insert(DependencyNode::new(dep_file), UnderNode(root_node))?;

        // if path exsit, then parse it
        if let Some(real_path) = path {
            let data = std::fs::read(real_path)?;
            let elf = Elf::parse(&data)?;
            for dep in elf.libraries.iter() {
                let interpreter = Path::new(self.interpreter.as_str())
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                if interpreter == *dep {
                    continue;
                }
                self.parse_dependency(tree, &node_id, dep.to_string())?;
            }
        }
        Ok(node_id)
    }

    fn find_library(&self, dep_path: PathBuf) -> Result<BinaryFile> {
        for ld_path in self.default_ld_paths.iter() {
            let mut path = PathBuf::from(ld_path);
            path.push(dep_path.clone());
            if path.exists() {
                return Ok(BinaryFile {
                    name: path.file_name().unwrap().to_string_lossy().to_string(),
                    real_path: Some(path.to_string_lossy().to_string()),
                    interpreter: None,
                    is_root: false,
                    is_executable: false,
                });
            }
        }
        let dep_file = BinaryFile {
            name: dep_path.to_string_lossy().to_string(),
            real_path: None,
            interpreter: None,
            is_root: false,
            is_executable: false,
        };
        Ok(dep_file)
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
        let fizz_path = data_path.join("bin/fizz-buzz-glibc-64");
        let mut analyzer = DependencyAnalyzer::new();
        let tree = analyzer.analyze(fizz_path.clone()).unwrap();
        // let tree = parse_binary_file(server_path.to_str().unwrap()).unwrap();
        let root_id = tree.root_node_id().unwrap();
        let root = tree.get(root_id).unwrap().data();
        assert_eq!(root.name, fizz_path.to_str().unwrap().to_string());
        assert_eq!(
            root.interpreter,
            Some("/lib64/ld-linux-x86-64.so.2".to_string())
        );
        assert_eq!(tree.height(), 3);
        let children = tree.children_ids(root_id).unwrap();
        assert_eq!(children.clone().into_iter().count(), 1);
        let child_id = children.into_iter().next().unwrap();
        let file = tree.get(child_id).unwrap().data();
        assert_eq!(file.name, "/lib/x86_64-linux-gnu/libc.so.6".to_string());
        let children = tree.children_ids(child_id).unwrap();
        assert_eq!(children.clone().into_iter().count(), 1);
        let child_id = children.into_iter().next().unwrap();
        let file = tree.get(child_id).unwrap().data();
        assert_eq!(
            file.name,
            "/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2".to_string()
        );
    }

    #[test]
    fn parse_server() {
        let data_path = Path::new(std::env!("CARGO_MANIFEST_DIR")).join("tests/elfbin");
        let server_path = data_path.join("bin/server");
        let mut analyzer = DependencyAnalyzer::new();
        let tree = analyzer.analyze(server_path.clone()).unwrap();

        let root_id = tree.root_node_id().unwrap();
        let root = tree.get(root_id).unwrap().data();
        assert_eq!(root.name, server_path.to_str().unwrap().to_string());
        assert_eq!(
            root.interpreter,
            Some("/lib64/ld-linux-x86-64.so.2".to_string())
        );
        let children_ids = tree.children_ids(root_id).unwrap();
        let mut children_id_iter = children_ids;
        let mut child_id = children_id_iter.next().unwrap();
        let mut child = tree.get(child_id).unwrap().data();
        assert_eq!(child.name, "libcraft.so".to_string());
        assert_eq!(child.real_path, None);
        child_id = children_id_iter.next().unwrap();
        child = tree.get(child_id).unwrap().data();
        assert_eq!(child.name, "libpthread.so.0".to_string());
        assert_eq!(
            child.real_path,
            Some("/lib/x86_64-linux-gnu/libpthread.so.0".to_string())
        );
        child_id = children_id_iter.next().unwrap();
        child = tree.get(child_id).unwrap().data();
        assert_eq!(child.name, "libdl.so.2".to_string());
        assert_eq!(
            child.real_path,
            Some("/lib/x86_64-linux-gnu/libdl.so.2".to_string())
        );
        child_id = children_id_iter.next().unwrap();
        child = tree.get(child_id).unwrap().data();
        assert_eq!(child.name, "libc.so.6".to_string());
        assert_eq!(
            child.real_path,
            Some("/lib/x86_64-linux-gnu/libc.so.6".to_string())
        );
        // let mut s = String::new();
        // tree.write_formatted(&mut s).unwrap();
        // assert_eq!(s, "", "{}", s);
    }
}
