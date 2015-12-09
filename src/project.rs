use std::collections::HashMap;

use toml::Value;

use dep::{Dep, DepKind};
use graph::DepGraph;
use error::{CliErrorKind, CliResult};
use config::Config;
use util;

const INTERNAL_ERROR: &'static str = "error: a fatal internal error has occurred, this is \
                                             a bug - please consider filing a bug report at\n\t\
                                             https://github.com/kbknapp/cargo-graph/issues";
// macro_rules! propagate_kind {
//     ($me:ident, $kind:expr) => ({
        // let mut changes = false;
            // for (dep, children) in &$me.dep_tree {
            //     debugln!("iter; dep={}; children={:?}", dep, children);
            //     if $me.styles.get(dep).expect(INTERNAL_ERROR) == &$kind {
            //         for child in children {
            //             if $me.styles.get(child).unwrap() != &$kind {
            //                 debugln!("iter; child={}; kind={:?}", child, &$kind);
            //                 *$me.styles.get_mut(child).unwrap() = $kind;
            //                 changes = true;
            //             }
            //         }
            //     }
            // }
            // if !changes { break; }
            // changes = false;
//         }
//     });
// }

#[derive(Debug)]
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
        // debugln!("exec=update_styles; dg={:#?}", dg);
        // propagate_kind!(self, DepKind::Dev);
        // debugln!("exec=update_styles; dg={:#?}", dg);
        // propagate_kind!(self, DepKind::Optional);
        // debugln!("exec=update_styles; dg={:#?}", dg);
        // propagate_kind!(self, DepKind::Build);
        // debugln!("exec=update_styles; dg={:#?}", dg);
        // let mut parent = &dg.root;
        debugln!("exec=upate_styles; self={:#?}", self);
        let mut parents: HashMap<_, _> = self.dep_tree.keys().cloned().map(|n| (n, false)).collect();
        parents.remove(&dg.root);
        for child in self.dep_tree.get(&dg.root).expect(INTERNAL_ERROR) {
            debugln!("iter=upate_styles; child={}", child);
            *parents.get_mut(child).expect(INTERNAL_ERROR) = true;
        }
        loop {
            debugln!("iter=upate_styles; parents={:#?}", parents);
            let parent = if let Some(parent) = parents.iter().filter(|&(n, &s)| s).map(|(n,_)| n).nth(0) {
                debugln!("{} set to true", &*parent);
                parent.clone()
            } else {
                debugln!("nothing true any more; parents={:#?}", parents);
                break;
            };
            parents.remove(&parent);
            let kind = self.styles.get(&parent).expect(INTERNAL_ERROR).clone();
            for child in self.dep_tree.get(&parent).expect(INTERNAL_ERROR) {
                if let Some(c) = parents.get_mut(child) {
                    *c = true;
                }
                if let Some(k) = self.styles.get_mut(child) {
                    *k = kind;
                }
            }
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
        dg.remove_orphans();
    }

    fn update_name(&mut self, old: &str, new: &str, dg: &mut DepGraph<'c, 'o>) {
        debugln!("exec=update_name; old={}; new={};", old, new);
        let kind = self.styles.remove(old);
        if let Some(k) = kind {
            debugln!("found and removed {}", old);
            self.styles.insert(new.to_owned(), k);
            let kids = self.dep_tree.remove(old);
            if let Some(k) = kids {
                self.dep_tree.insert(new.to_owned(), k);
            }
            dg.update_name(old, new);
        } else {
            debugln!("{} not found", old);
        }
        if let Some(ref mut v) = self.dep_tree.get_mut(&dg.root) {
            debugln!("Searching for match in root children");
            let mut j = v.len();
            for (i, d) in v.iter().enumerate() {
                debugln!("iter; i={}; d={}", i, d);
                if d == old { j = i; break; }
            }
            debugln!("iter; v_before={:?}", v);
            if j < v.len() {
                v.push(new.to_owned());
                v.swap_remove(j);
                debugln!("iter; v_after={:?}", v);
            }
        }
    }

    fn parse_lock_file(&mut self, dg: &mut DepGraph<'c, 'o>) -> CliResult<()> {
        debugln!("exec=parse_lock_file;self={:#?}", self);
        let lock_path = try!(util::find_manifest_file(self.cfg.lock_file));
        let lock_toml = try!(util::toml_from_file(lock_path));

        if let Some(&Value::Array(ref packages)) = lock_toml.get("package") {
            if let Some(ref mut v) = self.dep_tree.get_mut(&dg.root) {
                v.sort();
                v.dedup();
            }
            for pkg in packages {
                let mut children = vec![];
                let d_name = pkg.lookup("name")
                              .expect("no 'name' field in Cargo.lock [package] table")
                              .as_str()
                              .expect("'name' field of [package] table in Cargo.lock was not a \
                                       valid string");
                let name = if self.cfg.include_vers {
                    format!("{} v{}", d_name,
                        pkg.lookup("version")
                              .expect("no 'version' field in Cargo.lock [package] table")
                              .as_str()
                              .expect("'version' field of [package] table in Cargo.lock was not a \
                                       valid string"))
                } else {
                    d_name.to_owned()
                };
                self.update_name(d_name, &*name, dg);
                let kind = match self.styles.get(&name) {
                    Some(k) => {
                        debugln!("Got kind {:?} for {}", *k, &*name);
                        *k
                    },
                    None    => {
                        debugln!("{} not found, returning Unk", &*name);
                        DepKind::Unk
                    }
                };
                let id = dg.find_or_add(&*name, kind);

                if let Some(&Value::Array(ref deps)) = pkg.lookup("dependencies") {
                    for dep in deps {
                        let dep_vec = dep.as_str().unwrap_or("").split(" ").collect::<Vec<_>>();
                        let dep_string = if self.cfg.include_vers {
                            format!("{} v{}", dep_vec[0], dep_vec[1])
                        } else {
                            dep_vec[0].to_owned()
                        };

                        children.push(dep_string.clone());
                        self.styles.insert(dep_string.clone(), kind);
                        dg.add_child(id, &*dep_string, Some(kind));
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
            Dep::with_kind(format!("{} v{}", proj_name.clone(), proj_ver), DepKind::Build)
        } else {
            Dep::with_kind(proj_name.clone(), DepKind::Build)
        };
        let root_id = 0;
        let root_name = root.name.clone();
        self.styles.insert(root_name.clone(), DepKind::Build);
        let mut dg = DepGraph::with_root(root, self.cfg);
        let mut v = vec![];

        if let Some(table) = manifest_toml.get("dependencies") {
            if let Some(table) = table.as_table() {
                for (name, dep_table) in table.into_iter() {
                    if let Some(&Value::Boolean(opt)) = dep_table.lookup("optional") {
                        let d = self.styles.entry(name.clone()).or_insert(DepKind::Optional);
                        if self.cfg.optional_deps && opt { *d = DepKind::Optional; };
                        dg.add_child(root_id, name, Some(*d));
                        if !self.cfg.optional_deps && opt {
                            self.blacklist.push(name.clone());
                        }
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
                    if self.cfg.dev_deps && *d != DepKind::Build { *d = DepKind::Dev; };
                    dg.add_child(root_id, name, Some(*d));
                    if !self.cfg.dev_deps {
                        self.blacklist.push(name.clone());
                    }
                    v.push(name.clone());
                }
            }
        }
        self.dep_tree.insert(root_name, v);

        debugln!("return=parse_root_deps; self={:#?}", self);
        Ok(dg)
    }
}
