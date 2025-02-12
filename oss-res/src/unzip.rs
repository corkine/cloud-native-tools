use std::fs::File;
use std::path::Path;
use zip::ZipArchive;
use encoding_rs::{GBK, BIG5, UTF_8};
use std::str;

use crate::error::TransferError;

fn is_valid_filename(s: &str) -> bool {
    !s.contains('\0') && s.chars().all(|c| c.is_ascii() || c.is_alphabetic())
}

fn decode_filename(raw_name: &[u8]) -> String {
    // 首先尝试系统默认编码
    if let Ok(name) = str::from_utf8(raw_name) {
        if is_valid_filename(name) {
            return name.to_string();
        }
    }

    // 如果系统默认编码失败，尝试其他编码
    let encodings = [encoding_rs::WINDOWS_1252, GBK, BIG5, UTF_8];
    
    for encoding in &encodings {
        let (cow, _, had_errors) = encoding.decode(raw_name);
        if !had_errors && is_valid_filename(&cow) {
            return cow.into_owned();
        }
    }
    
    // 如果所有编码都失败，返回原始的字节作为字符串
    String::from_utf8_lossy(raw_name).into_owned()
}

pub fn unzip_file(zip_path: &Path, output_dir: &Path) -> Result<(), TransferError> {
    let file = File::open(zip_path).map_err(|e| TransferError::Other(format!("Failed to open zip file: {}", e)))?;
    let mut archive = ZipArchive::new(file).map_err(|e| TransferError::Other(format!("Failed to read zip: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| TransferError::Other(format!("Failed to read zip entry: {}", e)))?;
        
        let raw_name = file.name_raw();
        let file_name = decode_filename(raw_name);
        let outpath = output_dir.join(Path::new(&file_name));

        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| TransferError::Other(format!("Failed to create directory: {}", e)))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).map_err(|e| TransferError::Other(format!("Failed to create directory: {}", e)))?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| TransferError::Other(format!("Failed to create file: {}", e)))?;
            
            std::io::copy(&mut file, &mut outfile).map_err(|e| TransferError::Other(format!("Failed to write file: {}", e)))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))
                    .map_err(|e| TransferError::Other(format!("Failed to set permissions: {}", e)))?;
            }
        }
    }

    Ok(())
}
