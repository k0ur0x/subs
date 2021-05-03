use ansi_term::Color::{Blue, Green, Red};
use regex::Regex;
use std::{
    error,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read},
    path::Path,
    result,
};
use subtitles::SubRip;

type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub struct Config<'a> {
    pub regex: Regex,
    pub paths: Vec<&'a str>,
}

pub fn run(config: Config) -> Result<()> {
    for path in config.paths {
        if path == "-" {
            from_stdin(&config.regex);
        } else {
            from_path(path, &config.regex)?;
        }
    }
    Ok(())
}

fn from_stdin(regex: &Regex) {
    let stdin = io::stdin();
    let handle = stdin.lock();
    find(handle, &regex);
}

fn from_path(path: impl AsRef<Path>, regex: &Regex) -> Result<()> {
    let file_type = fs::metadata(&path)?.file_type();

    if file_type.is_dir() {
        from_dir(path, regex)?;
    } else if file_type.is_file() {
        from_file(path, regex)?;
    }

    Ok(())
}

fn from_dir(path: impl AsRef<Path>, regex: &Regex) -> Result<()> {
    let entries = fs::read_dir(&path)?;
    for entry in entries {
        let entry = entry?;
        from_path(entry.path(), regex)?;
    }

    Ok(())
}

fn from_file(path: impl AsRef<Path>, regex: &Regex) -> Result<()> {
    print_file_name(path.as_ref());
    let file = File::open(path)?;
    find(file, &regex);

    Ok(())
}

fn print_file_name(path: &Path) {
    let file_name = path
        .file_stem()
        .unwrap_or_else(|| OsStr::new(""))
        .to_str()
        .unwrap_or("");

    println!("{}", Blue.paint(file_name));
}

fn find<T: Read>(subtitle: T, regex: &Regex) {
    let parser = subtitles::open(subtitle);

    for entry in parser {
        match entry {
            Ok(sub) => print_matches(sub, regex),
            Err(err) => eprintln!("{}: {}", Red.paint("Error"), err),
        }
    }
}

fn print_matches(subtitle: SubRip, regex: &Regex) {
    let text = subtitle.text.join("\n");
    let mut last_uncolored = 0;

    for mat in regex.find_iter(&text) {
        print!(
            "{}{}",
            &text[last_uncolored..mat.start()],
            Green.paint(&text[mat.start()..mat.end()])
        );
        last_uncolored = mat.end();
    }

    if last_uncolored != 0 {
        println!("{}", &text[last_uncolored..]);
    }
}
