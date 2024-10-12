use roead::byml::Byml;
use clap::Parser;
use std::io::Write;
use rsdb_patcher::merge_byml_raw;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MergeArgs {
    #[arg(short, value_parser)]
    base_file: PathBuf,
    #[arg(short, value_parser)]
    input_files: Vec<PathBuf>,
    #[arg(short, long, value_parser)]
    output_file: PathBuf,
}


fn main() -> anyhow::Result<()> {
    let args = MergeArgs::parse();
    let mut base = {
        let base_file = std::fs::File::open(args.base_file.clone())?;
        if args.base_file.extension().map(|c| c.to_str().unwrap().ends_with("zs")).unwrap_or(false) {
            let decomp = zstd::decode_all(base_file)?;
            Byml::from_binary(decomp)?
        } else {
            Byml::read(base_file)?
        }
    };
    for file in args.input_files.iter() {
        let patch = {
            let patch_file = std::fs::read_to_string(file)?;
            Byml::from_text(patch_file)?
        };
        merge_byml_raw(&mut base, &patch)?;
    }
    // todo: get endian info. s3 uses little endian
    let base_res = base.to_binary(roead::Endian::Little);
    if args.output_file.extension().map(|c| c.to_str().unwrap().ends_with("zs")).unwrap_or(false) {
        let out_file = std::fs::File::create(args.output_file)?;
        let mut encoder = zstd::Encoder::new(out_file, 3)?;
        encoder.write_all(&base_res)?;
    } else {
        std::fs::write(args.output_file, base.to_binary(roead::Endian::Little))?;
    }
    anyhow::Result::Ok(())
}
