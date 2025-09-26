//! Library for parsing and writing RU IP allocation data from iptoasn.com
//!
//! Exposes small, testable building blocks so we can unit/integration-test
//! behavior without performing network I/O in tests.

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

/// One row of the iptoasn dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsnRow {
    pub first_ip: Ipv4Addr,
    pub last_ip: Ipv4Addr,
    pub number: u32,
    pub country: String,
    pub description: String,
}

impl AsnRow {
    /// Numeric key for sorting by the first IPv4 address.
    #[inline]
    pub fn first_ip_as_u32(&self) -> u32 {
        u32::from(self.first_ip)
    }

    /// Render a line for the "full" output file.
    pub fn format_full(&self) -> String {
        format!(
            "{}-{} {} {} {}",
            self.first_ip, self.last_ip, self.number, self.country, self.description
        )
    }

    /// Render a line for the "only" output file.
    pub fn format_only(&self) -> String {
        format!("{}-{}", self.first_ip, self.last_ip)
    }
}

/// Parse an IPv4 address, ignoring non-IPv4 inputs.
#[inline]
pub fn ip4(s: &str) -> Option<Ipv4Addr> {
    match IpAddr::from_str(s) {
        Ok(IpAddr::V4(v4)) => Some(v4),
        _ => None,
    }
}

/// Parse a single TSV line from iptoasn combined dataset.
///
/// Expected columns: `start_ip\tend_ip\tasn\tcountry\tdescription`
///
/// Returns Some(AsnRow) if the line is valid AND the country is exactly "RU".
pub fn parse_row(line: &str) -> Option<AsnRow> {
    let mut parts = line.splitn(5, '\t');

    let p0 = parts.next()?; // start_ip
    let p1 = parts.next()?; // end_ip
    let p2 = parts.next()?; // asn
    let p3 = parts.next()?; // country

    if p3 != "RU" {
        return None;
    }

    let p4 = parts.next().unwrap_or_default(); // description (may be empty)

    let first_ip = ip4(p0)?;
    let last_ip = ip4(p1)?;
    let number = p2.parse::<u32>().ok()?;

    Some(AsnRow {
        first_ip,
        last_ip,
        number,
        country: p3.to_string(),
        description: p4.to_string(),
    })
}

/// Iterate over gzipped TSV bytes and collect all RU rows.
/// This keeps memory usage reasonable by streaming the gzip and reading line-by-line.
pub fn collect_ru_rows_from_gzip_bytes(bytes: &[u8]) -> Vec<AsnRow> {
    let gz = GzDecoder::new(bytes);
    let reader = BufReader::new(gz);

    let mut rows: Vec<AsnRow> = Vec::with_capacity(100_000);
    for line in reader.lines() {
        if let Ok(l) = line {
            if let Some(row) = parse_row(&l) {
                rows.push(row);
            }
        }
    }
    rows
}

/// Write both output files using buffered I/O for performance.
pub fn write_outputs(rows: &[AsnRow]) -> Result<()> {
    let full_file = File::create("ru-ip-full.txt").context("create ru-ip-full.txt")?;
    let only_file = File::create("ru-ip-only.txt").context("create ru-ip-only.txt")?;
    let all_file = File::create("all_ip_ru.txt").context("create all_ip_ru.txt")?;

    let mut full = BufWriter::new(full_file);
    let mut only = BufWriter::new(only_file);
    let mut all = BufWriter::new(all_file);

    for r in rows {
        // Write the two summary files
        writeln!(full, "{}", r.format_full()).context("write full line")?;
        writeln!(only, "{}", r.format_only()).context("write only line")?;

        // Expand the range into individual IPv4s (inclusive)
        let start = u32::from(r.first_ip);
        let end = u32::from(r.last_ip);
        for n in start..=end {
            let ip = Ipv4Addr::from(n);
            writeln!(all, "{}", ip).context("write expanded ip line")?;
        }
    }

    // Ensure buffers are flushed to disk.
    full.flush().context("flush full")?;
    only.flush().context("flush only")?;
    all.flush().context("flush all")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip4_parsing() {
        assert_eq!(ip4("1.2.3.4").unwrap(), Ipv4Addr::new(1, 2, 3, 4));
        assert!(ip4("::1").is_none());
        assert!(ip4("not-an-ip").is_none());
    }

    #[test]
    fn test_parse_row_filters_country() {
        // Non-RU should be filtered out
        assert!(parse_row("1.1.1.0\t1.1.1.255\t1337\tUS\tCloud").is_none());

        // RU should parse
        let r = parse_row("5.255.192.0\t5.255.255.255\t13238\tRU\tYandex").unwrap();
        assert_eq!(r.number, 13238);
        assert_eq!(r.country, "RU");
        assert_eq!(r.first_ip, Ipv4Addr::new(5, 255, 192, 0));
        assert_eq!(r.last_ip, Ipv4Addr::new(5, 255, 255, 255));
    }

    #[test]
    fn test_formatters() {
        let r = AsnRow {
            first_ip: Ipv4Addr::new(5, 5, 5, 0),
            last_ip: Ipv4Addr::new(5, 5, 5, 255),
            number: 42,
            country: "RU".into(),
            description: "Example".into(),
        };
        assert_eq!(r.format_only(), "5.5.5.0-5.5.5.255");
        assert_eq!(r.format_full(), "5.5.5.0-5.5.5.255 42 RU Example");
    }
}
