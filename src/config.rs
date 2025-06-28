use core::str::FromStr;

use alloc::string::String;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uefi::{boot, fs::FileSystem, CString16, Guid, Handle};

#[derive(Serialize, Deserialize, Debug)]
pub enum FileSystemType {
	FAT
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BootConfig {
	guid: String,
	fs: FileSystemType,
	kernel: String
}

impl BootConfig {
	pub fn guid(&self) -> Guid {
		Guid::from_str(&self.guid).unwrap()
	}

	pub fn fs(&self) -> &FileSystemType {
		&self.fs
	}

	pub fn kernel(&self) -> &String {
		&self.kernel
	}
}

pub fn read_boot_config(image_handle: Handle) -> Result<BootConfig> {
	const CONFIG_PATH: &str = "primus.json";
	let mut fs = FileSystem::new(boot::get_image_file_system(image_handle)?);
	let boot_config = fs.read(CString16::try_from(CONFIG_PATH)?.as_ref())?;
	let boot_config = String::from_utf8_lossy(&boot_config);
	let boot_config = serde_json::from_str::<BootConfig>(&boot_config)?;
	Ok(boot_config)
}
