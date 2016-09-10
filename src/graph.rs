use std::fmt;
use std::io::{self, Write};

use config::Config;
use dep::{Dep, DepKind};
use error::CliResult;

pub type Nd = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Ed(Nd, Nd);

impl Ed {
    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> io::Result<()> {
        use dep::DepKind::{Optional, Dev, Build};
        let parent = dg.get(self.0).unwrap().kind;
        let child = dg.get(self.1).unwrap().kind;

        match (parent, child) {
            (Build, Build) => writeln!(w, "[label=\"\"{}];", dg.cfg.build_lines),
            (Build, Dev) => writeln!(w, "[label=\"\"{}];", dg.cfg.dev_lines),
            (Build, Optional) => writeln!(w, "[label=\"\"{}];", dg.cfg.optional_lines),
            (Optional, Build) => writeln!(w, "[label=\"\"{}];", dg.cfg.optional_lines),
            (Optional, Dev) => writeln!(w, "[label=\"\"{}];", dg.cfg.optional_lines),
            (Optional, Optional) => writeln!(w, "[label=\"\"{}];", dg.cfg.optional_lines),
            (Dev, Build) => writeln!(w, "[label=\"\"{}];", dg.cfg.dev_lines),
            (Dev, Dev) => writeln!(w, "[label=\"\"{}];", dg.cfg.dev_lines),
            (Dev, Optional) => writeln!(w, "[label=\"\"{}];", dg.cfg.dev_lines),
            _               => writeln!(w, "[label=\"\"];")
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
    pub nodes: Vec<Dep>,
    pub edges: Vec<Ed>,
    pub root: String,
    cfg: &'c Config<'o>,
}

impl<'c, 'o> DepGraph<'c, 'o> {
    pub fn with_root(root: Dep, cfg: &'c Config<'o>) -> Self {
        DepGraph {
            root: root.name.clone(),
            nodes: vec![root],
            edges: vec![],
            cfg: cfg,
        }
    }

    pub fn add_child(&mut self, parent: usize, d: &str, k: Option<DepKind>) -> usize {
        let idr = self.find_or_add(d, k.unwrap_or(DepKind::Unk));
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

    pub fn update_ver<S: Into<String>>(&mut self, name: &str, ver: S) {
        if let Some(id) = self.find(name) {
            if let Some(dep) = self.get_mut(id) {
                dep.ver(ver);
            }
        }
    }

    pub fn remove(&mut self, name: &str) {
        if let Some(id) = self.find(name) {
            debugln!("remove; name={}; index={}", name, id);
            self.nodes.remove(id);
            // Remove edges of the removed node.
            self.edges = self.edges.iter()
                .filter(|e| !(e.0 == id || e.1 == id))
                .map(|&e| e)
                .collect();
            self.shift_edges_after_node(id);
        }
    }

    fn shift_edges_after_node(&mut self, id: usize) {
        enum Side {
            Left,
            Right,
        }
        let mut to_upd = vec![];
        for c in id..self.nodes.len() {
            for (eid, &Ed(idl, idr)) in self.edges.iter().enumerate() {
                if idl == c { to_upd.push((eid, Side::Left, c-1)); }
                if idr == c { to_upd.push((eid, Side::Right, c-1)); }
            }
        }
        for (eid, side, new) in to_upd {
            match side {
                Side::Left => self.edges[eid].0 = new,
                Side::Right => self.edges[eid].1 = new,
            }
        }
    }

    pub fn remove_orphans(&mut self) {
        let len = self.nodes.len();
        self.edges.retain(|&Ed(idl,idr)| idl < len && idr < len);
        debugln!("remove_orphans; nodes={:?}", self.nodes);
        loop {
            let mut removed = false;
            let mut used = vec![false; self.nodes.len()];
            used[0] = true;
            for &Ed(_, idr) in &self.edges {
                debugln!("remove_orphans; idr={}", idr);
                used[idr] = true;
            }
            debugln!("remove_orphans; unsued_nodes={:?}", used);

            for (id, &u) in used.iter().enumerate() {
                if !u {
                    debugln!("remove_orphans; removing={}", id);
                    self.nodes.remove(id);

                    // Remove edges originating from the removed node
                    self.edges.retain(|&Ed(origin,_)| origin != id);
                    // Adjust edges to match the new node indexes
                    for edge in self.edges.iter_mut() {
                        if edge.0 > id {
                            edge.0 -= 1;
                        }
                        if edge.1 > id {
                            edge.1 -= 1;
                        }
                    }
                    removed = true;
                    break;
                }
            }
            if !removed {
                break;
            }
        }
    }

    fn remove_self_pointing(&mut self) {
        loop {
            let mut found = false;
            let mut self_p = vec![false; self.edges.len()];
            for (eid ,&Ed(idl, idr)) in self.edges.iter().enumerate() {
                if idl == idr {
                    found = true;
                    self_p[eid] = true;
                    break;
                }
            }
            debugln!("remove_self_pointing; self_pointing={:?}", self_p);

            for (id, &u) in self_p.iter().enumerate() {
                if u {
                    debugln!("remove_self_pointing; removing={}", id);
                    self.edges.remove(id);
                    break;
                }
            }
            if !found {
                break;
            }
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
        debugln!("exec=render_to;");
        self.edges.sort();
        self.edges.dedup();
        self.remove_orphans();
        self.remove_self_pointing();
        debugln!("dg={:#?}", self);
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
