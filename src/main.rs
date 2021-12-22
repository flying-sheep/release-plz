use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::Command,
};

use cargo_metadata::Package;
use semver::Version;

#[derive(Debug)]
struct LocalPackage {
    package: Package,
    next_version: Option<Version>,
    hash: String,
    done: bool,
}

#[derive(Debug)]
struct RemotePackage {
    package: Package,
    hash: String,
}

fn calculate_local_crates(
    crates: impl Iterator<Item = Package>,
) -> anyhow::Result<HashMap<PathBuf, LocalPackage>> {
    crates
        .map(|c| {
            let mut manifest_path = c.manifest_path.clone();
            manifest_path.pop();
            let crate_path: PathBuf = manifest_path.into_std_path_buf();
            let hash = hash_dir(&crate_path)?;
            let local_package = LocalPackage {
                package: c,
                next_version: None,
                hash,
                done: false,
            };
            Ok((crate_path, local_package))
        })
        .collect()
}

fn calculate_remote_crates(
    crates: impl Iterator<Item = Package>,
) -> anyhow::Result<HashMap<PathBuf, RemotePackage>> {
    crates
        .map(|c| {
            let mut manifest_path = c.manifest_path.clone();
            manifest_path.pop();
            let crate_path: PathBuf = manifest_path.into_std_path_buf();
            let hash = hash_dir(&crate_path)?;
            let remote_package = RemotePackage { package: c, hash };
            Ok((crate_path, remote_package))
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    install_dependencies()?;
    // TODO download in tmp directory
    //download_crate("rust-gh-example")?;
    let local_crates = list_crates(&PathBuf::from(
        "/home/marco/me/proj/rust-gh-example2/Cargo.toml",
    ));
    let remote_crates = list_crates(&PathBuf::from(
        "/home/marco/me/proj/rust-gh-example/Cargo.toml",
    ));
    dbg!(&remote_crates);
    let local_crates = calculate_local_crates(local_crates.into_iter())?;
    let remote_crates = calculate_remote_crates(remote_crates.into_iter())?;
    dbg!(&local_crates);
    dbg!(&remote_crates);
    // pr command:
    // - go back commit by commit and for every local crate:
    //   - If the local crate was edited in that commit:
    //     - if the hash of that crate is the same of the remote crate, that local crate is done.
    //     - otherwise:
    //       - add the entry to the changelog of that crate.
    //       - bump the version of that crate according to the semantic versioning of the commit.
    // - raise PR

    // release command (probably this is already done in ):
    // - for every local_crate with a version != remote one:
    //   - publish crate
    //   - create a new tag with format `local_crate v*new_version*`
    // // Maybe the same or similar is done by :
    // // cargo workspaces publish  --from-git --token "${TOKEN}" --yes
    Ok(())
}

fn install_dependencies() -> anyhow::Result<()> {
    for program in ["cargo-workspaces", "cargo-clone", "sha1dir"] {
        Command::new("cargo").args(["install", program]).output()?;
    }
    Ok(())
}

fn list_crates(directory: &Path) -> Vec<Package> {
    cargo_edit::workspace_members(Some(directory)).unwrap()
}

fn download_crate(crate_name: &str) -> anyhow::Result<()> {
    Command::new("cargo").args(["clone", crate_name]).output()?;
    Ok(())
}

fn hash_dir(dir: impl AsRef<Path>) -> anyhow::Result<String> {
    let output = Command::new("sha1dir").arg(dir.as_ref()).output()?;
    let output = String::from_utf8(output.stdout)?;
    let sha1 = output
        .split(' ')
        .into_iter()
        .next()
        .expect("cannot calculate hash");

    Ok(sha1.to_string())
}
