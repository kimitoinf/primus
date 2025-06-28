use core::ptr;

use alloc::vec::Vec;
use anyhow::Result;
use elf::{abi::PT_LOAD, endian::LittleEndian, ElfBytes};

pub fn load_kernel(kernel: Vec<u8>) -> Result<()> {
    const PRIMUS_HEADER: &str = ".primus_boot";
    let kernel_file = ElfBytes::<LittleEndian>::minimal_parse(kernel.as_slice())?;
    for program_header in kernel_file.segments().unwrap() {
        if program_header.p_type == PT_LOAD {
            let segment = kernel_file.segment_data(&program_header)?;
            let segment_addr = program_header.p_paddr as *mut u8;
            let segment_size = program_header.p_filesz as usize;
            let bss_size = (program_header.p_memsz - program_header.p_filesz) as usize;
            unsafe {
                ptr::copy_nonoverlapping(segment.as_ptr(), segment_addr, segment_size);
                if bss_size > 0 {
                    ptr::write_bytes(segment_addr.add(segment_size), 0, bss_size);
                }
            }
        }
    }
    Ok(())
}