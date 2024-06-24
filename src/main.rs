use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::mem::offset_of;
use std::path::Path;
use std::fs::OpenOptions;

struct Patch {
    offset: u64,
    data: Vec<u8>,
}

const DATABIN_OFFSET: u64 = 0x6D50000;
const HD_VER_DATABIN_OFFSET: u64 = 0x6d60000;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        panic!("Incorrect number of arguments. Usage 'p3rd-patcher <version> <path to iso> <path to patch>'")
    }

    let version = match args[1].to_lowercase().as_str() {
        "h" => "h",
        "n" => "n",
        _ => panic!("Invalid version. Try using 'h' for HD or 'n' for normal versions."),
    };

    let isopath = &args[2];
    let patchpath = &args[3];

    let patch: Vec<Patch> = read_patch(&patchpath);

    let iso = match OpenOptions::new().read(true).write(true).open(isopath) {
            Ok(file) => file,
            Err(_) => panic!("Could not open iso."),
    };
    
    apply_patch(&patch, &iso, &version);
}

fn apply_patch(patch: &Vec<Patch>, mut iso: &File, version: &str) {
    let offset: u64 = if version == "n" { DATABIN_OFFSET } else { HD_VER_DATABIN_OFFSET };
    for file in patch {
        let _ = iso.seek(std::io::SeekFrom::Start(&file.offset+offset)).unwrap();
        iso.write(&file.data).unwrap();
    }
}

fn read_patch(path: &str) -> Vec<Patch> {
    let mut data: Vec<Patch> = Vec::new();
    
    let mut file = match File::open(Path::new(path)) {
        Ok(file) => file,
        Err(_) => panic!("Could not open patch."),
    };

    let file_cnt: u64 = read_int(&file);

    let mut addr: u64 = (file_cnt + 1) * 8;
    if addr % 16 > 0 {
        addr = addr + 16 - (addr % 16);
    }

    for i in 0..file_cnt {
        let _ = file.seek(std::io::SeekFrom::Start(4+i*8));

        let offset = read_int(&file);
        let length = read_int(&file);

        let mut bytes = Vec::new();
        bytes.resize(length.try_into().unwrap(), 0);
        let _ = file.seek(std::io::SeekFrom::Start(addr)).unwrap();

        let _ = file.read(&mut bytes).unwrap();

        addr = file.seek(std::io::SeekFrom::Current(0)).unwrap();

        data.push(Patch {offset: offset, data: bytes});
    };

    data
}

fn read_int(mut file: &File) -> u64 {
    let mut data: [u8; 4] = [0, 0, 0, 0];
    let _ = file.read(&mut data);
    u64::try_from(u32::from_le_bytes(data)).unwrap()
}
