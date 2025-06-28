#![no_main]
#![no_std]

mod config;
mod fs;
mod kernel;

use uefi::{allocator, prelude::*};

extern crate alloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: allocator::Allocator = allocator::Allocator;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let image_handle = boot::image_handle();
    let boot_config = config::read_boot_config(image_handle).unwrap();
    let partition_handle = fs::find_partition(image_handle, boot_config.guid()).expect("No matched partition with GUID in boot config.");

    boot::stall(100000000);
    Status::SUCCESS
}
