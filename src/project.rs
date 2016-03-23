use std::collections::{HashMap, HashSet};

use toml::Value;

use dep::{Dep, DepKind};
use graph::DepGraph;
use error::{CliErrorKind, CliResult};
use config::Config;
use util;

const INTERNAL_ERROR: &'static str = "error: a fatal internal error has occurred, this is \
                                             a bug - please consider filing a bug report at\n\t\
                                             https://github.com/kbknapp/cargo-graph/issues";
#[derive(Debug)]
pub struct Project<'c, 'o>
    where 'o: 'c
{
    cfg: &'c Config<'o>,
    styles: HashMap<String, DepKind>,
    dep_tree: HashMap<String, Vec<String>>,
}

impl<'c, 'o> Project<'c, 'o> {
    pub fn with_config(cfg: &'c Config<'o>) -> CliResult<Self> {
        Ok(Project {
            cfg: cfg,
            styles: HashMap::new(),
            dep_tree: HashMap::new(),
        })
    }

    pub fn graph(mut self) -> CliResult<DepGraph<'c, 'o>> {
        let mut dg = try!(self.parse_root_deps());

        try!(self.parse_lock_file(&mut dg));

        self.update_styles(&mut dg);

        Ok(dg)
    }

    fn propagate(&mut self, root: &str, done: &mut HashSet<String>) {
        let root_kids = self.dep_tree.get(root).expect(INTERNAL_ERROR);
        for child in  root_kids {
            done.insert(child.to_owned());
            debugln!("iter=upate_styles; child={}", child);
            let kind = *self.styles.get(child).expect(INTERNAL_ERROR);
            for g_child in self.dep_tree.get(child).expect(INTERNAL_ERROR) {
                if !root_kids.contains(g_child) {
                    if let Some(k) = self.styles.get_mut(g_child) {
                        debugln!("iter; setting child={}; to kind={:?}", g_child, kind);
                        if !(kind != DepKind::Build && *k == DepKind::Build) {
                            *k = kind;
                        }
                    }
                }
            }
        }
    }

    fn update_styles(&mut self, dg: &mut DepGraph<'c, 'o>) {
        debugln!("exec=upate_styles; styles={:#?}", self.styles);
        let parents: HashSet<String> = self.dep_tree.keys().map(ToOwned::to_owned).collect();
        let mut done: HashSet<String> = HashSet::new();
        done.insert(dg.root.clone());
        self.propagate(&*dg.root, &mut done);
        for p in parents {
            self.propagate(&*p, &mut done);
        }
        for (name, kind) in &self.styles {
            if (*kind == DepKind::Dev && !self.cfg.dev_deps) ||
               (*kind == DepKind::Optional && !self.cfg.optional_deps) ||
               (*kind == DepKind::Build && !self.cfg.build_deps) {
                dg.remove(&*name);
            } else {
                dg.update_style(&*name, *kind);
            }
        }
        debugln!("updatee_styles; nodes_after={:#?}", dg.nodes);
        dg.remove_orphans();
    }

    fn parse_lock_file(&mut self, dg: &mut DepGraph<'c, 'o>) -> CliResult<()> {
        debugln!("exec=parse_lock_file; styles={:#?}", self.styles);
        let lock_path = try!(util::find_manifest_file(self.cfg.lock_file));
        let lock_toml = try!(util::toml_from_file(lock_path));

        if let Some(&Value::Array(ref packages)) = lock_toml.get("package") {
            if let Some(ref mut v) = self.dep_tree.get_mut(&dg.root) {
                v.sort();
                v.dedup();
            }
            for pkg in packages {
                let mut children = vec![];
                let name = pkg.lookup("name")
                              .expect("no 'name' field in Cargo.lock [package] table")
                              .as_str()
                              .expect("'name' field of [package] table in Cargo.lock was not a \
                                       valid string")
                              .to_owned();
                if self.cfg.include_vers {
                    let ver = pkg.lookup("version")
                              .expect("no 'version' field in Cargo.lock [package] table")
                              .as_str()
                              .expect("'version' field of [package] table in Cargo.lock was not a \
                                       valid string")
                              .to_owned();
                    dg.update_ver(&*name, ver);
                }
                debugln!("iter; checking kind for {}", name);
                let kind = match self.styles.get(&*name) {
                    Some(k) => {
                        debugln!("Got kind {:?} for {}", *k, name);
                        *k
                    },
                    None    => {
                        debugln!("{} not found, returning Unk", name);
                        DepKind::Unk
                    }
                };
                let id = dg.find_or_add(&*name, kind);

                if let Some(&Value::Array(ref deps)) = pkg.lookup("dependencies") {
                    for dep in deps {
                        let dep_vec = dep.as_str().unwrap_or("").split(' ').collect::<Vec<_>>();
                        let dep_string = dep_vec[0].to_owned();
                        if self.cfg.include_vers {
                            let ver = dep_vec[1];
                            dg.update_ver(&*dep_string, ver);
                        }
                        children.push(dep_string.clone());
                        let kind = self.styles.entry(dep_string.clone()).or_insert(DepKind::Unk);
                        dg.add_child(id, &*dep_string, Some(*kind));
                    }
                    self.dep_tree.insert(name.clone(), children);
                } else {
                    self.dep_tree.insert(name.clone(), vec![]);
                }
                self.styles.insert(name, kind);
            }
        }

        debugln!("return=parse_lock_file; self={:#?}", self);
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

        let root = if self.cfg.include_vers {
            let proj_ver = root_table.lookup("version")
                                  .expect("no 'version' field in the project manifest file's \
                                           [package] table")
                                  .as_str()
                                  .expect("'version' field in the project manifest file's [package] \
                                           table isn't a valid string");
            let mut d = Dep::with_kind(proj_name.clone(), DepKind::Build);
            d.ver(proj_ver);
            d
        } else {
            Dep::with_kind(proj_name.clone(), DepKind::Build)
        };
        let root_id = 0;
        let root_name = root.name.clone();
        self.styles.insert(root_name.clone(), DepKind::Build);
        let mut dg = DepGraph::with_root(root, self.cfg);
        self.styles.insert(proj_name.clone(), DepKind::Build);
        let mut v = vec![];

        if let Some(table) = manifest_toml.get("dependencies") {
            if let Some(table) = table.as_table() {
                for (name, dep_table) in table.into_iter() {
                    if let Some(&Value::Boolean(opt)) = dep_table.lookup("optional") {
                        let d = self.styles.entry(name.clone()).or_insert(DepKind::Optional);
                        if self.cfg.optional_deps && opt { *d = DepKind::Optional; };
                        dg.add_child(root_id, name, Some(*d));
                    } else {
                        self.styles.insert(name.clone(), DepKind::Build);
                        dg.add_child(root_id, name, Some(DepKind::Build));
                    }
                    v.push(name.clone());
                }
            }
        }

        if let Some(table) = manifest_toml.get("dev-dependencies") {
            if let Some(table) = table.as_table() {
                for (name, _) in table.into_iter() {
                    let d = self.styles.entry(name.clone()).or_insert(DepKind::Dev);
                    debugln!("iter; name={}; d={:?}", name, d);
                    if self.cfg.dev_deps && *d != DepKind::Build { *d = DepKind::Dev; };
                    dg.add_child(root_id, name, Some(*d));
                    v.push(name.clone());
                }
            }
        }
        self.dep_tree.insert(root_name, v);

        debugln!("return=parse_root_deps; self={:#?}", self);
        Ok(dg)
    }
}
