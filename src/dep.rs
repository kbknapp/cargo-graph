use std::io::{Result, Write};

use config::Config;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DepKind {
    Build,
    Dev,
    Optional,
    Unk,
}

#[derive(Debug)]
pub struct DeclaredDep {
    pub name: String,
    pub kind: DepKind,
}

impl DeclaredDep {
    pub fn with_kind(name: String, kind: DepKind) -> Self {
        DeclaredDep {
            name: name,
            kind: kind,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ResolvedDep {
    pub name: String,
    pub ver: String,
    pub is_build: bool,
    pub is_optional: bool,
    pub is_dev: bool,
    pub force_write_ver: bool,
}

impl ResolvedDep {
    pub fn new(name: String, ver: String) -> Self {
        ResolvedDep {
            name: name,
            ver: ver,
            is_build: false,
            is_optional: false,
            is_dev: false,
            force_write_ver: false,
        }
    }

    pub fn kind(&self) -> DepKind {
        if self.is_build {
            DepKind::Build
        } else if self.is_dev {
            DepKind::Dev
        } else if self.is_optional {
            DepKind::Optional
        } else {
            DepKind::Unk
        }
    }

    pub fn label<W: Write>(&self, w: &mut W, c: &Config) -> Result<()> {
        let name = if self.force_write_ver || c.include_vers {
            format!("{} v{}", self.name, self.ver)
        } else {
            self.name.clone()
        };
        match self.kind() {
            DepKind::Dev => writeln!(w, "[label={:?}{}];", name, c.dev_style),
            DepKind::Optional => writeln!(w, "[label={:?}{}];", name, c.optional_style),
            _ => writeln!(w, "[label={:?}{}];", name, c.build_style),
        }
    }
}
