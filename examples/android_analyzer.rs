use std::fs::File;
use android_hprof::hprof_parser::HprofParser;

fn main() {
    let file = File::open("resource/test.hprof");
    let file = match file {
        Ok(f) => f,
        Err(error) => {
            panic!("Problem creating the file: {:?}", error);
        }
    };
    HprofParser::parse(&file);
}