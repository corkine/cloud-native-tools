mod error;
mod oss;
mod unzip;

use clap::Parser;
use error::TransferError;
use oss::{handle_oss, parse_oss_config};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Aliyun OSS Base64 Configuration or Configuration File Path
    #[clap(long)]
    oss_config: String,

    /// File URL, start with /
    #[clap(short, long)]
    file: String,

    /// Zip file
    #[clap(short, long)]
    unzip: bool,

    /// Output Dir, Default to .
    #[clap(short, long, default_value = ".")]
    output: String
}

fn main() -> Result<(), TransferError> {
    let args = Args::parse();
    if let Ok(oss_config) = parse_oss_config(&args.oss_config) {
        return Ok(handle_oss(args, oss_config)?)
    } else {
        let json_str = r#"
    {
        "oss_bucket": "my-bucket",
        "oss_endpoint": "oss-cn-beijing.aliyuncs.com",
        "key_secret": "your-secret-key",
        "key_id": "your-access-key-id"
    }
    "#;
        return Err(TransferError::Other(
            format!(
                "oss_config cannot be empty,
        you can put base64 encode json format like {json_str} 
        to use aliyun oss config, or just prove config's path")
            .into(),
        ));
    }
}
