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
    pub ver: Option<String>,
}

impl Dep {
    pub fn with_kind(name: String, kind: DepKind) -> Self {
        Dep {
            name: name,
            kind: kind,
            ver: None,
        }
    }

    pub fn ver<S: Into<String>>(&mut self, ver: S) {
        self.ver = Some(ver.into());
    }

    pub fn label<W: Write>(&self, w: &mut W, c: &Config) -> Result<()> {
        let name = if c.include_vers {
            format!("{}{}", self.name, if let Some(ref v) = self.ver { format!(" v{}", v) } else { String::new() })
        } else {
            self.name.clone()
        };
        match self.kind {
            DepKind::Dev => writeln!(w, "[label={:?}{}];", name, c.dev_style),
            DepKind::Optional => writeln!(w, "[label={:?}{}];", name, c.optional_style),
            _ => writeln!(w, "[label={:?}{}];", name, c.build_style),
        }

    }
}
