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
    let mut pos = 0;
    let mut path = JsonPath::new(&root);

    let stdin = std::io::stdin();
    let mut stdout = HideCursor::from(std::io::stdout().into_raw_mode()?);

    write_data(&mut stdout, &path, pos)?;
    for evt in stdin.events() {
        let key = match evt? {
            Event::Key(key) => key,
            _ => continue,
        };
        match key {
            Key::Esc | Key::Char('q') => break,
            Key::Char('j') => {
                if pos + 1 < path.field_count() {
                    pos += 1;
                }
            }
            Key::Char('k') => {
                if pos > 0 {
                    pos -= 1;
                }
            }
            Key::Char('h') => {
                if let Some(prev) = path.pop() {
                    pos = prev;
                }
            }
            Key::Char('l') => {
                if path.push(pos) {
                    pos = 0;
                }
            }
            _ => continue,
        };
        write_data(&mut stdout, &path, pos)?;
    }

    // Restore the cursor and then exit.
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    Ok(())
}

fn write_data(
    stdout: &mut RawTerminal<Stdout>,
    path: &JsonPath,
    pos: usize,
) -> std::io::Result<()> {
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1))?;
    write!(stdout, "/")?;
    for name in path.names() {
        write!(stdout, "{}/", name)?;
    }
    write!(stdout, "{}", Goto(1, 3))?;
    match path.cur {
        Value::Null => write!(stdout, "null")?,
        Value::Bool(b) => write!(stdout, "{}", b)?,
        Value::Number(x) => write!(stdout, "{}", x)?,
        Value::String(s) => write!(stdout, "{}", s)?,
        Value::Array(ref vs) => write_fields(stdout, 0..vs.len(), pos)?,
        Value::Object(ref obj) => write_fields(stdout, obj.keys(), pos)?,
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
        write!(stdout, "{}", Goto(1, 3 + i as u16))?;
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

struct JsonPath<'a> {
    path: Vec<(&'a Value, usize)>,
    cur: &'a Value,
}
impl<'a> JsonPath<'a> {
    fn new(root: &Value) -> JsonPath {
        JsonPath {
            path: Vec::new(),
            cur: root,
        }
    }

    /// How many children does the current node have?
    fn field_count(&self) -> usize {
        match self.cur {
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
            Value::Array(vs) => vs.len(),
            Value::Object(ref obj) => obj.len(),
        }
    }

    /// Descend down the `idx`-th child.
    fn push(&mut self, idx: usize) -> bool {
        let next = match self.cur {
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
            Value::Array(ref vs) => vs.get(idx),
            Value::Object(ref obj) => obj.keys().nth(idx).and_then(|key| obj.get(key)),
        };
        match next {
            None => false,
            Some(next) => {
                self.path.push((self.cur, idx));
                self.cur = next;
                true
            }
        }
    }

    /// Back up one step, and return the index of the child we popped.
    fn pop(&mut self) -> Option<usize> {
        let (prev, idx) = self.path.pop()?;
        self.cur = prev;
        Some(idx)
    }

    /// An iterator over the field names on this path.
    fn names(&self) -> impl Iterator<Item = &dyn Display> {
        self.path.iter().filter_map(|(value, idx)| {
            let name: Option<&dyn Display> = match value {
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
                Value::Array(_) => Some(idx),
                Value::Object(ref obj) => obj.keys().nth(*idx).map(|s| {
                    let s: &dyn Display = s;
                    s
                }),
            };
            name
        })
    }
}
