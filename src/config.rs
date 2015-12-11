use std::io::Write;
use std::ascii::AsciiExt;
use std::fmt;

use clap::ArgMatches;

use error::{CliErrorKind, CliResult};

trait BoolArg {
    fn parse_arg(&self) -> CliResult<bool>;
}

impl<'a> BoolArg for &'a str {
    fn parse_arg(&self) -> CliResult<bool> {
        match &*self.to_ascii_lowercase() {
            "yes" |
            "true" |
            "y" |
            "t" => Ok(true),
            "no" |
            "false" |
            "n" |
            "f" => Ok(false),
            _ => Err(From::from(CliErrorKind::UnknownBoolArg)),
        }
    }
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    pub enum DotLineShape {
        Solid,
        Dotted,
        Dashed
    }
}

impl DotLineShape {
    fn write(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DotLineShape::Solid => Ok(()),
            DotLineShape::Dotted => write!(f, ",style=dotted"),
            DotLineShape::Dashed => write!(f, ",style=dashed"),
        }
    }
}

arg_enum!{
    #[derive(Debug, Copy, Clone)]
    pub enum DotColor {
        Blue,
        Green,
        Red,
        Yellow,
        White,
        Black,
        Purple,
        Orange
    }
}

impl DotColor {
    fn write(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DotColor::Blue => write!(f, ",color=blue"),
            DotColor::Green => write!(f, ",color=green"),
            DotColor::Red => write!(f, ",color=red"),
            DotColor::Yellow => write!(f, ",color=yellow"),
            DotColor::White => write!(f, ",color=white"),
            DotColor::Black => Ok(()),
            DotColor::Purple => write!(f, ",color=purple"),
            DotColor::Orange => write!(f, ",color=orange"),
        }
    }
}

arg_enum!{
    #[derive(Debug, Copy, Clone)]
    pub enum DotShape {
        Box,
        Round,
        Diamond,
        Triangle
    }
}

impl DotShape {
    fn write(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DotShape::Round => Ok(()),
            DotShape::Box => write!(f, ",shape=box"),
            DotShape::Diamond => write!(f, ",shape=diamond"),
            DotShape::Triangle => write!(f, ",shape=triangle"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DepStyle(DotShape, DotColor);

impl fmt::Display for DepStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.0.write(f));
        self.1.write(f)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LineStyle(DotLineShape, DotColor);

impl fmt::Display for LineStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.0.write(f));
        self.1.write(f)
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
    pub optional_deps: bool,
    pub build_style: DepStyle,
    pub dev_style: DepStyle,
    pub optional_style: DepStyle,
    pub include_vers: bool,
}

impl<'a> Config<'a> {
    pub fn from_matches(m: &'a ArgMatches) -> CliResult<Self> {
        Ok(Config {
            lock_file: m.value_of("lock-file").unwrap_or("Cargo.lock"),
            manifest_file: m.value_of("manifest-file").unwrap_or("Cargo.toml"),
            dot_file: m.value_of("dot-file"),
            dev_deps: try!(m.value_of("dev-deps").unwrap_or("false").parse_arg()),
            build_deps: try!(m.value_of("build-deps").unwrap_or("true").parse_arg()),
            optional_deps: try!(m.value_of("optional-deps").unwrap_or("true").parse_arg()),
            build_lines: LineStyle(value_t!(m.value_of("build-line-style"), DotLineShape)
                                       .unwrap_or(DotLineShape::Solid),
                                   value_t!(m.value_of("build-line-color"), DotColor)
                                       .unwrap_or(DotColor::Black)),
            optional_lines: LineStyle(value_t!(m.value_of("optional-line-style"), DotLineShape)
                                          .unwrap_or(DotLineShape::Solid),
                                      value_t!(m.value_of("optional-line-color"), DotColor)
                                          .unwrap_or(DotColor::Black)),
            dev_lines: LineStyle(value_t!(m.value_of("dev-line-style"), DotLineShape)
                                     .unwrap_or(DotLineShape::Solid),
                                 value_t!(m.value_of("dev-line-color"), DotColor)
                                     .unwrap_or(DotColor::Black)),
            build_style: DepStyle(value_t!(m.value_of("build-shape"), DotShape)
                                      .unwrap_or(DotShape::Round),
                                  value_t!(m.value_of("build-color"), DotColor)
                                      .unwrap_or(DotColor::Black)),
            optional_style: DepStyle(value_t!(m.value_of("optional-shape"), DotShape)
                                         .unwrap_or(DotShape::Round),
                                     value_t!(m.value_of("optional-color"), DotColor)
                                         .unwrap_or(DotColor::Black)),
            dev_style: DepStyle(value_t!(m.value_of("dev-shape"), DotShape)
                                    .unwrap_or(DotShape::Round),
                                value_t!(m.value_of("dev-color"), DotColor)
                                    .unwrap_or(DotColor::Black)),
            include_vers: m.is_present("include-versions"),
        })
    }
}
