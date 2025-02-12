use core::str;
use std::fs;
use std::path::PathBuf;
use std::{fs::File, path::Path};

use std::io::Read;
use std::io::Write;

use crate::error::TransferError;
use crate::unzip::unzip_file;
use crate::Args;
use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OssConfig {
    oss_bucket: String,
    oss_endpoint: String,
    key_secret: String,
    key_id: String,
}

impl From<OssConfig> for OSS {
    fn from(value: OssConfig) -> Self {
        OSS::new(
            value.key_id,
            value.key_secret,
            value.oss_endpoint.clone(),
            value.oss_bucket,
        )
    }
}

pub fn parse_oss_config(file_or_plain: &str) -> Result<OssConfig, TransferError> {
    if file_or_plain.is_empty() {
        return Err(TransferError::Other("Destination cannot be empty".into()));
    }

    // 尝试解码 base64
    if let Ok(decoded) = BASE64.decode(file_or_plain) {
        if let Ok(s) = std::str::from_utf8(&decoded) {
            return parse_oss_config_from_str(s);
        }
    }

    // 检查是否为文件路径
    if Path::new(file_or_plain).is_file() {
        let mut file = File::open(file_or_plain)
            .map_err(|e| TransferError::Other(format!("Failed to open file: {}", e)))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| TransferError::Other(format!("Failed to read file: {}", e)))?;
        return parse_oss_config_from_str(&content);
    }

    // 如果既不是 base64 也不是文件，则尝试直接解析 JSON
    parse_oss_config_from_str(file_or_plain)
}

fn parse_oss_config_from_str(s: &str) -> Result<OssConfig, TransferError> {
    serde_json::from_str(s).map_err(TransferError::JsonParseError)
}

pub fn handle_oss(args: Args, oss_config: OssConfig) -> Result<(), TransferError> {
    let oss: OSS = oss_config.clone().into();

    if !check_need_download(&oss, &args) {
        println!("Skipping download of {} as it already exists.", args.file);
        return Ok(());
    }

    let build = RequestBuilder::new();

    let file_name = Path::new(&args.file).file_name().unwrap().to_str().unwrap();

    println!("Downloading {}...", file_name);

    let download_bin = oss
        .get_object(&args.file, build.clone())
        .map_err(|e| TransferError::OssError(format!("{}", e)))?;

    let output_path = PathBuf::from(&args.output).join(file_name);

    std::fs::create_dir_all(output_path.parent().unwrap())
        .map_err(|e| TransferError::Other(format!("Failed to create directory: {}", e)))?;

    let mut file =
        File::create(&output_path).map_err(|e| TransferError::Other(format!("{}", e)))?;

    file.write_all(&download_bin)
        .map_err(|e| TransferError::Other(format!("Failed to write file: {}", e)))?;

    println!("Downloaded {} successfully.", file_name);

    if args.unzip {
        println!("Unzipping {}...", file_name);

        let zip_path = PathBuf::from(output_path);
        let output_dir = zip_path
            .parent()
            .unwrap_or(Path::new(&args.output))
            .to_path_buf();

        unzip_file(&zip_path, &output_dir)?;

        if !args.cache {
            std::fs::remove_file(&zip_path).map_err(|e| TransferError::Other(format!("{}", e)))?;
        }

        println!("Unzipped {} successfully.", file_name);
    }

    Ok(())
}

fn check_need_download(oss: &OSS, args: &Args) -> bool {
    if !args.cache {
        return true;
    }

    let download_file_name = Path::new(&args.file).file_name().unwrap().to_str().unwrap();
    let local_path = Path::new(&args.output).join(download_file_name);

    if !fs::metadata(&local_path).is_ok() {
        return true;
    }

    match (
        calculate_local_md5(&local_path.to_str().unwrap()),
        get_remote_md5(oss, &args.file),
    ) {
        (Some(local_md5), Some(remote_md5)) => local_md5 != remote_md5,
        _ => true,
    }
}

fn calculate_local_md5(file_path: &str) -> Option<String> {
    let mut local_file = fs::File::open(file_path).ok()?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer).ok()?;
    let local_md5 = md5::compute(&buffer);
    Some(format!("{:x}", local_md5))
}

fn get_remote_md5(oss: &OSS, file: &str) -> Option<String> {
    let build = RequestBuilder::new();
    match oss.get_object(format!("{}.md5", file), build) {
        Ok(response) => {
            let bytes = response.as_slice();
            str::from_utf8(bytes).map(|s| s.trim().to_string()).ok()
        }
        Err(_) => None,
    }
}

#[cfg(test)]
fn build_config() -> OssConfig {
    return OssConfig {
        oss_bucket: "cm-binary".into(),
        oss_endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
        key_id: "xxx".into(),
        key_secret: "xxx".into(),
    };
}

#[test]
fn file_md5() {
    let res = calculate_local_md5("temp2/deploy.zip".into());
    println!("{:?}", res);
}

#[test]
fn remote_md5() {
    let oss: OSS = build_config().clone().into();
    let res = get_remote_md5(&oss, "/projectA/deploy.zip");
    println!("{:?}", res);
}

#[test]
fn test_handle_oss() {
    handle_oss(
        Args {
            oss_config: "".into(),
            file: "/projectA/deploy.zip".into(),
            unzip: true,
            output: ".".into(),
            cache: true,
        },
        build_config(),
    )
    .unwrap();
}

#[test]
fn write_json() {
    let json_str = serde_json::to_string(&build_config()).unwrap();
    println!("{}", json_str);
}
