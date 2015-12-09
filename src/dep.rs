use std::io::{Result, Write};

use config::Config;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DepKind {
    Build,
    Dev,
    Optional,
    Unk,
}

#[derive(Debug, PartialEq)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind,
}

impl Dep {
    pub fn new(name: String) -> Self {
        Dep {
            name: name,
            kind: DepKind::Unk,
        }
    }

    pub fn with_kind(name: String, kind: DepKind) -> Self {
        Dep {
            name: name,
            kind: kind,
        }
    }

    pub fn label<W: Write>(&self, w: &mut W, c: &Config) -> Result<()> {
        match self.kind {
            DepKind::Dev => writeln!(w, "[label={:?}{}];", self.name, c.dev_style),
            DepKind::Optional => writeln!(w, "[label={:?}{}];", self.name, c.optional_style),
            _ => writeln!(w, "[label={:?}{}];", self.name, c.build_style),
        }

    }
}
