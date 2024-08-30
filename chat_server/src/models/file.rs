use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::error::AppError;

use super::ChatFile;
use sha1::{Digest, Sha1};

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let ext = filename.split('.').last().unwrap_or("txt");
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: ext.to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError("Invalid file url".to_string()));
        };
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError("Invalid file url".to_string()));
        }
        let ws_id = parts[0]
            .parse::<u64>()
            .map_err(|_| AppError::ChatFileError("Invalid ws_id in parse ChatFile".to_string()))?;
        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError("Invalid file url".to_string()));
        };
        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_file_url_should_work() {
        let file = ChatFile::new(1234, "filename.txt", "hello".as_bytes());
        assert_eq!(file.ext, "txt");
        assert_eq!(
            file.url(),
            "/files/1234/aaf/4c6/1ddcc5e8a2dabede0f3b482cd9aea9434d.txt"
        );
    }
}
