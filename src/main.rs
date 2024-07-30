use clap::Parser;
use serde::{de, Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use tabbycat::attributes::*;
use tabbycat::{AttrList, Edge, GraphBuilder, GraphType, Identity, StmtList, SubGraph};

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    dependencies: Vec<Dependency>,
    manifest_display_name: String,
    name: String,
    path: String,
    platforms: Vec<Platform>,
    products: Vec<Product>,
    swift_languages_versions: Vec<String>,
    targets: Vec<Target>,
    tools_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    identity: String,
    requirement: Requirement,
    #[serde(rename = "type")]
    dependency_type: DependencyType,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyType {
    #[serde(rename = "sourceControl")]
    SourceControl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Requirement {
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<Vec<Range>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    lower_bound: String,
    upper_bound: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    name: String,
    targets: Vec<String>,
    #[serde(rename = "type")]
    product_type: TypeClass,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeClass {
    #[serde(skip_serializing_if = "Option::is_none")]
    library: Option<Vec<Library>>,
    executable: Option<serde_json::Value>,
    #[serde(rename = "macro")]
    type_macro: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Library {
    Automatic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    #[serde(rename = "c99name")]
    c99_name: String,
    module_type: ModuleType,
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_memberships: Option<Vec<String>>,
    sources: Vec<String>,
    #[serde(rename = "type")]
    target_type: TargetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_dependencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_dependencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<Vec<Resource>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleType {
    #[serde(rename = "ClangTarget")]
    ClangTarget,
    #[serde(rename = "SwiftTarget")]
    SwiftTarget,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    path: String,
    rule: Rule,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    #[serde(skip_serializing_if = "Option::is_none")]
    copy: Option<Copy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    process: Option<Copy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Copy {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Executable,
    Library,
    Macro,
    Test,
}

// MARK: -

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Directory containing the Swift package
    input: Option<PathBuf>,
    /// Output file, defaults to package name with .dot extension
    output: Option<PathBuf>,

    #[clap(long)]
    /// Skip unit test targets
    skip_test_targets: bool,

    #[clap(long)]
    /// Skip external product dependencies
    skip_product_dependencies: bool,
}

fn main() {
    let cli = Cli::parse();

    let output = Command::new("swift")
        .args(["package", "describe", "--type", "json"])
        .current_dir(cli.input.unwrap())
        .output()
        .expect("failed to execute process");

    let package: Package = serde_json::from_slice(&output.stdout).unwrap();

    let mut statements = StmtList::new();

    for target in package.targets {
        if cli.skip_test_targets && target.target_type == TargetType::Test {
            continue;
        }

        statements = statements.add_node(
            Identity::id(&target.name).unwrap(),
            None,
            Some(
                AttrList::new()
                    .add_pair(color(Color::Black))
                    .add_pair(shape(Shape::Box)),
            ),
        );

        for target_dependency in target.target_dependencies.unwrap_or_default() {
            statements = statements.add_node(
                Identity::id(&target_dependency).unwrap(),
                None,
                Some(
                    AttrList::new()
                        .add_pair(color(Color::Black))
                        .add_pair(shape(Shape::Box)),
                ),
            );

            statements = statements.add_edge(
                Edge::head_node(Identity::id(&target.name).unwrap(), None)
                    .arrow_to_node(Identity::id(&target_dependency).unwrap(), None),
            );
        }
        if !cli.skip_product_dependencies {
            for product_dependency in target.product_dependencies.unwrap_or_default() {
                statements = statements.add_node(
                    Identity::id(&product_dependency).unwrap(),
                    None,
                    Some(
                        AttrList::new()
                            .add_pair(color(Color::Blue))
                            .add_pair(shape(Shape::Box)),
                    ),
                );

                statements = statements.add_edge(
                    Edge::head_node(Identity::id(&target.name).unwrap(), None)
                        .arrow_to_node(Identity::id(&product_dependency).unwrap(), None),
                );
            }
        }
    }
    let graph = GraphBuilder::default()
        .graph_type(GraphType::DiGraph)
        .strict(false)
        .id(Identity::id(&package.name).unwrap())
        .stmts(statements)
        .build()
        .unwrap();

    let graph_string = graph.to_string();
    let graph_bytes = graph_string.as_bytes();

    let output_path = cli
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{}.dot", &package.name)));
    let output_extension = output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("dot");

    match output_extension {
        "dot" => {
            // write graph string to file
            let mut file = std::fs::File::create(&output_path).unwrap();
            file.write_all(graph_bytes).unwrap();
        }
        "svg" => {
            let mut dot = Command::new("dot")
                .args(["-Tsvg", "-o", output_path.to_str().unwrap()])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            dot.stdin.as_mut().unwrap().write_all(graph_bytes).unwrap();
        }
        "png" => {
            let mut dot = Command::new("dot")
                .args(["-Tpng", "-o", output_path.to_str().unwrap()])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            dot.stdin.as_mut().unwrap().write_all(graph_bytes).unwrap();
        }
        _ => {
            println!("Unknown output extension");
        }
    }
}
