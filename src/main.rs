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
    let mut data = vec![&root];
    let mut pos = vec![0];
    let mut fields = get_fields(data.last().unwrap());

    let stdin = std::io::stdin();
    let mut stdout = HideCursor::from(std::io::stdout().into_raw_mode()?);

    write_fields(&mut stdout, &fields, *pos.last().unwrap())?;
    for evt in stdin.events() {
        let key = match evt? {
            Event::Key(key) => key,
            _ => continue,
        };
        match key {
            Key::Esc | Key::Char('q') => break,
            Key::Char('j') => {
                let idx = pos.last_mut().unwrap();
                if *idx + 1 < fields.len() {
                    *idx += 1;
                }
            }
            Key::Char('k') => {
                let idx = pos.last_mut().unwrap();
                if *idx > 0 {
                    *idx -= 1;
                }
            }
            Key::Char('h') => {
                if data.len() > 1 {
                    data.pop();
                    pos.pop();
                    fields = get_fields(data.last().unwrap());
                }
            }
            Key::Char('l') => {
                if let Some(child) = descend(data.last().unwrap(), *pos.last().unwrap()) {
                    data.push(child);
                    fields = get_fields(data.last().unwrap());
                    pos.push(0);
                }
            }
            _ => continue,
        };
        write_fields(&mut stdout, &fields, *pos.last().unwrap())?;
    }

    // Restore the cursor and then exit.
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    Ok(())
}

fn get_fields(v: &Value) -> Vec<String> {
    match v {
        Value::Null => vec!["null".to_owned()],
        Value::Bool(b) => vec![b.to_string()],
        Value::Number(x) => vec![x.to_string()],
        Value::String(s) => vec![s.to_owned()],
        Value::Array(ref vs) => (0..vs.len()).map(|i| i.to_string()).collect(),
        Value::Object(ref obj) => obj.keys().cloned().collect(),
    }
}

fn descend(v: &Value, idx: usize) -> Option<&Value> {
    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
        Value::Array(ref vs) => vs.get(idx),
        Value::Object(ref obj) => obj.keys().nth(idx).and_then(|key| obj.get(key)),
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
