extern crate ecoji;
#[macro_use]
extern crate clap;

use std::io;

use clap::{App, AppSettings};
use ecoji::*;

fn main() {
    let matches = App::new("ecoji")
        .version(crate_version!())
        .author("Vladimir Matveev <vladimir.matweev@gmail.com>")
        .about(
            "Encode or decode data in standard input as emojis and print results to standard output.\n\
             A Rust reimplementation of the original Ecoji library and tool (https://github.com/keith-turner/ecoji)."
        )
        .setting(AppSettings::ColoredHelp)
        .args_from_usage("-d, --decode 'Decode data'")
        .args_from_usage("-1, --version1 'Use version 1 (default)'")
        .args_from_usage("-2, --version2 'Use version 2'")
        .get_matches();

    let version = match (matches.is_present("version1"), matches.is_present("version2")) {
        (true, true) => panic!("Both V1 and V2 selected."),
        (false, true) => VERSION2,
        (_ , false) => VERSION1,
    };

    let (stdin, stdout) = (io::stdin(), io::stdout());
    let (mut stdin, mut stdout) = (stdin.lock(), stdout.lock());
    if matches.is_present("decode") {
        version.decode(&mut stdin, &mut stdout).expect("Failed to decode data");
    } else {
        version.encode(&mut stdin, &mut stdout).expect("Failed to encode data");
    }
}
