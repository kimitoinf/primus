#![no_main]
#![no_std]

use core::{ptr, str::FromStr};

use alloc::{string::String, vec::Vec};
use anyhow::Result;
use elf::{abi::PT_LOAD, endian::LittleEndian, segment::ProgramHeader, ElfBytes};
use serde::{Deserialize, Serialize};
use uefi::{allocator, boot::OpenProtocolParams, fs::FileSystem, prelude::*, proto::{device_path::{media::{HardDrive, PartitionFormat, PartitionSignature}, DevicePath}, media::fs::SimpleFileSystem}, CString16, Guid, Identify};

extern crate alloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: allocator::Allocator = allocator::Allocator;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let image_handle = boot::image_handle();
    let boot_config = read_boot_config(image_handle).unwrap();
    let partition_handle = find_partition(image_handle, boot_config.guid()).expect("No matched partition with GUID in boot config.");

    boot::stall(100000000);
    Status::SUCCESS
}

#[derive(Serialize, Deserialize, Debug)]
enum FileSystemType {
    FAT
}

#[derive(Serialize, Deserialize, Debug)]
struct BootConfig {
    guid: String,
    fs: FileSystemType,
    kernel: String
}

impl BootConfig {
    fn guid(&self) -> Guid {
        Guid::from_str(&self.guid).unwrap()
    }

    fn fs(&self) -> &FileSystemType {
        &self.fs
    }

    fn kernel(&self) -> &String {
        &self.kernel
    }
}

fn read_boot_config(image_handle: Handle) -> Result<BootConfig> {
    const CONFIG_PATH: &str = "primus.json";
    let mut fs = FileSystem::new(boot::get_image_file_system(image_handle)?);
    let boot_config = fs.read(CString16::try_from(CONFIG_PATH)?.as_ref())?;
    let boot_config = String::from_utf8_lossy(&boot_config);
    let boot_config = serde_json::from_str::<BootConfig>(&boot_config)?;
    Ok(boot_config)
}

fn find_partition(image_handle: Handle, guid: Guid) -> Option<Handle> {
    let handles = boot::locate_handle_buffer(boot::SearchType::ByProtocol(&DevicePath::GUID)).unwrap();
    for &handle in handles.iter() {
        let protocol = match unsafe {boot::open_protocol::<DevicePath>(boot::OpenProtocolParams {
            handle: handle,
            agent: image_handle,
            controller: None
        }, boot::OpenProtocolAttributes::GetProtocol)} {
            Ok(p) => p,
            Err(_) => continue
        };
        for node in protocol.node_iter() {
            if let Ok(hard_drive_node) = <&HardDrive>::try_from(node) {
                if hard_drive_node.partition_format() == PartitionFormat::GPT {
                    if let PartitionSignature::Guid(signature) = hard_drive_node.partition_signature() {
                        if signature == guid {
                            return Some(handle);
                        }
                    }
                }
            }
        }
    }
    None
}

fn read_file(image_handle: Handle, partition_handle: Handle, filename: &str) -> Result<Vec<u8>> {
    let fs = unsafe {boot::open_protocol::<SimpleFileSystem>(OpenProtocolParams {
        handle: partition_handle,
        agent: image_handle,
        controller: None
    }, boot::OpenProtocolAttributes::GetProtocol)?};
    let mut fs = FileSystem::new(fs);
    Ok(fs.read(CString16::try_from(filename)?.as_ref())?)
}

fn load_kernel(kernel: Vec<u8>) -> Result<()> {
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
