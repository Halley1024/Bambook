use std::{fs, path::Path};

use serde::{de::DeserializeOwned, Serialize};

use crate::{error::AppResult, infrastructure::filesystem::write_bytes_atomically};

pub(crate) fn read_optional_json<T: DeserializeOwned>(path: &Path) -> AppResult<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

pub(crate) fn write_pretty_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> AppResult<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut output = bytes;
    output.push(b'\n');
    write_bytes_atomically(path, &output)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde::{Deserialize, Serialize};

    use super::{read_optional_json, write_pretty_json};

    #[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
    struct Fixture {
        revision: u64,
        title: String,
    }

    #[test]
    fn atomically_creates_and_replaces_json() {
        let root = test_root();
        let path = root.join("nested/settings.json");
        write_pretty_json(
            &path,
            &Fixture {
                revision: 1,
                title: "旧值".into(),
            },
        )
        .unwrap();
        write_pretty_json(
            &path,
            &Fixture {
                revision: 2,
                title: "Bambook".into(),
            },
        )
        .unwrap();

        let stored: Fixture = read_optional_json(&path).unwrap().unwrap();
        assert_eq!(
            stored,
            Fixture {
                revision: 2,
                title: "Bambook".into()
            }
        );
        let leftovers = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(leftovers, 0);
        fs::remove_dir_all(root).unwrap();
    }

    fn test_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "bambook-json-store-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
