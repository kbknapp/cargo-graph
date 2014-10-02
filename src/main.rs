#![feature(phase)]
extern crate cargo;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
extern crate graphviz;
extern crate serialize;

use cargo::core::resolver::Resolve;
use cargo::core::source::SourceId;
use docopt::{Error, FlagParser};
use graphviz as dot;
use std::io::File;
use std::str;

docopt!(Flags, "
Generate a graph of package dependencies in graphviz format

Usage: cargo dot [options]
       cargo dot --help

Options:
    -h, --help         Show this message
    -V, --version      Print version info and exit
    --lock-file=FILE   Specify location of input file, default \"Cargo.lock\"
    --dot-file=FILE    Output to file, default prints to stdout
")

fn main() {
    let config = docopt::Config { version: Some("0.1.0".to_string()) , ..docopt::DEFAULT_CONFIG };
    let flags: Flags = FlagParser::parse_conf(config).unwrap_or_else(|e| e.exit());
    let lock_file  = unless_empty(flags.flag_lock_file, "Cargo.lock");
    let dot_f_flag = if flags.flag_dot_file.is_empty() { None } else { Some(flags.flag_dot_file) };

    // TODO: figure out how to get rid of this.
    let dummy_src_id = SourceId::from_url("git+https://github.com/doopdoop/dodoododo#b3a9dee814af4846267383c800999a42b295e0d2".to_string());
    let resolved = cargo::ops::load_lockfile(&Path::new(lock_file), &dummy_src_id)
                     .unwrap_or_else(|e| exit_with(e.description().as_slice()))
                     .unwrap_or_else(||  exit_with("Lock file not found."));

    let graph = Graph::with_root(resolved.root().get_name());
    graph.add_dependencies(&resolved);

    match dot_f_flag {
        None           => graph.render_to(&mut std::io::stdio::stdout()),
        Some(dot_file) => graph.render_to(&mut File::create(&Path::new(dot_file)))
    };

}

fn exit_with(s: &str) -> ! {
    fail!("Error parsing lockfile: {}", s)
}

fn unless_empty(s: String, default: &str) -> String {
    if s.is_empty() {
        default.to_string()
    } else {
        s
    }
}

pub type Nd = uint;
pub type Ed = (uint, uint);
pub struct Graph<'a> {
    nodes: Vec<&'a str>,
    edges: Vec<Ed>,
}

impl<'a> Graph<'a> {
    pub fn with_root(root: &str) -> Graph {
        Graph { nodes: vec![root], edges: vec![] }
    }

    pub fn add_dependencies(&mut self, resolved: &'a Resolve) {
        for crat in resolved.iter() {
            match resolved.deps(crat) {
                Some(mut crate_deps) => {
                    let name = crat.get_name();
                    let idl = self.find_or_add(name);
                    for dep in crate_deps {
                        let dep_name = dep.get_name();
                        let idr = self.find_or_add(dep_name);
                        self.edges.push((idl, idr));
                    };
                },
                None => { }
            }
        }
    }

    fn find_or_add(&mut self, new: &'a str) -> uint {
        for (i, s) in self.nodes.iter().enumerate() {
            if s.as_slice() == new {
                return i
            }
        }
        self.nodes.push(new);
        self.nodes.len() - 1
    }

    pub fn render_to<W:Writer>(&'a self, output: &mut W) {
        dot::render(self, output).unwrap()
    }
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Graph<'a> {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example3")
    }
    fn node_id(&'a self, n: &Nd) -> dot::Id {
        dot::Id::new(format!("N{:u}", *n))
    }
    fn node_label<'a>(&'a self, i: &Nd) -> dot::LabelText<'a> {
        dot::LabelStr(str::Slice(self.nodes[*i].as_slice()))
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph<'a> {
    fn nodes(&'a self) -> dot::Nodes<'a,Nd> {
        range(0, self.nodes.len()).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a,Ed> {
        dot::maybe_owned_vec::Borrowed(self.edges.as_slice())
    }
    fn source(&self, &(s, _): &Ed) -> Nd { s }
    fn target(&self, &(_, t): &Ed) -> Nd { t }
}
