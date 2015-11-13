use std::env;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;

use toml::{self, Table};

use error::{CliErrorKind, CliResult};

pub fn toml_from_file<P: AsRef<Path>>(p: P) -> CliResult<Box<Table>> {
    debugln!("executing; from_file; file={:?}", p.as_ref());
    let mut f = try!(File::open(p.as_ref()));

    let mut s = String::new();
    try!(f.read_to_string(&mut s));

    let mut parser = toml::Parser::new(&s);
    if let Some(toml) = parser.parse() {
        return Ok(Box::new(toml));
    }

    // On err
    let mut error_str = format!("could not parse input as TOML\n");
    for error in parser.errors.iter() {
        let (loline, locol) = parser.to_linecol(error.lo);
        let (hiline, hicol) = parser.to_linecol(error.hi);
        error_str.push_str(&format!("{:?}:{}:{}{} {}\n",
                                    f,
                                    loline + 1,
                                    locol + 1,
                                    if loline != hiline || locol != hicol {
                                        format!("-{}:{}", hiline + 1, hicol + 1)
                                    } else {
                                        "".to_owned()
                                    },
                                    error.desc));
    }
    Err(From::from(CliErrorKind::Generic(error_str)))
}

pub fn find_manifest_file(file: &str) -> CliResult<PathBuf> {
    let mut pwd = try!(env::current_dir());

    loop {
        let manifest = pwd.join(file);
        if let Ok(metadata) = fs::metadata(&manifest) {
            if metadata.is_file() {
                return Ok(manifest);
            }
        }

        let pwd2 = pwd.clone();
        let parent = pwd2.parent();
        if let None = parent {
            break;
        }
        pwd = parent.unwrap().to_path_buf();
    }

    Err(From::from(CliErrorKind::Generic(format!("Could not find `{}` in `{}` or any \
                                                  parent directory, or it isn't a valid \
                                                  lock-file",
                                                 file,
                                                 pwd.display()))))
}
