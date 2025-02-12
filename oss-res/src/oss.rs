use std::path::PathBuf;
use std::{fs::File, path::Path};

use std::io::Read;
use std::io::Write;

use crate::error::TransferError;
use crate::unzip::unzip_file;
use crate::Args;
use aliyun_oss_rust_sdk::oss::OSS;
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
    use aliyun_oss_rust_sdk::request::RequestBuilder;

    let oss: OSS = oss_config.clone().into();
    let build = RequestBuilder::new();

    let file_name = Path::new(&args.file).file_name().unwrap().to_str().unwrap();

    println!("Downloading {}...", file_name);

    let download_bin = oss
        .get_object(&args.file, build.clone())
        .map_err(|e| TransferError::OssError(format!("{}", e)))?;

    let output_path = PathBuf::from(&args.output).join(file_name);

    std::fs::create_dir_all(output_path.parent().unwrap())
        .map_err(|e| TransferError::Other(format!("Failed to create directory: {}", e)))?;

    let mut file = File::create(&output_path).map_err(|e| TransferError::Other(format!("{}", e)))?;

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

        std::fs::remove_file(&zip_path).map_err(|e| TransferError::Other(format!("{}", e)))?;

        println!("Unzipped {} successfully.", file_name);
    }

    Ok(())
}

#[test]
fn test_handle_oss() {
    handle_oss(
        Args {
            oss_config: "".into(),
            file: "/projectA/deploy.zip".into(),
            unzip: true,
            output: ".".into(),
        },
        OssConfig {
            oss_bucket: "cm-xxx".into(),
            oss_endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
            key_id: "xxx".into(),
            key_secret: "xxx".into(),
        },
    )
    .unwrap();
}

#[test]
fn write_json() {
    let config = OssConfig {
        oss_bucket: "cm-xxx".into(),
        oss_endpoint: "oss-cn-hangzhou.aliyuncs.com".into(),
        key_id: "xxx".into(),
        key_secret: "xxx".into(),
    };

    let json_str = serde_json::to_string(&config).unwrap();
    println!("{}", json_str);
}
