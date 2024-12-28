use chrono::{DateTime, Datelike, Utc};
use sha256;
use std::path::{Path, PathBuf};
use zstd;

pub(crate) struct PayloadInfo {
    timestamp: DateTime<Utc>,
    compressed_bytes: Vec<u8>,
    hash: String,
    key: String,
}
impl PayloadInfo {
    pub(crate) fn from_raw_bytes(raw_bytes: &[u8]) -> Self {
        let timestamp = chrono::Utc::now();
        let compressed_bytes = zstd::bulk::compress(&raw_bytes, 3).unwrap();

        let hash = sha256::digest(&compressed_bytes)[0..8].to_owned();
        let key = format!("{}_{}", timestamp.format("%Y-%m-%dT%H-%M-%S"), hash);

        PayloadInfo {
            timestamp,
            compressed_bytes,
            hash,
            key,
        }
    }
    fn primary_key(self) -> String {
        format!(
            "{}_{}",
            self.timestamp.format("%Y-%m-%dT%H-%m-%S"),
            self.hash
        )
    }
}

pub(crate) trait Storage {
    async fn store(&self, payload: &PayloadInfo);
}

// -------------------------------------- Storage backed by local disk ----------------
pub(crate) struct LocalStorage {
    pub(crate) path: PathBuf,
}
impl LocalStorage {
    pub(crate) fn new(path: &Path) -> Self {
        LocalStorage {
            path: path.to_path_buf(),
        }
    }
}

impl Storage for LocalStorage {
    async fn store(&self, payload: &PayloadInfo) {
        let native_key = self
            .path
            .join(payload.timestamp.year().to_string())
            .join(payload.timestamp.month().to_string())
            .join(payload.key.as_str())
            .with_extension("json.zst");
        std::fs::create_dir_all(native_key.parent().unwrap()).expect("Directory creation failed.");
        tokio::fs::write(native_key, &payload.compressed_bytes)
            .await
            .expect("Writing failed");
    }
}

// -------------------------- Storage backed by S3 -------------------------------------
// struct S3Storage {
//     bucket_name: String,
//     aws: aws_sdk_s3::Client,
// }
// impl Storage for S3Storage {
//     async fn store(&self, payload: &str) -> String {
//         let timestamp = chrono::Utc::now();
//         let bytes = zstd::bulk::compress(payload.as_bytes(), 3).unwrap();
//     }
// }

#[cfg(test)]
mod tests {
    use crate::storage::{LocalStorage, Storage};
    use std::path::Path;

    #[tokio::test]
    async fn test_localstorage_store() {
        let payload = "a nice string";
        let path = Path::new("/tmp/test/");
        let store = LocalStorage::new { path };
        let pk = store.store(payload).await;
    }
}
