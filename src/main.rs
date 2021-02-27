use serde_json::Value;
use std::io::BufReader;
use std::io::Stdout;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use termion::cursor::{Goto, HideCursor};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::style;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();
    let data: Value = {
        let fin = std::fs::File::open(args.input)?;
        let reader = BufReader::new(fin);
        serde_json::from_reader(reader)?
    };
    let stdin = std::io::stdin();
    let mut stdout = HideCursor::from(std::io::stdout().into_raw_mode()?);

    write_keys(&mut stdout, &data, 0)?;
    for evt in stdin.events() {
        match evt? {
            Event::Key(key) => {
                match key {
                    Key::Esc => break,
                    _ => {}
                };
            }
            _ => {}
        }
    }

    // Restore the cursor and then exit.
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    Ok(())
}

fn write_keys(
    stdout: &mut RawTerminal<Stdout>,
    v: &Value,
    highlighted: usize,
) -> std::io::Result<()> {
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    let keys: Vec<String> = match v {
        Value::Array(_) | Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            vec![]
        }
        Value::Object(ref obj) => obj.keys().cloned().collect(),
    };
    for (i, key) in keys.iter().enumerate() {
        write!(stdout, "{}", Goto(1, 1 + i as u16))?;
        if i == highlighted {
            write!(
                stdout,
                "{}{}{}",
                style::Bold,
                style::Italic,
                style::Underline
            )?;
        }
        write!(stdout, "{}{}", key, style::Reset)?;
    }
    stdout.flush()?;
    Ok(())
}

#[derive(StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}
