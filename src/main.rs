use sound_label_info::SliFile;
use structopt::StructOpt;

use std::path::{Path, PathBuf};
use std::fs;

#[derive(StructOpt)]
struct Args {
    in_file: PathBuf,
    out_file: PathBuf,

    #[structopt(short, long)]
    labels: Option<PathBuf>,
}

fn main() {
    let args = Args::from_args();

    match SliFile::open(&args.in_file) {
        Ok(sli_file) => {
            let _ = sound_label_info::set_labels(
                args.labels.as_deref().unwrap_or(Path::new("Hashes.txt"))
            );

            fs::write(&args.out_file, serde_yaml::to_string(&sli_file).unwrap()).unwrap();
        }
        Err(sound_label_info::Error::BadMagic { .. }) => {
            // Magic doesn't match, should be yaml file

            let contents = fs::read_to_string(&args.in_file).unwrap();
            let sli_file: SliFile = serde_yaml::from_str(&contents).unwrap();

            sli_file.save(&args.out_file).unwrap();
        },
        Err(err) => {
            // Another error occurred, magic matches but failed to parse
            eprintln!("An error occurred: {}", err);
        }
    }
}
