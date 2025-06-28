use alloc::vec::Vec;
use anyhow::Result;
use uefi::{boot::{self, OpenProtocolParams}, fs::FileSystem, proto::{device_path::{media::{HardDrive, PartitionFormat, PartitionSignature}, DevicePath}, media::fs::SimpleFileSystem}, CString16, Guid, Handle, Identify};

pub fn find_partition(image_handle: Handle, guid: Guid) -> Option<Handle> {
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

pub fn read_file(image_handle: Handle, partition_handle: Handle, filename: &str) -> Result<Vec<u8>> {
    let fs = unsafe {boot::open_protocol::<SimpleFileSystem>(OpenProtocolParams {
        handle: partition_handle,
        agent: image_handle,
        controller: None
    }, boot::OpenProtocolAttributes::GetProtocol)?};
    let mut fs = FileSystem::new(fs);
    Ok(fs.read(CString16::try_from(filename)?.as_ref())?)
}
