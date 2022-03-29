use clap::Parser;
use twins::DependencyAnalyzer;


#[derive(Parser)]
struct Cli {
    #[clap(parse(from_os_str))]
    file: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    let mut analyzer = DependencyAnalyzer::new();
    let tree = analyzer.analyze(args.file).unwrap();
    let root_id = tree.root_node_id().unwrap();
    let ids = tree.children_ids(root_id).unwrap();
    for id in ids {
        let node = tree.get(id).unwrap();
        let data = node.data();
        println!("{} => {}", data.name, data.real_path.as_ref().unwrap_or(&"not found".to_string()));
    }
}