use std::io::Read;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use toml::{self, Value, Table};

use dep::{Dep, DepKind};
use graph::DepGraph;
use error::{CliError, CliResult};
use config::Config;

pub struct Project<'c, 'o> where 'o: 'c {
    pwd: PathBuf,
    cfg: &'c Config<'o>,
}

impl<'c, 'o> Project<'c, 'o> {
    pub fn from_config(cfg: &'c Config<'o>) -> CliResult<Self> {
        let pwd = if let Ok(pwd) = env::current_dir() {
            pwd
        } else {
            return Err(CliError::CurrentDir);
        };


        Ok(Project{
            pwd: pwd,
            cfg: cfg,
        })
    }


    fn find_project_file(&self, file: &str) -> CliResult<PathBuf> {
        let mut pwd = self.pwd.as_path();

        loop {
            let manifest = pwd.join(file);
            if let Ok(metadata) =  fs::metadata(&manifest) {
                if metadata.is_file() {
                    return Ok(manifest)
                }
            }

            match pwd.parent() {
                Some(p) => pwd = p,
                None => break,
            }
        }

        Err(CliError::Generic(format!("Could not find `{}` in `{}` or any parent directory, or it isn't a valid lock-file",
                          file, pwd.display())))
    }

    fn toml_from_file<P: AsRef<Path>>(p: P) -> CliResult<Box<Table>> {
        debugln!("executing; from_file; file={:?}", p.as_ref());
        let mut f = match File::open(p.as_ref()) {
            Ok(f) => f,
            Err(e) => return Err(CliError::FileOpen(e.description().to_owned()))
        };

        let mut s = String::new();
        if let Err(e) = f.read_to_string(&mut s) {
            return Err(CliError::Generic(format!("Couldn't read the contents of Cargo.lock with error: {}", e.description())))
        }

        let mut parser = toml::Parser::new(&s);
        match parser.parse() {
            Some(toml) => return Ok(Box::new(toml)),
            None => {}
        }

        // On err
        let mut error_str = format!("could not parse input as TOML\n");
        for error in parser.errors.iter() {
            let (loline, locol) = parser.to_linecol(error.lo);
            let (hiline, hicol) = parser.to_linecol(error.hi);
            error_str.push_str(&format!("{:?}:{}:{}{} {}\n",
                                        f,
                                        loline + 1, locol + 1,
                                        if loline != hiline || locol != hicol {
                                            format!("-{}:{}", hiline + 1,
                                                    hicol + 1)
                                        } else {
                                            "".to_owned()
                                        },
                                        error.desc));
        }
        Err(CliError::Generic(error_str))
    }

    pub fn graph(mut self) -> CliResult<DepGraph<'c, 'o>> {
        let dg = cli_try!(self.parse_root_deps());

        self.parse_lock_file(dg)
    }

    fn parse_lock_file(&self, mut dg: DepGraph<'c, 'o>) -> CliResult<DepGraph<'c, 'o>> {
        let lock_path = cli_try!(self.find_project_file(self.cfg.lock_file));
        let lock_toml = cli_try!(Project::toml_from_file(lock_path));

        if let Some(&Value::Array(ref packages)) = lock_toml.get("package") {
            for pkg in packages {
                let name = pkg.lookup("name")
                              .expect("no 'name' field in Cargo.lock [package] table")
                              .as_str()
                              .expect("'name' field of [package] table in Cargo.lock was not a valid string");
                let id = dg.find_or_add(name, DepKind::Build);

                if let Some(&Value::Array(ref deps)) = pkg.lookup("dependencies") {
                    for dep in deps {
                        let dep_string = dep.as_str().unwrap_or("").split(" ").collect::<Vec<_>>()[0];

                        dg.add_child(id, dep_string, None);
                    }
                }
            }
        }

        debugln!("all lock deps: {:#?}", dg);
        Ok(dg)
    }

    pub fn parse_root_deps(&mut self) -> CliResult<DepGraph<'c, 'o>> {
        debugln!("executing; parse_root_deps;");
        let manifest_path = cli_try!(self.find_project_file(self.cfg.manifest_file));
        let manifest_toml = cli_try!(Project::toml_from_file(manifest_path));
        let root_table = match manifest_toml.get("package") {
            Some(table) => table,
            None        => return Err(CliError::TomlTableRoot)
        };

        let proj_name = root_table.lookup("name")
            .expect("no 'name' field in the project manifest file's [package] table")
            .as_str()
            .expect("'name' field in the project manifest file's [package] table isn't a valid string").to_owned();

        let root = Dep::new(proj_name);
        let root_id = 0;
        let mut dg = DepGraph::with_root(root, self.cfg);

        if let Some(table) = manifest_toml.get("dependencies") {
            if let Some(table) = table.as_table() {
                for (name, dep_table) in table.iter() {
                    if let Some(&Value::Boolean(opt)) = dep_table.lookup("optional") {
                        if self.cfg.optional_deps && opt {
                            dg.add_child(root_id, name, Some(DepKind::Optional));
                        }
                    } else {
                        dg.add_child(root_id, name, None);
                    }
                }
            }
        }

        if self.cfg.dev_deps {
            if let Some(table) = manifest_toml.get("dev-dependencies") {
                if let Some(table) = table.as_table() {
                    for (name, _) in table.iter() {
                        dg.add_child(root_id, name, Some(DepKind::Dev));
                    }
                }
            }
        }

        debugln!("root deps: {:#?}", dg);

        Ok(dg)
    }
}
