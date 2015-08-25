use std::ascii::AsciiExt;

use clap::ArgMatches;

use LineStyle;
use error::{CliError, CliResult};

trait BoolArg {
    fn parse_arg(&self) -> CliResult<bool>;
}

impl<'a> BoolArg for &'a str {
    fn parse_arg(&self) -> CliResult<bool> {
        match &*self.to_ascii_lowercase() {
            "yes"  | 
            "true" |
            "y"    | 
            "t"     => Ok(true),
            "no"    | 
            "false" | 
            "n"     | 
            "f"      => Ok(false),
            _        =>  Err(CliError::UnknownBoolArg)
        }
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    pub lock_file: &'a str,
    pub manifest_file: &'a str,
    pub dot_file: Option<&'a str>,
    pub dev_lines: LineStyle,
    pub build_lines: LineStyle,
    pub optional_lines: LineStyle,
    pub dev_deps: bool,
    pub build_deps: bool,
    pub optional_deps: bool
}

impl<'a> Config<'a> {
    pub fn from_matches(m: &'a ArgMatches) -> CliResult<Self> {
        Ok(Config {
            lock_file: m.value_of("lock-file").unwrap_or("Cargo.lock"),
            manifest_file: m.value_of("manifest-file").unwrap_or("Cargo.toml"),
            dot_file: m.value_of("dot-file"),
            dev_lines: value_t!(m.value_of("dev-style"), LineStyle).unwrap_or(LineStyle::Solid),
            build_lines: value_t!(m.value_of("build-style"), LineStyle).unwrap_or(LineStyle::Solid),
            dev_deps: cli_try!(m.value_of("dev-deps").unwrap_or("false").parse_arg()),
            build_deps: cli_try!(m.value_of("build-deps").unwrap_or("true").parse_arg()),
            optional_deps: cli_try!(m.value_of("optional-deps").unwrap_or("true").parse_arg()),
            optional_lines: value_t!(m.value_of("optional-style"), LineStyle).unwrap_or(LineStyle::Solid),
        })
    }
}