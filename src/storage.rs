use chrono::Datelike;
use sha256;
use std::path::Path;
use zstd;
async fn store_payload(payload: String, path: Box<Path>) {}

pub(crate) trait Storage {
    async fn store(&self, payload: &str) -> String;
}

pub(crate) struct LocalStorage<'a> {
    pub(crate) path: &'a Path,
}
impl LocalStorage<'_> {
    pub(crate) fn new(path: &'_ Path) -> LocalStorage<'_> {
        LocalStorage { path }
    }
}
impl Storage for LocalStorage<'_> {
    async fn store(&self, payload: &str) -> String {
        let timestamp = chrono::Utc::now();
        let bytes = zstd::bulk::compress(payload.as_bytes(), 3).unwrap();

        let hash = &sha256::digest(&bytes)[0..8];
        let primary_key = format!("{}_{}", timestamp.format("%Y-%m-%dT%H-%m-%S"), hash);
        let subpath = self
            .path
            .join(timestamp.year().to_string())
            .join(timestamp.month().to_string())
            .join(primary_key.as_str())
            .with_extension("json.zst");
        std::fs::create_dir_all(subpath.parent().unwrap()).expect("Directory creation failed.");
        tokio::fs::write(subpath, &bytes)
            .await
            .expect("Writing failed");
        primary_key
    }
}
#[cfg(test)]
mod tests {
    use crate::storage::{LocalStorage, Storage};
    use std::path::Path;

    #[tokio::test]
    async fn test_localstorage_store() {
        let payload = "a nice string";
        let store = LocalStorage {
            path: Path::new("/tmp/test/"),
        };
        let pk = store.store(payload).await;
    }
}
