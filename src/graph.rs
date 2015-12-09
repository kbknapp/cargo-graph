use std::fmt;
use std::io::{self, Write};

use config::Config;
use dep::{Dep, DepKind};
use error::CliResult;

pub type Nd = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ed(Nd, Nd);

impl Ed {
    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> io::Result<()> {
        if let Some(dep) = dg.get(self.1) {
            match dep.kind {
                DepKind::Build => writeln!(w, "[label=\"\"{}];", dg.cfg.build_lines),
                DepKind::Dev => writeln!(w, "[label=\"\"{}];", dg.cfg.dev_lines),
                DepKind::Optional => writeln!(w, "[label=\"\"{}];", dg.cfg.optional_lines),
            }
        } else {
            writeln!(w, "[label=\"\"];")
        }
    }
}

impl fmt::Display for Ed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Ed(il, ir) = self;
        write!(f, "N{} -> N{}", il, ir)
    }
}

#[derive(Debug)]
pub struct DepGraph<'c, 'o>
    where 'o: 'c
{
    nodes: Vec<Dep>,
    pub edges: Vec<Ed>,
    cfg: &'c Config<'o>,
}

impl<'c, 'o> DepGraph<'c, 'o> {
    pub fn with_root(root: Dep, cfg: &'c Config<'o>) -> Self {
        DepGraph {
            nodes: vec![root],
            edges: vec![],
            cfg: cfg,
        }
    }

    pub fn add_child(&mut self, parent: usize, d: &str, k: Option<DepKind>) -> usize {
        let idr = self.find_or_add(d, k.unwrap_or(DepKind::Build));
        self.edges.push(Ed(parent, idr));
        idr
    }

    pub fn get(&self, id: usize) -> Option<&Dep> {
        if id < self.nodes.len() {
            return Some(&self.nodes[id]);
        }
        None
    }

    pub fn update_style(&mut self, name: &str, kind: DepKind) {
        if let Some(id) = self.find(name) {
            if let Some(dep) = self.get_mut(id) {
                dep.kind = kind;
            }
        }
    }

    pub fn remove(&mut self, name: &str) {
        if let Some(id) = self.find(name) {
            debugln!("Removing node: {} index {}", name, id);
            self.nodes.remove(id);
        }
    }

    pub fn remove_orphans(&mut self) {
        let len = self.nodes.len();
        loop {
            let mut to_rem = None;
            for (eid, &Ed(idl, idr)) in self.edges.iter().enumerate() {
                if idl > len || idr > len {
                    to_rem = Some(eid);
                }
            }
            if let Some(id) = to_rem {
                self.edges.remove(id);
                continue;
            }
            break;
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Dep> {
        if id < self.nodes.len() {
            return Some(&mut self.nodes[id]);
        }
        None
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        for (i, d) in self.nodes.iter().enumerate() {
            if d.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn find_or_add(&mut self, new: &str, k: DepKind) -> usize {
        for (i, d) in self.nodes.iter().enumerate() {
            if d.name == new {
                return i;
            }
        }
        self.nodes.push(Dep::with_kind(new.to_owned(), k));
        self.nodes.len() - 1
    }

    pub fn render_to<W: Write>(mut self, output: &mut W) -> CliResult<()> {
        self.remove_orphans();
        self.edges.sort();
        self.edges.dedup();
        try!(writeln!(output, "{}", "digraph dependencies {"));
        for (i, dep) in self.nodes.iter().enumerate() {
            try!(write!(output, "\tN{}", i));
            try!(dep.label(output, self.cfg));
        }
        for ed in &self.edges {
            try!(write!(output, "\t{}", ed));
            try!(ed.label(output, &self));
        }
        try!(writeln!(output, "{}", "}"));
        Ok(())
    }

}
