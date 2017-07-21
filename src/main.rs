#![recursion_limit = "1024"]

extern crate attalus;

use attalus::emulation_manager::*;
use attalus::log::*;
use attalus::hardware::memory_mapper::implementation::*;

fn main() {
    let log = LogEverything::new(std::io::stdout());
    let mut args: Vec<String> = Vec::new();
    args.extend(std::env::args());
    if args.len() < 3 {
        println!("Usage: exec filename n");
        return;
    }
    let filename = &args[1];
    let smmh =
        <SegaMemoryMapperHardware as MemoryMapperHardware>::
            new_from_file(filename.clone(), 0x2000).unwrap();

    let mut em = EmulationManager::new(log, smmh);

    let n: u32 = args[2].parse().expect("Usage: exec filename n");

    main_loop(&mut em, n);

    // let mut z80: types::TestingZ80 = Default::default();
    // match env::args().nth(1) {
    //     Some(s) => {
    //         read_file(&mut z80.memory, s);
    //     }
    //     _ => panic!("provide a filename"),
    // }
    // for _ in 0..0x50000 {
    //     execute::execute1(&mut z80);
    //     println!("\n  {:?}", z80);
    // }
}
