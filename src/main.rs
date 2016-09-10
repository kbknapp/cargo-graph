//!
//! A `cargo` subcommand for building `GraphViz` DOT files of dependency graphs.
//! This subcommand was originally based off and inspired by the project
//! [cargo-dot](https://github.com/maxsnew/cargo-dot) by [Max
//! New](https://github.com/maxsnew)
//!
//!
//! ## Demo
//!
//! Let's say we wanted to build a dependency graph of
//! [cargo-count](https://github.com/kbknapp/cargo-count) but we wanted
//! optional dependencies to use red dashed lines and black boxes, and regular
//! (aka "build") dependencies to use orange lines to green diamonds, one would
//! run the following.
//!
//! **NOTE:** `GraphViz` `dot` needs to be installed to produce the .PNG from the
//! dotfile
//!
//! ```ignore
//! $ cargo graph --optional-line-style dashed --optional-line-color red
//! --optional-shape box --build-shape diamond --build-color green
//! --build-line-color orange > cargo-count.dot
//! $ dot -Tpng > rainbow-graph.png cargo-count.dot
//! ```
//!
//! **NOTE:** It's also possible to run `cargo graph [options] | dot [options]
//! > [file]` instead of individual commands
//!
//! The above commands would produce the following graph:
//!
//! ![cargo-graph dependencies](rainbow-graph.png)
//!
//! Now, *why* someone would do that to a graph is a different story...but it's
//! possible :)
//!
//! ## Installing
//!
//! `cargo-graph` can be installed with `cargo install`
//!
//! ```ignore
//! $ cargo install cargo-graph
//! ```
//!
//! This may require a nightly version of `cargo` if you get an error about the
//! `install` command not being found. You may also compile and install the
//! traditional way by following the instructions below.
//!
//!
//! ## Compiling
//!
//! Follow these instructions to compile `cargo-count`, then skip down to
//! Installation.
//!
//! 1. Ensure you have current version of `cargo` and
//! [Rust](https://www.rust-lang.org) installed
//! 2. Clone the project `$ git clone https://github.com/kbknapp/cargo-graph
//! && cd cargo-graph`
//! 3. Build the project `$ cargo build --release` (**NOTE:** There is a large
//! performance difference when compiling without optimizations, so I recommend
//! always using `--release` to enable to them)
//! 4. Once complete, the binary will be located at
//! `target/release/cargo-graph`
//!
//! ## Installation and Usage
//!
//! All you need to do is place `cargo-graph` somewhere in your `$PATH`. Then
//! run `cargo graph` anywhere in your project directory. For full details see
//! below.
//!
//! ### Linux / OS X
//!
//! You have two options, place `cargo-graph` into a directory that is already
//! located in your `$PATH` variable (To see which directories those are, open
//! a terminal and type `echo "${PATH//:/\n}"`, the quotation marks are
//! important), or you can add a custom directory to your `$PATH`
//!
//! **Option 1**
//! If you have write permission to a directory listed in your `$PATH` or you
//! have root permission (or via `sudo`), simply copy the `cargo-graph` to that
//! directory `# sudo cp cargo-graph/usr/local/bin`
//!
//! **Option 2**
//! If you do not have root, `sudo`, or write permission to any directory
//! already in `$PATH` you can create a directory inside your home directory,
//! and add that. Many people use `$HOME/.bin` to keep it hidden (and not
//! clutter your home directory), or `$HOME/bin` if you want it to be always
//! visible. Here is an example to make the directory, add it to `$PATH`, and
//! copy `cargo-graph` there.
//!
//! Simply change `bin` to whatever you'd like to name the directory, and
//! `.bashrc` to whatever your shell startup file is (usually `.bashrc`,
//! `.bash_profile`, or `.zshrc`)
//!
//! ```ignore
//! $ mkdir ~/bin
//! $ echo "export PATH=$PATH:$HOME/bin" >> ~/.bashrc
//! $ cp cargo-graph~/bin
//! $ source ~/.bashrc
//! ```
//!
//! ### Windows
//!
//! On Windows 7/8 you can add directory to the `PATH` variable by opening a
//! command line as an administrator and running
//!
//! ```ignore
//! C\:> setx path \"%path%;C:\path\to\cargo-graph\binary\"
//! ```
//!
//! Otherwise, ensure you have the `cargo-graph` binary in the directory which
//! you operating in the command line from, because Windows automatically adds
//! your current directory to PATH (i.e. if you open a command line to
//! `C:\my_project\` to use `cargo-graph` ensure `cargo-graph.exe` is inside
//! that directory as well).
//!
//!
//! ### Options
//!
//! There are a few options for using `cargo-graph` which should be somewhat
//! self explanatory.
//!
//! ```ignore
//! USAGE:
//!     cargo graph [FLAGS] [OPTIONS]
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//! --build-color <COLOR>            Color for regular deps (Defaults
//! to 'black')
//! [values: blue black yellow purple
//! green red white orange]
//! --build-deps <true|false>        Should build deps be in the graph?
//! (Defaults to 'true')
//! ex. --build-deps=false OR
//! --build-deps=no
//! --build-line-color <COLOR>       Line color for regular deps
//! (Defaults to 'black')
//! [values: blue black yellow purple
//! green red white orange]
//! --build-line-style <STYLE>       Line style for build deps
//! (Defaults to 'solid')
//!                                           [values: solid dotted dashed]
//! --build-shape <SHAPE>            Shape for regular deps (Defaults
//! to 'round')
//! [values: box round diamond
//! triangle]
//! --dev-color <COLOR>              Color for dev deps (Defaults to
//! 'black')
//! [values: blue black yellow purple
//! green red white orange]
//! --dev-deps <true|false>          Should dev deps be included in the
//! graph? (Defaults to 'false')
//! ex. --dev-deps=true OR
//! --dev-deps=yes
//! --dev-line-color <COLOR>         Line color for dev deps (Defaults
//! to 'black')
//! [values: blue black yellow purple
//! green red white orange]
//! --dev-line-style <STYLE>         Line style for dev deps (Defaults
//! to 'solid')
//!                                           [values: solid dotted dashed]
//! --dev-shape <SHAPE>              Shape for dev deps (Defaults to
//! 'round')
//! [values: box round diamond
//! triangle]
//!         --dot-file <FILE>                Output file (Default to stdout)
//! --lock-file <FILE>               Specify location of .lock file
//! (Default 'Cargo.lock')
//! --manifest-file <FILE>           Specify location of manifest file
//! (Default 'Cargo.toml')
//! --optional-color <COLOR>         Color for optional deps (Defaults
//! to 'black')
//! [values: blue black yellow purple
//! green red white orange]
//! --optional-deps <true|false>     Should optional deps be in the
//! graph? (Defaults to 'true')
//! ex. --optional-deps=false OR
//! --optional-deps=no
//! --optional-line-color <COLOR>    Line color for optional deps
//! (Defaults to 'black')
//! [values: blue black yellow purple
//! green red white orange]
//! --optional-line-style <STYLE>    Line style for optional deps
//! (Defaults to 'solid')
//!                                           [values: solid dotted dashed]
//! --optional-shape <SHAPE>         Shape for optional deps (Defaults
//! to 'round')
//! [values: box round diamond
//! triangle]
//! ```
//!
//! ## License
//!
//! `cargo-graph` is released under the terms of the MIT. See the LICENSE-MIT
//! file for the details.
//!
//! ## Dependencies Tree
//! ![cargo-graph dependencies](cargo-graph.png)

