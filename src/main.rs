use anyhow::{Context, Result};
use reqwest::Client;
use ru_ip_dump::{collect_ru_rows_from_gzip_bytes, write_outputs};
use std::env;
use std::path::PathBuf;

/// Source dataset: gzipped TSV with IPv4 ranges and ASNs
const DATA_URL: &str = "https://iptoasn.com/data/ip2asn-combined.tsv.gz";

#[tokio::main]
async fn main() -> Result<()> {
    let cwd: PathBuf = env::current_dir().context("determine current directory")?;
    println!(
        "ru-ip-dump: will write outputs into: {} (files: ru-ip-full.txt, ru-ip-only.txt, all_ip_ru.txt)",
        cwd.display()
    );

    // Build an HTTP client (TLS via rustls, HTTP/2 enabled in Cargo.toml)
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36")
        .build()
        .context("build reqwest client")?;

    println!("ru-ip-dump: downloading {} ...", DATA_URL);

    // Download the .gz file; reqwest handles HTTPS and HTTP/2
    let resp = client
        .get(DATA_URL)
        .send()
        .await
        .context("download tsv.gz")?
        .error_for_status()
        .context("non-200 response")?;

    // NOTE: We buffer the gzip in memory for simplicity. The subsequent parsing
    // is streaming (line-by-line) over the decompressor to bound memory usage.
    let bytes = resp.bytes().await.context("read body bytes")?;

    // Decompress and collect RU rows
    let mut rows = collect_ru_rows_from_gzip_bytes(&bytes);

    // Sort by first IPv4 numeric value
    rows.sort_by_key(|r| r.first_ip_as_u32());

    // Write outputs to files
    write_outputs(&rows)?;

    println!(
        "ru-ip-dump: wrote {} rows to:\n  {}\n  {}",
        rows.len(),
        cwd.join("ru-ip-full.txt").display(),
        cwd.join("ru-ip-only.txt").display()
    );

    Ok(())
}
