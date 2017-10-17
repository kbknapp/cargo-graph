use std::fmt;
use std::io::{self, Write};

use config::Config;
use dep::ResolvedDep;
use error::CliResult;

pub type Nd = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Ed(pub Nd, pub Nd);

impl Ed {
    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> io::Result<()> {
        use dep::DepKind::{Optional, Dev, Build};
        let parent = dg.get(self.0).unwrap().kind();
        let child = dg.get(self.1).unwrap().kind();

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
    pub nodes: Vec<ResolvedDep>,
    pub edges: Vec<Ed>,
    cfg: &'c Config<'o>,
}

impl<'c, 'o> DepGraph<'c, 'o> {
    pub fn new(cfg: &'c Config<'o>) -> Self {
        DepGraph {
            nodes: vec![],
            edges: vec![],
            cfg: cfg,
        }
    }

    pub fn add_child(&mut self, parent: usize, dep_name: &str, dep_ver: &str) -> usize {
        let idr = self.find_or_add(dep_name, dep_ver);
        self.edges.push(Ed(parent, idr));
        idr
    }

    pub fn get(&self, id: usize) -> Option<&ResolvedDep> {
        if id < self.nodes.len() {
            return Some(&self.nodes[id]);
        }
        None
    }

    pub fn remove(&mut self, id: usize) {
        debugln!("remove; index={}", id);
        self.nodes.remove(id);
        // Remove edges of the removed node.
        self.edges = self.edges.iter()
            .filter(|e| !(e.0 == id || e.1 == id))
            .cloned()
            .collect();
        self.shift_edges_after_node(id);
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

    pub fn find_or_add(&mut self, name: &str, ver: &str) -> usize {
        for (i, d) in self.nodes.iter().enumerate() {
            if d.name == name && d.ver == ver {
                return i;
            }
        }
        self.nodes.push(ResolvedDep::new(name.to_owned(), ver.to_owned()));
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
