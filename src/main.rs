extern crate toml;
extern crate graphviz;

use graphviz as dot;
use std::io::File;
use std::str;

fn main() {
    let (name, indirect_deps) = read_cargo_lock("Cargo.lock");
    let direct_deps           = read_cargo_toml("Cargo.toml");
    let mut nodes    = vec!(name).append(direct_deps.as_slice());
    let mut edges    = range(1, nodes.len()).map(|n| (0, n)).collect();
    
    add_deps(&mut nodes, &mut edges, indirect_deps);
    
    let graph = Graph { nodes: nodes, edges: edges };
    let mut f    = File::create(&Path::new("Cargo.dot"));
    
    graph.render_to(&mut f);
}

fn read_cargo_toml(toml_file_name: &str) -> (Vec<String>) {
    let toml_str = File::open(&Path::new(toml_file_name)).read_to_string().unwrap();
    let toml     = toml::Parser::new(toml_str.as_slice()).parse().unwrap();
    let deps: Vec<String> =
        toml.find(&"dependencies".to_string()).unwrap().as_table().unwrap().keys().map(|s| s.to_string()).collect();
    deps
}

fn read_cargo_lock(_lock_file_name: &str) -> (String, Vec<(&str, Vec<&str>)>) {
    ("cargo".to_string(), vec!())
}

fn add_deps(nodes: &mut Vec<String>, edges: &mut Vec<(uint, uint)>, deps: Vec<(&str, Vec<&str>)>) {
    for (crat, crate_deps) in deps.move_iter() {
        let idl = add_or_find(nodes, crat);
        for dep in crate_deps.move_iter() {
            let idr = add_or_find(nodes, dep);
            edges.push((idl, idr));
        }
    }
}

fn add_or_find<'a>(nodes: &mut Vec<String>, new: &'a str) -> uint {
    for i in range(0, nodes.len()) {
        let ref s = (*nodes)[i];
        if s.as_slice() == new {
            return i
        }
    }
    nodes.push(new.to_string());
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
