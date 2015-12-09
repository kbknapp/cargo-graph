use std::collections::HashMap;

use toml::Value;

use dep::{Dep, DepKind};
use graph::DepGraph;
use error::{CliErrorKind, CliResult};
use config::Config;
use util;

macro_rules! propagate_kind {
    ($me:ident, $kind:expr) => ({
        let mut changes = false;
        loop {
            for (dep, children) in &$me.dep_tree {
                if $me.styles.get(dep).unwrap() == &$kind {
                    for child in children {
                        if $me.styles.get(child).unwrap() != &$kind {
                            *$me.styles.get_mut(child).unwrap() = $kind;
                            changes = true;
                        }
                    }
                }
            }
            if !changes { break; }
            changes = false;
        }
    });
}

pub struct Project<'c, 'o>
    where 'o: 'c
{
    cfg: &'c Config<'o>,
    styles: HashMap<String, DepKind>,
    dep_tree: HashMap<String, Vec<String>>,
    blacklist: Vec<String>,
}

impl<'c, 'o> Project<'c, 'o> {
    pub fn with_config(cfg: &'c Config<'o>) -> CliResult<Self> {
        Ok(Project {
            cfg: cfg,
            styles: HashMap::new(),
            dep_tree: HashMap::new(),
            blacklist: vec![],
        })
    }

    pub fn graph(mut self) -> CliResult<DepGraph<'c, 'o>> {
        let mut dg = try!(self.parse_root_deps());

        try!(self.parse_lock_file(&mut dg));

        self.update_styles(&mut dg);

        Ok(dg)
    }

    fn update_styles(&mut self, dg: &mut DepGraph<'c, 'o>) {
        propagate_kind!(self, DepKind::Dev);
        propagate_kind!(self, DepKind::Optional);
        propagate_kind!(self, DepKind::Build);
        for (name, kind) in &self.styles {
            if (*kind == DepKind::Dev && !self.cfg.dev_deps) ||
               (*kind == DepKind::Optional && !self.cfg.optional_deps) ||
               (*kind == DepKind::Build && !self.cfg.build_deps) {
                dg.remove(&*name);
            } else {
                dg.update_style(&*name, *kind);
            }
        }
        dg.remove_orphans();
    }

    fn parse_lock_file(&mut self, dg: &mut DepGraph<'c, 'o>) -> CliResult<()> {
        let lock_path = try!(util::find_manifest_file(self.cfg.lock_file));
        let lock_toml = try!(util::toml_from_file(lock_path));

        if let Some(&Value::Array(ref packages)) = lock_toml.get("package") {
            for pkg in packages {
                let mut children = vec![];
                let name = pkg.lookup("name")
                              .expect("no 'name' field in Cargo.lock [package] table")
                              .as_str()
                              .expect("'name' field of [package] table in Cargo.lock was not a \
                                       valid string")
                              .to_owned();
                let kind = match self.styles.get(&name) {
                    Some(k) => k.clone(),
                    None => DepKind::Build,
                };
                let id = dg.find_or_add(&*name, kind);

                if let Some(&Value::Array(ref deps)) = pkg.lookup("dependencies") {
                    for dep in deps {
                        let dep_string =
                            dep.as_str().unwrap_or("").split(" ").collect::<Vec<_>>()[0];

                        children.push(dep_string.to_owned());
                        self.styles.insert(dep_string.to_owned(), kind);
                        dg.add_child(id, dep_string, Some(kind));
                    }
                    self.dep_tree.insert(name.to_owned(), children);
                } else {
                    self.dep_tree.insert(name.to_owned(), vec![]);
                }
                self.styles.insert(name.to_owned(), kind);
            }
        }

        debugln!("all lock deps: {:#?}", dg);
        Ok(())
    }

    pub fn parse_root_deps(&mut self) -> CliResult<DepGraph<'c, 'o>> {
        debugln!("executing; parse_root_deps;");
        let manifest_path = try!(util::find_manifest_file(self.cfg.manifest_file));
        let manifest_toml = try!(util::toml_from_file(manifest_path));
        let root_table = match manifest_toml.get("package") {
            Some(table) => table,
            None => return Err(From::from(CliErrorKind::TomlTableRoot)),
        };

        let proj_name = root_table.lookup("name")
                                  .expect("no 'name' field in the project manifest file's \
                                           [package] table")
                                  .as_str()
                                  .expect("'name' field in the project manifest file's [package] \
                                           table isn't a valid string")
                                  .to_owned();

        let root = Dep::new(proj_name);
        let root_id = 0;
        let mut dg = DepGraph::with_root(root, self.cfg);

        if let Some(table) = manifest_toml.get("dependencies") {
            if let Some(table) = table.as_table() {
                for (name, dep_table) in table.into_iter() {
                    if let Some(&Value::Boolean(opt)) = dep_table.lookup("optional") {
                        self.styles.insert(name.clone(), DepKind::Optional);
                        dg.add_child(root_id, name, Some(DepKind::Optional));
                        if !self.cfg.optional_deps && opt {
                            self.blacklist.push(name.clone());
                        }
                    } else {
                        self.styles.insert(name.clone(), DepKind::Build);
                        dg.add_child(root_id, name, None);
                    }
                }
            }
        }

        if let Some(table) = manifest_toml.get("dev-dependencies") {
            if let Some(table) = table.as_table() {
                for (name, _) in table.into_iter() {
                    let d = self.styles.entry(name.clone()).or_insert(DepKind::Dev);
                    if self.cfg.dev_deps { *d = DepKind::Dev; };
                    dg.add_child(root_id, name, Some(*d));
                    if !self.cfg.dev_deps {
                        self.blacklist.push(name.clone());
                    }
                }
            }
        }

        debugln!("root deps: {:#?}", dg);
        Ok(dg)
    }
}
