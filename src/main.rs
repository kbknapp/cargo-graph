extern crate toml;
extern crate graphviz;

use graphviz as dot;
use std::io::{File};

fn main() {
    let contents =
        File::open(&Path::new("Cargo.lock")).read_to_string();
    match contents {
        Err(e) => {
            println!("Error: {}", e);
            std::os::set_exit_status(1)
        },
        Ok(toml_str) => {
            match toml::Parser::new(toml_str.as_slice()).parse() {
                None => {
                    println!("Cargo.lock is invalid toml");
                    std::os::set_exit_status(1)
                }
                Some(toml) => {
                    println!("Valid toml serialization: {}", toml::Table(toml))
                }
            }
        }
    }
}
