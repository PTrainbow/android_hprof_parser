use android_hprof::hprof_parser::HprofParser;
use anyhow::Result;
use std::fs::File;

fn main() -> Result<()> {
    let file = File::open("resource/test.hprof")?;
    let _ = HprofParser::parse(&file);
    Ok(())
}
