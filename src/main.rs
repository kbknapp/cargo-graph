#![feature(rustc_private, plugin, old_io, env, old_path)]
#![plugin(docopt_macros)]

extern crate cargo;
extern crate docopt;
extern crate graphviz;
extern crate "rustc-serialize" as rustc_serialize;

use cargo::core::{Resolve, SourceId, PackageId};
use graphviz as dot;
use std::borrow::{Cow, IntoCow};
use std::env;
use std::old_io as io;
use std::old_io::fs::File;
use std::old_path::Path;

docopt!(Flags, "
Generate a graph of package dependencies in graphviz format

Usage: cargo dot [options]
       cargo dot --help

Options:
    -h, --help         Show this message
    -V, --version      Print version info and exit
    --lock-file=FILE   Specify location of input file, default \"Cargo.lock\"
    --dot-file=FILE    Output to file, default prints to stdout
    --source-labels    Use sources for the label instead of package names
");

fn main() {
    let flags: Flags = Flags::docopt()
                             // cargo passes the exe name first, so we skip it
                             .argv(env::args().skip(1))
                             .version(Some("0.2".to_string()))
                             .decode()
                             .unwrap_or_else(|e| e.exit());
    let lock_file  = unless_empty(flags.flag_lock_file, "Cargo.lock");
    let dot_f_flag = if flags.flag_dot_file.is_empty() { None } else { Some(flags.flag_dot_file) };
    let source_labels = flags.flag_source_labels;

    let lock_file = Path::new(&lock_file);
    let project_dir = lock_file.dir_path();
    let project_dir = std::env::current_dir().unwrap().join(&project_dir);
    let src_id = SourceId::for_path(&project_dir).unwrap();
    let resolved = cargo::ops::load_lockfile(&lock_file, &src_id).unwrap()
        .expect("Lock file not found.");

    let mut graph = Graph::with_root(resolved.root(), source_labels);
    graph.add_dependencies(&resolved);

    match dot_f_flag {
        None           => graph.render_to(&mut io::stdio::stdout()),
        Some(dot_file) => graph.render_to(&mut File::create(&Path::new(&dot_file)).unwrap())
    };

}

fn unless_empty(s: String, default: &str) -> String {
    if s.is_empty() {
        default.to_string()
    } else {
        s
    }
}

pub type Nd = usize;
pub type Ed = (usize, usize);
pub struct Graph<'a> {
    nodes: Vec<&'a PackageId>,
    edges: Vec<Ed>,
    source_labels: bool
}

impl<'a> Graph<'a> {
    pub fn with_root(root: &PackageId, source_labels: bool) -> Graph {
        Graph { nodes: vec![root], edges: vec![], source_labels: source_labels }
    }

    pub fn add_dependencies(&mut self, resolved: &'a Resolve) {
        for crat in resolved.iter() {
            match resolved.deps(crat) {
                Some(crate_deps) => {
                    let idl = self.find_or_add(crat);
                    for dep in crate_deps {
                        let idr = self.find_or_add(dep);
                        self.edges.push((idl, idr));
                    };
                },
                None => { }
            }
        }
    }

    fn find_or_add(&mut self, new: &'a PackageId) -> usize {
        for (i, id) in self.nodes.iter().enumerate() {
            if *id == new {
                return i
            }
        }
        self.nodes.push(new);
        self.nodes.len() - 1
    }

    pub fn render_to<W:Writer>(&'a self, output: &mut W) {
        match dot::render(self, output) {
            Ok(_) => {},
            Err(e) => panic!("error rendering graph: {}", e)
        }
    }
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Graph<'a> {
    fn graph_id(&self) -> dot::Id<'a> {
        dot::Id::new(self.nodes[0].name()).unwrap_or(dot::Id::new("dependencies").unwrap())
    }
    fn node_id(&self, n: &Nd) -> dot::Id {
        // unwrap is safe because N######## is a valid graphviz id
        dot::Id::new(format!("N{}", *n)).unwrap()
    }
    fn node_label(&'a self, i: &Nd) -> dot::LabelText<'a> {
        if !self.source_labels {
            dot::LabelText::LabelStr(self.nodes[*i].name().into_cow())
        } else {
            dot::LabelText::LabelStr(self.nodes[*i].source_id().url().to_string().into_cow())
        }
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph<'a> {
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        (0..self.nodes.len()).collect()
    }
    fn edges(&self) -> dot::Edges<Ed> {
        Cow::Borrowed(&self.edges[..])
    }
    fn source(&self, &(s, _): &Ed) -> Nd { s }
    fn target(&self, &(_, t): &Ed) -> Nd { t }
}
