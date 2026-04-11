use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use sdkwork_api_app_billing::CommercialBillingAdminKernel;
use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};

mod manifest;
mod registry;

#[cfg(test)]
pub(crate) use manifest::load_bootstrap_profile_pack;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct BootstrapLoadOutcome {
    pub applied: bool,
    pub data_root: Option<PathBuf>,
    pub profile_id: String,
    pub release_version: Option<String>,
    pub applied_update_ids: Vec<String>,
    pub applied_stage_count: usize,
    pub applied_record_count: usize,
}

pub(crate) async fn bootstrap_repository_data_from_config(
    store: &dyn AdminStore,
    account_kernel: &dyn AccountKernelStore,
    commercial_billing: &dyn CommercialBillingAdminKernel,
    config: &StandaloneConfig,
) -> Result<BootstrapLoadOutcome> {
    let profile_id = normalized_profile_id(&config.bootstrap_profile)?;
    let Some(data_root) = resolve_bootstrap_data_root(config, &profile_id)? else {
        return Ok(BootstrapLoadOutcome {
            applied: false,
            data_root: None,
            profile_id,
            release_version: None,
            applied_update_ids: Vec::new(),
            applied_stage_count: 0,
            applied_record_count: 0,
        });
    };

    let pack = manifest::load_bootstrap_profile_pack(&data_root, &profile_id)?;
    let summary =
        registry::apply_bootstrap_profile_pack(store, account_kernel, commercial_billing, &pack)
            .await
            .with_context(|| {
                format!(
                    "failed to apply bootstrap data profile {} from {}",
                    pack.profile_id,
                    pack.data_root.display()
                )
            })?;

    Ok(BootstrapLoadOutcome {
        applied: true,
        data_root: Some(pack.data_root),
        profile_id: pack.profile_id,
        release_version: pack.release_version,
        applied_update_ids: pack.update_ids,
        applied_stage_count: summary.applied_stage_count,
        applied_record_count: summary.applied_record_count,
    })
}

fn normalized_profile_id(profile_id: &str) -> Result<String> {
    let profile_id = profile_id.trim();
    if profile_id.is_empty() {
        bail!("bootstrap_profile must not be empty");
    }
    Ok(profile_id.to_owned())
}

fn resolve_bootstrap_data_root(
    config: &StandaloneConfig,
    profile_id: &str,
) -> Result<Option<PathBuf>> {
    if let Some(explicit_root) = config
        .bootstrap_data_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let root = PathBuf::from(explicit_root);
        let root = fs::canonicalize(&root).with_context(|| {
            format!(
                "configured bootstrap_data_dir {} does not exist",
                root.display()
            )
        })?;
        ensure_profile_manifest_exists(&root, profile_id)?;
        return Ok(Some(root));
    }

    for candidate in discover_bootstrap_data_roots() {
        if profile_manifest_path(&candidate, profile_id).is_file() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

fn ensure_profile_manifest_exists(data_root: &Path, profile_id: &str) -> Result<()> {
    if !data_root.is_dir() {
        bail!(
            "bootstrap data root {} is not a directory",
            data_root.display()
        );
    }

    let manifest_path = profile_manifest_path(data_root, profile_id);
    if !manifest_path.is_file() {
        bail!(
            "bootstrap profile manifest {} is missing",
            manifest_path.display()
        );
    }

    Ok(())
}

fn discover_bootstrap_data_roots() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        push_candidate_roots(&current_dir, &mut candidates);
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    push_candidate_roots(&manifest_dir, &mut candidates);

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            push_candidate_roots(exe_dir, &mut candidates);
        }
    }

    candidates
}

fn push_candidate_roots(start_dir: &Path, candidates: &mut Vec<PathBuf>) {
    let mut discovered = Vec::new();

    for ancestor in start_dir.ancestors() {
        let candidate = ancestor.join("data");
        if !candidate.exists() {
            continue;
        }

        if let Ok(candidate) = fs::canonicalize(candidate) {
            discovered.push(candidate);
        }
    }

    for candidate in discovered.into_iter().rev() {
        if candidates.iter().any(|existing| existing == &candidate) {
            continue;
        }
        candidates.push(candidate);
    }
}

pub(crate) fn profile_manifest_path(data_root: &Path, profile_id: &str) -> PathBuf {
    data_root
        .join("profiles")
        .join(format!("{profile_id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_DISCOVERY_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempDiscoveryRoot {
        path: PathBuf,
    }

    impl TempDiscoveryRoot {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "sdkwork-bootstrap-discovery-{label}-{}",
                TEMP_DISCOVERY_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            if path.exists() {
                fs::remove_dir_all(&path).unwrap();
            }
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDiscoveryRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn bootstrap_push_candidate_roots_prefers_outer_repository_data_before_nested_bin_data() {
        let temp_root = TempDiscoveryRoot::new("prefer-outer-data");
        let repo_root = temp_root.path.join("repo");
        let root_data = repo_root.join("data");
        let nested_bin_data = repo_root.join("bin").join("data");

        fs::create_dir_all(&root_data).unwrap();
        fs::create_dir_all(&nested_bin_data).unwrap();

        let mut candidates = Vec::new();
        push_candidate_roots(&repo_root.join("bin"), &mut candidates);

        assert_eq!(
            candidates,
            vec![
                fs::canonicalize(&root_data).unwrap(),
                fs::canonicalize(&nested_bin_data).unwrap()
            ]
        );
    }
}
