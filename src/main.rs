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
Usage: cargo dot [options]
       cargo dot --help

Options:
    -h, --help         Show this message
    -V, --version      Print version info and exit
    --lock-file=FILE   Specify location of input file, default \"Cargo.lock\"
    --dot-file=FILE    Specify location of output file, default \"Cargo.dot\"
")

fn main() {
    let config = docopt::Config { version: Some("0.1.0".to_string()) , ..docopt::DEFAULT_CONFIG };
    let flags: Flags = FlagParser::parse_conf(config).unwrap_or_else(|e| e.exit());
    let lock_file = unless_empty(flags.flag_lock_file, "Cargo.lock");
    let dot_file  = unless_empty(flags.flag_dot_file, "Cargo.dot");

    // TODO: figure out how to get rid of this.
    let dummy_src_id = SourceId::from_url("git+https://github.com/doopdoop/dodoododo#b3a9dee814af4846267383c800999a42b295e0d2".to_string());
    let resolved = cargo::ops::load_lockfile(&Path::new(lock_file), &dummy_src_id)
                     .unwrap_or_else(|e| exit_with(e.description().as_slice()))
                     .unwrap_or_else(||  exit_with("Lock file not found."));
    let root = resolved.root();
    let name = root.get_name();
    let mut nodes = vec!(name);
    let mut edges = vec!();
    add_deps(&mut nodes, &mut edges, &resolved);

    let graph = Graph { nodes: nodes, edges: edges };
    let mut f = File::create(&Path::new(dot_file));

    graph.render_to(&mut f);
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

fn add_deps<'a>(nodes: &mut Vec<&'a str>, edges: &mut Vec<(uint, uint)>, resolved: &'a Resolve) {
    for crat in resolved.iter() {
        match resolved.deps(crat) {
            Some(mut crate_deps) => {
                let name = crat.get_name();
                let idl = add_or_find(nodes, name);
                for dep in crate_deps {
                    let dep_name = dep.get_name();
                    let idr = add_or_find(nodes, dep_name);
                    edges.push((idl, idr));
                };
            },
            None => { }
        }
    }
}

fn add_or_find<'a>(nodes: &mut Vec<&'a str>, new: &'a str) -> uint {
    for (i, s) in nodes.iter().enumerate() {
        if s.as_slice() == new {
            return i
        }
    }
    nodes.push(new);
    nodes.len() - 1
}

type Nd = uint;
type Ed<'a> = &'a (uint, uint);
struct Graph<'a> {
    nodes: Vec<&'a str>,
    edges: Vec<(uint,uint)>,
}

impl<'a> Graph<'a> {
    fn render_to<W:Writer>(&'a self, output: &mut W) {
        dot::render(self, output).unwrap()
    }
}

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph<'a> {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example3") }
    fn node_id(&'a self, n: &Nd) -> dot::Id {
        dot::Id::new(format!("N{:u}", *n))
    }
    fn node_label<'a>(&'a self, i: &Nd) -> dot::LabelText<'a> {
        dot::LabelStr(str::Slice(self.nodes[*i].as_slice()))
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph<'a> {
    fn nodes(&'a self) -> dot::Nodes<'a,Nd> {
        range(0, self.nodes.len()).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a,Ed<'a>> {
        self.edges.iter().collect()
    }
    fn source(&self, e: &Ed) -> Nd { let &(s,_) = *e; s }
    fn target(&self, e: &Ed) -> Nd { let &(_,t) = *e; t }
}
