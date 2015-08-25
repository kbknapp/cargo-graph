#[derive(PartialEq, Debug)]
pub enum DepKind {
    Build,
    Dev,
    Optional
}

#[derive(Debug, PartialEq)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind 
}

impl Dep {
    pub fn new(name: String) -> Self {
        Dep {
            name: name,
            kind: DepKind::Build
        }
    }

    pub fn with_kind(name: String, kind: DepKind) -> Self {
        Dep {
            name: name,
            kind: kind
        }
    }
}
