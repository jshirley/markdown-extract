mod error;

use error::NoMatchesError;
use markdown_extract::{extract_from_reader, MarkdownSection};
use regex::RegexBuilder;
use std::error::Error;
use std::path::PathBuf;
use std::io::{self};
use std::fs;
use structopt::StructOpt;

/// Extract sections of a markdown file according to a regular expression.
#[derive(StructOpt)]
#[structopt(name = "markdown-extract")]
pub struct Opts {
    /// Print all matching sections (don't quit after first match)
    #[structopt(short, long)]
    all: bool,

    /// Treat pattern as case sensitive
    #[structopt(short = "s", long)]
    case_sensitive: bool,

    /// Do not include the matched heading in the output
    #[structopt(short, long)]
    no_print_matched_heading: bool,

    /// Pattern to match against headings
    pattern: String,

    /// Path to markdown file
    #[structopt(parse(from_os_str))]
    path: Option<PathBuf>,

    /// Read from stdin instead of a file
    #[structopt(long)]
    stdin: bool,
}

fn print_section(section: &MarkdownSection, no_print_matched_heading: bool) {
    let iterator = section
        .iter()
        .skip(if no_print_matched_heading { 1 } else { 0 });

    for line in iterator {
        println!("{}", line);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();

    let regex = RegexBuilder::new(&opts.pattern)
        .case_insensitive(!opts.case_sensitive)
        .size_limit(1024 * 100) // 100 kb
        .build()
        .unwrap();

    let matches = match (opts.path, opts.stdin) {
        (Some(i), false) => extract_from_reader(fs::File::open(i)?, &regex)?,
        (None, true) => extract_from_reader(io::stdin(), &regex)?,
        (Some(_), true) => panic!("Must supply a path or --stdin, but not both"),
        (None, false) => panic!("Must supply a path or --stdin")
    };

    if matches.len() == 0 {
        return Err(Box::new(NoMatchesError::new()));
    }

    if !opts.all {
        print_section(&matches[0], opts.no_print_matched_heading);
    } else {
        for m in matches.iter() {
            print_section(&m, opts.no_print_matched_heading);
        }
    }

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(error) => {
            println!("Error: {}", error);
            1
        }
    })
}
