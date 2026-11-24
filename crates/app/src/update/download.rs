use super::{Package, Version, manifest};
use aws_lc_rs::digest::{Context, SHA256};
use futures::AsyncReadExt;
use http_client::{AsyncBody, HttpClient};
use std::{fs::File, io::Write, path::Path, sync::Arc};

pub const RELEASES: &str = "https://github.com/Ziuziuk-Denys/Flexolotl/releases";
const LARGEST_NOTE: u64 = 64 * 1024;
const LARGEST_PACKAGE: u64 = 200 * 1024 * 1024;
const PIECE: usize = 64 * 1024;

pub async fn newer_package(client: &Arc<dyn HttpClient>, suffix: &str) -> Option<Package> {
    let sums = note(client, "SHA256SUMS").await?;
    let signature = note(client, "SHA256SUMS.sig").await?;
    manifest::find_package(&sums, &signature, suffix)
        .filter(|package| package.version > Version::current())
}

pub async fn fetch(client: &Arc<dyn HttpClient>, package: &Package, path: &Path) -> Option<()> {
    let url = format!("{RELEASES}/download/v{}/{}", package.version, package.name);
    let response = client.get(&url, AsyncBody::empty(), true).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let mut body = response.into_body().take(LARGEST_PACKAGE);
    let mut file = File::create(path).ok()?;
    let mut checksum = Context::new(&SHA256);
    let mut piece = vec![0; PIECE];
    loop {
        let count = body.read(&mut piece).await.ok()?;
        if count == 0 {
            break;
        }
        checksum.update(&piece[..count]);
        file.write_all(&piece[..count]).ok()?;
    }
    file.sync_all().ok()?;
    (checksum.finish().as_ref() == package.checksum).then_some(())
}

async fn note(client: &Arc<dyn HttpClient>, name: &str) -> Option<Vec<u8>> {
    let url = format!("{RELEASES}/latest/download/{name}");
    let response = client.get(&url, AsyncBody::empty(), true).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let mut bytes = Vec::new();
    let body = response.into_body();
    body.take(LARGEST_NOTE).read_to_end(&mut bytes).await.ok()?;
    Some(bytes)
}
