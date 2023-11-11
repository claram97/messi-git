use std::io::{Write, self};

use crate::ignorer::Ignorer;

pub fn git_check_ignore(ignorer_name: &str, ignorer: &Ignorer, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    if line.len() == 1 || (line.len() == 2 && line[1].eq("-v")) {
        writeln!(output, "fatal: no path specified")?;
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No path specified"));
    } else {
        let verbose = line[1] == "-v";
        let mut line_number = 1;

        for path in line.iter().skip(2) {
            if ignorer.ignore(path) {
                if verbose {
                    writeln!(output, "{}:{}:{}", ignorer_name, line_number, path)?;
                } else {
                    writeln!(output, "{}", path)?;
                }
            }
            line_number += 1;
        }
    }
    Ok(())
}
