use android_hprof::hprof_parser::HprofParser;
use anyhow::Result;

fn main() -> Result<()> {
    HprofParser::parse("resource/test.hprof")?;
    Ok(())
}
