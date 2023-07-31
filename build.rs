extern crate phf_codegen;

use std::char;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    run().expect("Failed to generate 'emojis.rs'");
}

/// Given an input file containing base-16 representations of emoji code points, generates
/// an array of length 1024 of these emojis to serve as an alphabet for the ecoji encoding.
///
/// Padding characters are generated here as well.
///
/// Also generates a reverse mapping from code points to the indices of the respective code points
/// in the alphabet array using the phf crate.
fn run() -> Result<(), Box<dyn Error>> {
    const VERSIONS: usize = 2;

    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("emojis.rs");
    let mut output = BufWriter::new(File::create(&dest_path)?);

    // https://github.com/keith-turner/ecoji/blob/main/v2/mapping.go
    let padding = [
        None,
        Some([0x269C, 0x1F3CD, 0x1F4D1, 0x1F64B]),
        Some([0x1F977, 0x1F6FC, 0x1F4D1, 0x1F64B]),
    ];

    for version in 1..=VERSIONS {
        writeln!(
            &mut output,
            r"pub const VERSION{version} : Version = Version {{"
        )?;

        writeln!(&mut output, r"  VERSION_NUMBER: {version},")?;

        let padding = &padding[version].unwrap();
        writeln!(&mut output, r"  PADDING: '\u{{2615}}',")?;
        for (i, bytes) in padding.iter().enumerate() {
            writeln!(&mut output, r"  PADDING_4{i}: '\u{{{bytes:x}}}',")?;
        }

        let path = format!("emojisV{version}.txt");
        let input = BufReader::new(File::open(&path)?);

        let mut rev_map = phf_codegen::Map::new();

        writeln!(&mut output, "  EMOJIS: [")?;
        for (i, line) in input.lines().into_iter().take(1024).enumerate() {
            let line = line?;
            writeln!(&mut output, r"    '\u{{{}}}',  // {}", &line, i)?;
            rev_map.entry(
                char::from_u32(u32::from_str_radix(&line, 16).unwrap()).unwrap(),
                &i.to_string(),
            );
        }
        writeln!(&mut output, "  ],")?;

        write!(&mut output, "  EMOJIS_REV: ")?;
        rev_map.build(&mut output)?;
        writeln!(&mut output, ",")?;

        writeln!(&mut output, r"}};")?;
    }

    write!(
        &mut output,
        r"pub const VERSIONS : [&Version; {VERSIONS}] = ["
    )?;
    for version in 1..=2 {
        write!(&mut output, r"&VERSION{version},")?;
    }
    writeln!(&mut output, r"];")?;

    Ok(())
}
