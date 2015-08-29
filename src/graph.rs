use std::io::Write;

// use dot;

use config::Config;
use dep::{Dep, DepKind};
use error::{CliError, CliResult};

pub type Nd = usize;
pub type Ed = (usize, usize);
#[derive(Debug)]
pub struct DepGraph<'c, 'o> where 'o: 'c {
    nodes: Vec<Dep>,
    pub edges: Vec<Ed>,
    cfg: &'c Config<'o>
}

impl<'c, 'o> DepGraph<'c, 'o> {
    pub fn with_root(root: Dep, cfg: &'c Config<'o>) -> Self {
        DepGraph { 
            nodes: vec![root],
            edges: vec![],
            cfg: cfg
        }
    }

    pub fn add_child(&mut self, parent: usize, d: &str, k: Option<DepKind>) -> usize {
        let idr = self.find_or_add(d, k.unwrap_or(DepKind::Build));
        self.edges.push((parent, idr));
        idr
    }

    pub fn find_or_add(&mut self, new: &str, k: DepKind) -> usize {
        for (i, d) in self.nodes.iter().enumerate() {
            if d.name == new {
                return i
            }
        }
        self.nodes.push(Dep::with_kind(new.to_owned(), k));
        self.nodes.len() - 1
    }

    pub fn render_to<W:Write>(self, output: &mut W) -> CliResult<()> {
        cli_try!(writeln!(output, "{}", "digraph dependencies {"));
        for (i, dep) in self.nodes.iter().enumerate() {
            cli_try!(write!(output, "\tN{}",i));
            cli_try!(dep.label(output, self.cfg));
        }
        for (il, ir) in self.edges.into_iter() {
            cli_try!(writeln!(output, "\tN{} -> N{}[label=\"\"];",il, ir));
        }
        cli_try!(writeln!(output, "{}", "}"));
        Ok(())
    }
}
