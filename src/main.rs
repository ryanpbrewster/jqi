use serde_json::Value;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use termion::event::{Event, Key};
use termion::input::{MouseTerminal, TermRead};
use termion::{cursor::Goto, raw::IntoRawMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();
    let data: Value = {
        let fin = std::fs::File::open(args.input)?;
        let reader = BufReader::new(fin);
        serde_json::from_reader(reader)?
    };
    let stdin = std::io::stdin();
    let mut stdout = MouseTerminal::from(std::io::stdout().into_raw_mode()?);

    write!(stdout, "{}", termion::clear::All)?;
    stdout.flush()?;
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
        write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
        let keys: Vec<String> = match data {
            Value::Array(_)
            | Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_) => vec![],
            Value::Object(ref obj) => obj.keys().cloned().collect(),
        };
        for (i, key) in keys.iter().enumerate() {
            write!(stdout, "{}{}\n", Goto(1, 2 + i as u16), key)?;
        }
        stdout.flush()?;
    }

    Ok(())
}

#[derive(StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}
