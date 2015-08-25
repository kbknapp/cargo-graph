use std::io::Write;

// use dot;

use LineStyle;
use config::Config;
use dep::{Dep, DepKind};
use error::{CliError, CliResult};

pub type Nd = usize;
pub type Ed = (usize, usize);
#[derive(Debug)]
pub struct DepGraph {
    nodes: Vec<Dep>,
    pub edges: Vec<Ed>,
    dev_style: LineStyle,
    build_style: LineStyle,
    optional_style: LineStyle
}

impl DepGraph {
    pub fn with_root(root: Dep, cfg: &Config) -> Self {
        DepGraph { 
            nodes: vec![root],
            edges: vec![] ,
            dev_style: cfg.dev_lines,
            build_style: cfg.build_lines,
            optional_style: cfg.optional_lines
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
            cli_try!(writeln!(output, "\tN{}[label={:?}];",i, dep.name));
        }
        for (il, ir) in self.edges.into_iter() {
            cli_try!(writeln!(output, "\tN{} -> N{}[label=\"\"];",il, ir));
        }
        cli_try!(writeln!(output, "{}", "}"));
        Ok(())
    }
}