#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]
#![cfg_attr(feature = "lints", allow(explicit_iter_loop))]
#![cfg_attr(feature = "lints", allow(should_implement_trait))]
#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", deny(warnings))]
#![cfg_attr(not(any(feature = "nightly", feature = "unstable")), deny(unstable_features))]
#![deny(missing_docs,
        missing_debug_implementations,
        missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unused_import_braces,
        unused_qualifications)]

extern crate toml;
#[macro_use]
extern crate clap;
#[cfg(feature = "color")]
extern crate ansi_term;

use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use error::CliResult;
use config::Config;
use project::Project;

#[macro_use]
mod macros;
mod error;
mod graph;
mod fmt;
mod project;
mod dep;
mod config;
mod util;

static LINE_STYLES: [&'static str; 3] = ["solid", "dotted", "dashed"];
static COLORS: [&'static str; 8] = ["blue", "black", "yellow", "purple", "green", "red", "white",
                                    "orange"];
static DEP_SHAPES: [&'static str; 4] = ["box", "round", "diamond", "triangle"];

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("cargo-graph")
        .version(&*format!("v{}", crate_version!()))
        .bin_name("cargo")
        .settings(&[AppSettings::GlobalVersion, AppSettings::SubcommandRequired])
        .subcommand(SubCommand::with_name("graph")
                        .author("Kevin K. <kbknapp@gamil.com>\nMax New <maxsnew@gmail.com>")
                        .about("Generate a graph of package dependencies in graphviz format")
                        .args_from_usage("
                            -I, --include-versions 'Include the dependency version on nodes'
                                --dot-file [PATH] 'Output file (Default stdout)'
                                --dev-deps [true|false] 'Should dev deps be included in the graph? (Default false, also allows yes|no)'
                                --build-deps [true|false] 'Should build deps be in the graph? (Default true, also allows yes|no)'
                                --optional-deps [true|false] 'Should optional deps be in the graph? (Default true, also allows yes|no)'
                        ")
                        .args(&[
                            Arg::from_usage("--lock-file [PATH] 'Specify location of .lock file (Default Cargo.lock)'")
                                .validator(is_file),
                            Arg::from_usage("--manifest-file [PATH] 'Specify location of manifest file (Default Cargo.toml)'")
                                .validator(is_file),
                            Arg::from_usage("--build-line-style [STYLE] 'Line style for build deps (Default solid)'")
                                .possible_values(&LINE_STYLES),
                            Arg::from_usage("--build-line-color [COLOR] 'Line color for regular deps (Default black)'")
                                .possible_values(&COLORS),
                            Arg::from_usage("--build-shape [SHAPE] 'Shape for regular deps (Default round)'")
                                .possible_values(&DEP_SHAPES),
                            Arg::from_usage("--build-color [COLOR] 'Color for regular deps (Default black)'")
                                .possible_values(&COLORS),
                            Arg::from_usage("--optional-line-style [STYLE] 'Line style for optional deps (Default solid)'")
                                .possible_values(&LINE_STYLES),
                            Arg::from_usage("--optional-line-color [COLOR] 'Line color for optional deps (Default black)'")
                                .possible_values(&COLORS),
                            Arg::from_usage("--optional-shape [SHAPE] 'Shape for optional deps (Default round)'")
                                .possible_values(&DEP_SHAPES),
                            Arg::from_usage("--optional-color [COLOR] 'Color for optional deps (Default black)'")
                                .possible_values(&COLORS),
                            Arg::from_usage("--dev-line-style [STYLE] 'Line style for dev deps (Default solid)'")
                                .possible_values(&LINE_STYLES),
                            Arg::from_usage("--dev-line-color [COLOR] 'Line color for dev deps (Default black)'")
                                .possible_values(&COLORS),
                            Arg::from_usage("--dev-shape [SHAPE] 'Shape for dev deps (Default round)'")
                                .possible_values(&DEP_SHAPES),
                            Arg::from_usage("--dev-color [COLOR] 'Color for dev deps (Default black)'")
                                .possible_values(&COLORS)]))
        .get_matches()
}

fn main() {
    debugln!("parsing cli...");
    let m = parse_cli();

    if let Some(m) = m.subcommand_matches("graph") {
        let cfg = Config::from_matches(m).unwrap_or_else(|e| e.exit());
        debugln!("cfg={:#?}", cfg);
        execute(cfg).map_err(|e| e.exit()).unwrap();
    }
}

fn execute(cfg: Config) -> CliResult<()> {
    let project = try!(Project::with_config(&cfg));
    let graph = try!(project.graph());

    match cfg.dot_file {
        None => {
            let o = io::stdout();
            let mut bw = BufWriter::new(o.lock());
            graph.render_to(&mut bw)
        }
        Some(file) => {
            let o = File::create(&Path::new(&file)).expect("Failed to create file");
            let mut bw = BufWriter::new(o);
            graph.render_to(&mut bw)
        }
    }
}

fn is_file(s: String) -> Result<(), String> {
    let p = Path::new(&*s);
    if let None = p.file_name() {
        return Err(format!("'{}' doesn't appear to be a valid file name", &*s));
    }
    Ok(())
}
