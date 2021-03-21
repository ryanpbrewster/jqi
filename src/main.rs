use serde_json::Value;
use std::io::Stdout;
use std::io::Write;
use std::path::PathBuf;
use std::{fmt::Display, io::BufReader};
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
    let mut data = NonEmptyVec::new(&root);
    let mut pos = NonEmptyVec::new(0);

    let stdin = std::io::stdin();
    let mut stdout = HideCursor::from(std::io::stdout().into_raw_mode()?);

    write_data(&mut stdout, data.last(), *pos.last())?;
    for evt in stdin.events() {
        let key = match evt? {
            Event::Key(key) => key,
            _ => continue,
        };
        match key {
            Key::Esc | Key::Char('q') => break,
            Key::Char('j') => {
                let idx = pos.last_mut();
                if *idx + 1 < field_count(data.last()) {
                    *idx += 1;
                }
            }
            Key::Char('k') => {
                let idx = pos.last_mut();
                if *idx > 0 {
                    *idx -= 1;
                }
            }
            Key::Char('h') => {
                data.pop();
                pos.pop();
            }
            Key::Char('l') => {
                if let Some(child) = descend(data.last(), *pos.last()) {
                    data.push(child);
                    pos.push(0);
                }
            }
            _ => continue,
        };
        write_data(&mut stdout, data.last(), *pos.last())?;
    }

    // Restore the cursor and then exit.
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    Ok(())
}

fn field_count(v: &Value) -> usize {
    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
        Value::Array(ref vs) => vs.len(),
        Value::Object(ref obj) => obj.len(),
    }
}

fn descend(v: &Value, idx: usize) -> Option<&Value> {
    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
        Value::Array(ref vs) => vs.get(idx),
        Value::Object(ref obj) => obj.keys().nth(idx).and_then(|key| obj.get(key)),
    }
}

fn write_data(
    stdout: &mut RawTerminal<Stdout>,
    value: &Value,
    highlighted: usize,
) -> std::io::Result<()> {
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    match value {
        Value::Null => write!(stdout, "null")?,
        Value::Bool(b) => write!(stdout, "{}", b)?,
        Value::Number(x) => write!(stdout, "{}", x)?,
        Value::String(s) => write!(stdout, "{}", s)?,
        Value::Array(ref vs) => write_fields(stdout, 0..vs.len(), highlighted)?,
        Value::Object(ref obj) => write_fields(stdout, obj.keys(), highlighted)?,
    }
    stdout.flush()?;
    Ok(())
}

fn write_fields<T: Display>(
    stdout: &mut RawTerminal<Stdout>,
    fields: impl Iterator<Item = T>,
    highlighted: usize,
) -> std::io::Result<()> {
    for (i, name) in fields.enumerate() {
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
        write!(stdout, "{}{}", name, style::Reset)?;
    }
    Ok(())
}

#[derive(StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    fn new(singleton: T) -> Self {
        Self(vec![singleton])
    }

    fn push(&mut self, item: T) {
        self.0.push(item);
    }
    fn pop(&mut self) -> Option<T> {
        if self.0.len() > 1 {
            self.0.pop()
        } else {
            None
        }
    }
    fn last(&self) -> &T {
        self.0.last().unwrap()
    }
    fn last_mut(&mut self) -> &mut T {
        self.0.last_mut().unwrap()
    }
}
