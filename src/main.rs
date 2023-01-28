use std::error::Error;
use std::ffi::OsString;
use std::fs::read;
use std::process::exit;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os();
    if args.len() != 3 {
        eprintln!("Usage: {} infile.dat outfile.dat",
                  args.next().unwrap_or_else(|| "sir0_relocator".parse().unwrap()).to_string_lossy());
        exit(1);
    }

    args.next();
    let mut in_bytes = read(args.next().unwrap())?;
    let out_path: OsString = args.next().unwrap();

    if in_bytes[0..4] != [b'S', b'I', b'R', b'0'] {
        println!("Bad file magic {:?}", &in_bytes[0..4]);
        exit(1);
    }
    in_bytes[3] = b'O';

    let offsets_addr: usize = u32::from_le_bytes(in_bytes[8..12].try_into().unwrap()).try_into()?;

    decode_offset_table(&mut in_bytes, offsets_addr);

    std::fs::write(out_path, in_bytes)?;

    Ok(())
}

fn decode_offset_table(bytes: &mut [u8], mut table_addr: usize) {
    let mut file_pos = 0usize;
    loop {
        let mut offset = 0u32;
        while bytes[table_addr] & 0x80 != 0 {
            offset |= u32::from(bytes[table_addr] & 0x7F);
            offset = offset.wrapping_shl(7);
            table_addr += 1;
        }
        offset |= u32::from(bytes[table_addr] & 0x7F);
        table_addr += 1;

        if offset == 0 {
            break;
        }

        file_pos = file_pos.wrapping_add(offset.try_into().unwrap());
        // println!("Adjusting pointer at {}", file_pos);

        assert_eq!(file_pos % 4, 0);
        let mut word = u32::from_le_bytes(bytes[file_pos..file_pos+4].try_into().unwrap());
        word += 0xCCCC0000;
        bytes[file_pos..file_pos+4].copy_from_slice(word.to_le_bytes().as_slice());
    }
}