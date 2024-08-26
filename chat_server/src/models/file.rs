use std::path::{Path, PathBuf};

use super::ChatFile;
use sha1::{Digest, Sha1};

impl ChatFile {
    pub fn new(filename: &str, data: &[u8]) -> Self {
        let ext = filename.split('.').last().unwrap_or("txt");
        let hash = Sha1::digest(data);
        Self {
            ext: ext.to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self, ws_id: u64) -> String {
        format!("/files/{}/{}", ws_id, self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}.{}", part1, part2, part3, self.ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_file_url_should_work() {
        let file = ChatFile::new("filename.txt", "hello".as_bytes());
        assert_eq!(file.ext, "txt");
        assert_eq!(
            file.url(1234),
            "/files/1234/aaf/4c6/1ddcc5e8a2dabede0f3b482cd9aea9434d.txt"
        );
    }
}
