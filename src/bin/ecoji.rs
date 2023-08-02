extern crate clap;
extern crate ecoji;

use std::io;

use clap::{arg, crate_version, ArgAction, Command};
use ecoji::*;

fn main() {
    let matches = Command::new("ecoji")
        .version(crate_version!())
        .author("Vladimir Matveev <vladimir.matweev@gmail.com>")
        .about(
            "Encode or decode data in standard input as emojis and print results to standard output.\n\
             A Rust reimplementation of the original Ecoji library and tool (https://github.com/keith-turner/ecoji)."
        )
        .arg(arg!(-d --decode "Decode data").action(ArgAction::SetTrue))
        .arg(arg!(--v1 "Use version 1 (default)").action(ArgAction::SetTrue))
        .arg(arg!(--v2 "Use version 2").action(ArgAction::SetTrue))
        .get_matches();

    let version = match (matches.get_flag("v1"), matches.get_flag("v2")) {
        (true, true) => panic!("Both V1 and V2 selected."),
        (false, true) => VERSION2,
        (_, false) => VERSION1,
    };

    let (stdin, stdout) = (io::stdin(), io::stdout());
    let (mut stdin, mut stdout) = (stdin.lock(), stdout.lock());
    if matches.get_flag("decode") {
        version
            .decode(&mut stdin, &mut stdout)
            .expect("Failed to decode data");
    } else {
        version
            .encode(&mut stdin, &mut stdout)
            .expect("Failed to encode data");
    }
}
