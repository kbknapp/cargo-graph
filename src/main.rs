#![feature(rustc_private, plugin, std_misc)]
#![plugin(docopt_macros)]

extern crate cargo;
extern crate docopt;
extern crate graphviz;
extern crate "rustc-serialize" as rustc_serialize;

use cargo::core::{Resolve, SourceId, PackageId};
use graphviz as dot;
use std::borrow::{Cow, IntoCow};
use std::env;
use std::io;
use std::io::Write;
use std::fs::File;
use std::path::{Path, PathBuf, AsPath};

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
    let mut argv: Vec<String> = env::args().collect();
    if argv.len() > 0 {
        argv[0] = "cargo".to_string();
    }
    let flags: Flags = Flags::docopt()
                             // cargo passes the exe name first, so we skip it
                             .argv(argv.into_iter())
                             .version(Some("0.2".to_string()))
                             .decode()
                             .unwrap_or_else(|e|
                                             e.exit());

    let dot_f_flag = if flags.flag_dot_file.is_empty() { None } else { Some(flags.flag_dot_file) };
    let source_labels = flags.flag_source_labels;

    let lock_path = unless_empty(flags.flag_lock_file, "Cargo.lock");
    let lock_path = Path::new(&lock_path);
    let lock_path_buf = absolutize(lock_path.to_path_buf());
    let lock_path = lock_path_buf.as_path();

    let proj_dir  = lock_path.parent().unwrap(); // TODO: check for None
    let src_id = SourceId::for_path(&proj_dir).unwrap();
    let resolved = cargo::ops::load_lockfile(&lock_path, &src_id).unwrap()
        .expect("Lock file not found.");

    let mut graph = Graph::with_root(resolved.root(), source_labels);
    graph.add_dependencies(&resolved);

    match dot_f_flag {
        None           => graph.render_to(&mut io::stdout()),
        Some(dot_file) => graph.render_to(&mut File::create(&Path::new(&dot_file)).unwrap())
    };
}

fn absolutize(pb: PathBuf) -> PathBuf {
    if pb.as_path().is_absolute() {
        pb
    } else {
        std::env::current_dir().unwrap().join(&pb.as_path()).clone()
    }
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

    pub fn render_to<W:Write>(&'a self, output: &mut W) {
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
