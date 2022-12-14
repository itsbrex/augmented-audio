use crate::manifests::{AugmentedMetadata, CargoTomlPackageMetadata};
use crate::services::ListCratesService;
use sha3::{Digest, Sha3_256};
use std::fs::DirEntry;
use std::path::Path;

fn hash_entry(
    root_directory: &Path,
    hasher: &mut impl Digest,
    entry: std::io::Result<DirEntry>,
) -> anyhow::Result<()> {
    let entry = entry?;
    if entry.file_type()?.is_dir() {
        if entry.path().parent().unwrap().eq(root_directory) && entry.file_name() == "target" {
            return Ok(());
        }

        log::debug!("-- Recursive scan: {:?}", entry);
        get_package_content_hash_inner(root_directory, &entry.path(), hasher)?;
    } else {
        let contents = std::fs::read(entry.path())?;
        let path = entry.path();
        let file_entry_path = &path
            .strip_prefix(root_directory)?
            .to_str()
            .map(Ok)
            .unwrap_or_else(|| Err(anyhow::format_err!("Failed to render path")))?;
        log::debug!("Adding entry: {:?}", file_entry_path);
        hasher.update(file_entry_path);
        hasher.update(&contents);
    }

    Ok(())
}

fn get_package_content_hash_inner(
    root_directory: &Path,
    directory: &Path,
    hasher: &mut impl Digest,
) -> anyhow::Result<()> {
    let entries = std::fs::read_dir(directory)?;

    for entry in entries {
        hash_entry(root_directory, hasher, entry)?;
    }

    Ok(())
}

fn get_package_content_hash(root_directory: &Path) -> anyhow::Result<String> {
    let mut hasher = Sha3_256::new();

    get_package_content_hash_inner(root_directory, root_directory, &mut hasher)?;

    let result = hasher.finalize();
    let result = hex::encode(result);
    Ok(result)
}

pub fn get_all_package_hashes(list_crates_service: &ListCratesService) -> anyhow::Result<()> {
    let crates = list_crates_service.find_augmented_crates();
    for (crate_path, crate_toml) in crates {
        let crate_path = Path::new(&crate_path);
        if let Some(CargoTomlPackageMetadata {
            augmented:
                Some(AugmentedMetadata {
                    private: Some(false),
                    ..
                }),
            ..
        }) = crate_toml.package.metadata
        {
            let hash = get_package_content_hash(crate_path).map_err(|err| {
                anyhow::format_err!("Failed to hash package={:?} : {:?}", crate_path, err)
            })?;
            println!(
                "{}@{} {}",
                crate_toml.package.name, crate_toml.package.version, hash
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::services::content_hashes::{get_all_package_hashes, get_package_content_hash};
    use crate::services::ListCratesService;
    use log::LevelFilter;
    use std::env::set_current_dir;
    use std::path::Path;

    #[test]
    fn test_get_hash() {
        let hash = get_package_content_hash(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        println!("{}", hash);
    }

    #[test]
    fn test_get_all_hashes() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .try_init();
        let list_crates_service = ListCratesService::default();
        set_current_dir("../../../../").unwrap();
        get_all_package_hashes(&list_crates_service).unwrap();
    }
}
