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
    let root: Value = {
        let fin = std::fs::File::open(args.input)?;
        let reader = BufReader::new(fin);
        serde_json::from_reader(reader)?
    };
    let mut data = &root;

    let mut path = vec![0];
    let mut fields = get_fields(data);

    let stdin = std::io::stdin();
    let mut stdout = HideCursor::from(std::io::stdout().into_raw_mode()?);

    write_fields(&mut stdout, &fields, 0)?;
    for evt in stdin.events() {
        let key = match evt? {
            Event::Key(key) => key,
            _ => continue,
        };
        match key {
            Key::Esc | Key::Char('q') => break,
            Key::Char('j') => {
                let idx = path.last_mut().unwrap();
                if *idx + 1 < fields.len() {
                    *idx += 1;
                }
            }
            Key::Char('k') => {
                let idx = path.last_mut().unwrap();
                if *idx > 0 {
                    *idx -= 1;
                }
            }
            Key::Char('l') => {
                data = descend(data, *path.last().unwrap());
                fields = get_fields(data);
                path.push(0);
            }
            _ => continue,
        };
        write_fields(&mut stdout, &fields, *path.last().unwrap())?;
    }

    // Restore the cursor and then exit.
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    Ok(())
}

fn get_fields(v: &Value) -> Vec<String> {
    match v {
        Value::Array(_) | Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            vec![]
        }
        Value::Object(ref obj) => obj.keys().cloned().collect(),
    }
}

fn descend(v: &Value, idx: usize) -> &Value {
    match v {
        Value::Array(_) | Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => v,
        Value::Object(ref obj) => {
            let key = match obj.keys().nth(idx) {
                None => return v,
                Some(key) => key,
            };
            obj.get(key).unwrap_or(v)
        }
    }
}

fn write_fields(
    stdout: &mut RawTerminal<Stdout>,
    fields: &[String],
    highlighted: usize,
) -> std::io::Result<()> {
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    for (i, key) in fields.iter().enumerate() {
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
