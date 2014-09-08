#![feature(phase)]
extern crate serialize;
#[phase(plugin)] extern crate docopt_macros;
extern crate docopt;
extern crate graphviz;
extern crate toml;

use docopt::{Error, FlagParser};
use graphviz as dot;
use std::io::File;
use std::str;

docopt!(Flags, "
Usage: cargo dot [options]
       cargo dot --help

Options:
    -h, --help         Show this message
    -V, --version      Print version info and exit
    --lock-file=FILE   Specify location of input file, default \"Cargo.lock\"
")

fn main() {
    let config = docopt::Config { version: Some("0.1.0".to_string()) , ..docopt::DEFAULT_CONFIG };
    let flags: Flags = FlagParser::parse_conf(config).unwrap_or_else(|e| e.exit());
    println!("{}", flags);
    let lock_file = if flags.flag_lock_file.is_empty() { "Cargo.lock".to_string() } else { flags.flag_lock_file };

    let (name, direct_deps, indirect_deps) = read_cargo_lock(lock_file.as_slice());
    let mut nodes = vec!(name).append(direct_deps.as_slice());
    let mut edges = range(1, nodes.len()).map(|n| (0, n)).collect();
    add_deps(&mut nodes, &mut edges, indirect_deps);
    
    let graph = Graph { nodes: nodes, edges: edges };
    let mut f    = File::create(&Path::new("Cargo.dot"));
    
    graph.render_to(&mut f);
}

fn read_cargo_lock(file_name: &str) -> (String, Vec<String>, Vec<(String, Vec<String>)>) {
    let toml = read_toml(file_name);
    let root = toml.find(&"root".to_string()).unwrap()
                   .as_table().unwrap();
    let name = root.find(&"name".to_string()).unwrap()
                   .as_str().unwrap().to_string();
    let direct_deps = extract_deps(root);
    let all_deps = toml.find(&"package".to_string()).unwrap()
                       .as_slice().unwrap().iter().map(other_deps).collect();
    (name, direct_deps, all_deps)
}

fn extract_deps(v: &toml::Table) -> Vec<String> {
    match v.find(&"dependencies".to_string()) {
        None     => vec!(),
        Some(ds) => ds.as_slice().unwrap().iter()
                      .map(|v| parse_dep(v.as_str().unwrap())).collect()
    }
}

fn other_deps(v: &toml::Value) -> (String, Vec<String>) {
    let t = v.as_table().unwrap();
    let name = t.find(&"name".to_string()).unwrap()
                .as_str().unwrap().to_string();
    let deps = extract_deps(t);
    (name, deps)
}

fn read_toml(file_name: &str) -> toml::Table {
    let toml_str = File::open(&Path::new(file_name)).read_to_string().unwrap();
    toml::Parser::new(toml_str.as_slice()).parse().unwrap()

}

fn parse_dep(s: &str) -> String {
    s.chars().take_while(|&a| a != ' ').collect()
}

fn add_deps(nodes: &mut Vec<String>, edges: &mut Vec<(uint, uint)>, deps: Vec<(String, Vec<String>)>) {
    for (crat, crate_deps) in deps.move_iter() {
        let idl = add_or_find(nodes, crat);
        for dep in crate_deps.move_iter() {
            let idr = add_or_find(nodes, dep);
            edges.push((idl, idr));
        }
    }
}

fn add_or_find(nodes: &mut Vec<String>, new: String) -> uint {
    for i in range(0, nodes.len()) {
        let ref s = (*nodes)[i];
        if *s == new {
            return i
        }
    }
    nodes.push(new);
    nodes.len() - 1
}

type Nd = uint;
type Ed<'a> = &'a (uint, uint);
struct Graph { nodes: Vec<String>,
               edges: Vec<(uint,uint)>,
             }

impl Graph {
    fn render_to<W:Writer>(self, output: &mut W) {
        dot::render(&self, output).unwrap()
    }
}

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example3") }
    fn node_id(&'a self, n: &Nd) -> dot::Id {
        dot::Id::new(format!("N{:u}", *n))
    }
    fn node_label<'a>(&'a self, i: &Nd) -> dot::LabelText<'a> {
        dot::LabelStr(str::Slice(self.nodes[*i].as_slice()))
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a,Nd> {
        range(0, self.nodes.len()).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a,Ed<'a>> {
        self.edges.iter().collect()
    }
    fn source(&self, e: &Ed) -> Nd { let &(s,_) = *e; s }
    fn target(&self, e: &Ed) -> Nd { let &(_,t) = *e; t }
}
