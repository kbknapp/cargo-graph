#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]
#![cfg_attr(feature = "lints", deny(warnings))]
#![cfg_attr(feature = "lints", allow(option_unwrap_used))]

extern crate toml;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use clap::{App, AppSettings, Arg, SubCommand, ArgMatches};

use error::{CliError, CliResult};
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

static LINE_STYLES: [&'static str; 3] = ["solid", "dotted", "dashed"];
static COLORS: [&'static str; 8] = ["blue", "black", "yellow", "purple", "green", "red", "white", "orange"];
static DEP_SHAPES: [&'static str; 2] = ["box", "round"];

fn parse_cli<'a, 'b>() -> ArgMatches<'a, 'b> {
    App::new("cargo-dot")
        .version(&*format!("v{}", crate_version!()))
        // We have to lie about our binary name since this will be a third party
        // subcommand for cargo but we want usage strings to generated properly
        .bin_name("cargo")
        // Global version uses the version we supplied (Cargo.toml) for all subcommands as well
        .settings(&[AppSettings::GlobalVersion,
                    AppSettings::SubcommandRequired])
        // We use a subcommand because everything parsed after `cargo` is sent to the third party 
        // plugin which will then be interpreted as a subcommand/positional arg by clap
        .subcommand(SubCommand::with_name("dot")
            .author("Max New <maxsnew@gmail.com>\n\
                     Kevin K. <kbknapp@gmail.com>")
            .about("Generate a graph of package dependencies in graphviz format")
            // Here we list the valid arguments. There are less verbose was to do this with clap
            // using "usage strings"; but because we want to set some additional properties not
            // available to "usage string" syntax, we set them individually 
            //
            // The two properites we need to set which aren't available to usage strings are
            // 'value_name()' which sets name for the argument inside help messages and usage
            // strings displayed to the user. The default is to use the name with which we access
            // the arg values later, but in this case we want two arguments to have the same "name"
            // such as "FILE" which would conflict. This setting allows us to work around that
            // conflict.
            //
            // The second setting is 'possible_values()' which defines a set of valid values for
            // the argument. The benefit to using this setting (which isn't mandatory to accept
            // a specific set of values) is the user will have all values presented to them in
            // help messages automatically, and will receive suggestions when they typo a specific
            // value.
            .args(vec![
                Arg::with_name("lock-file")
                    .help("Specify location of .lock file (Default \'Cargo.lock\')")
                    .long("lock-file")
                    .value_name("FILE")
                    .validator(is_file)
                    .takes_value(true),
                Arg::with_name("manifest-file")
                    .help("Specify location of manifest file (Default \'Cargo.toml\')")
                    .long("manifest-file")
                    .value_name("FILE")
                    .validator(is_file)
                    .takes_value(true),
                Arg::with_name("dot-file")
                    .help("Output file (Default to stdout)")
                    .long("dot-file")
                    .takes_value(true)
                    .value_name("FILE"),
                Arg::with_name("dev-deps")
                    .help("Should dev deps be included in the graph? (Defaults to \'false\'){n}\
                           ex. --dev-deps=true OR --dev-deps=yes")
                    .long("--dev-deps")
                    .takes_value(true)
                    .value_name("true|false"),
                Arg::with_name("build-deps")
                    .help("Should build deps be in the graph? (Defaults to \'true\'){n}\
                           ex. --build-deps=false OR --build-deps=no")
                    .long("--build-deps")
                    .takes_value(true)
                    .value_name("true|false"),
                Arg::with_name("optional-deps")
                    .help("Should opitonal deps be in the graph? (Defaults to \'true\'){n}\
                           ex. --optional-deps=false OR --optional-deps=no")
                    .long("--optional-deps")
                    .takes_value(true)
                    .value_name("true|false"),
                // REGULAR "BUILD" DEP STYLE OPTIONS
                Arg::with_name("build-line-style")
                    .help("Line style for build deps (Defaults to \'solid\'){n}")
                    .long("build-line-style")
                    .takes_value(true)
                    .value_name("STYLE")
                    .possible_values(&LINE_STYLES),
                Arg::with_name("build-line-color")
                    .help("Line color for regular deps (Defaults to \'black\'){n}")
                    .long("build-line-color")
                    .takes_value(true)
                    .value_name("COLOR")
                    .possible_values(&COLORS),
                Arg::with_name("build-shape")
                    .help("Shape for regular deps (Defaults to \'round\'){n}")
                    .long("build-shape")
                    .takes_value(true)
                    .value_name("SHAPE")
                    .possible_values(&DEP_SHAPES),
                Arg::with_name("build-color")
                    .help("Color for regular deps (Defaults to \'black\'){n}")
                    .long("build-color")
                    .takes_value(true)
                    .value_name("COLOR")
                    .possible_values(&COLORS),
                // OPTIONAL DEP STYLE OPTIONS
                Arg::with_name("optional-line-style")
                    .help("Line style for optional deps (Defaults to \'solid\'){n}")
                    .long("optional-line-style")
                    .takes_value(true)
                    .value_name("STYLE")
                    .possible_values(&LINE_STYLES),
                Arg::with_name("optional-line-color")
                    .help("Line color for optional deps (Defaults to \'black\'){n}")
                    .long("optional-line-color")
                    .takes_value(true)
                    .value_name("COLOR")
                    .possible_values(&COLORS),
                Arg::with_name("optional-shape")
                    .help("Shape for optional deps (Defaults to \'round\'){n}")
                    .long("optional-shape")
                    .takes_value(true)
                    .value_name("SHAPE")
                    .possible_values(&DEP_SHAPES),
                Arg::with_name("optional-color")
                    .help("Color for optional deps (Defaults to \'black\'){n}")
                    .long("optional-color")
                    .takes_value(true)
                    .value_name("COLOR")
                    .possible_values(&COLORS),
                // DEV DEP STYLE OPTIONS
                Arg::with_name("dev-line-style")
                    .help("Line style for dev deps (Defaults to \'solid\'){n}")
                    .long("dev-line-style")
                    .takes_value(true)
                    .value_name("STYLE")
                    .possible_values(&LINE_STYLES),
                Arg::with_name("dev-line-color")
                    .help("Line color for dev deps (Defaults to \'black\'){n}")
                    .long("dev-line-color")
                    .takes_value(true)
                    .value_name("COLOR")
                    .possible_values(&COLORS),
                Arg::with_name("dev-shape")
                    .help("Shape for dev deps (Defaults to \'round\'){n}")
                    .long("dev-shape")
                    .takes_value(true)
                    .value_name("SHAPE")
                    .possible_values(&DEP_SHAPES),
                Arg::with_name("dev-color")
                    .help("Color for dev deps (Defaults to \'black\'){n}")
                    .long("dev-color")
                    .takes_value(true)
                    .value_name("COLOR")
                    .possible_values(&COLORS)]))
        .get_matches()
}

fn main() {
    let m = parse_cli();

    if let Some(m) = m.subcommand_matches("dot") {
        let cfg = Config::from_matches(m).unwrap_or_else(|e| e.exit());
        debugln!("cfg={:#?}", cfg);
        if let Err(e) = execute(cfg) {
            e.exit();
        }
    }
}

fn execute(cfg: Config) -> CliResult<()> {
    let project = cli_try!(Project::from_config(&cfg));
    let graph = cli_try!(project.graph());

    match cfg.dot_file {
        None       => {
            let o = io::stdout();
            let mut bw = BufWriter::new(o.lock());
            graph.render_to(&mut bw)
        },
        Some(file) => {
            let o = File::create(&Path::new(&file)).ok().expect("Failed to create file");
            let mut bw = BufWriter::new(o);
            graph.render_to(&mut bw)
        }
    }
}

fn is_file(s: String) -> Result<(), String> {
    let p = Path::new(&*s);
    if let None = p.file_name() {
        return Err(format!("'{}' doesn't appear to be a valid file name", &*s))
    }
    Ok(())
}