use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();
    let data: serde_json::Value = {
        let fin = std::fs::File::open(args.input)?;
        let reader = BufReader::new(fin);
        serde_json::from_reader(reader)?
    };
    println!("{:?}", data);
    Ok(())
}

#[derive(StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}
