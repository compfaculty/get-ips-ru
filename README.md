# ru-ip-dump

Fetch the public IP → ASN dataset from iptoasn.com, filter Russian (RU) allocations, sort by first IPv4 numerically, and write two outputs:

- `ru-ip-full.txt` — lines like: `FIRST-LAST ASN COUNTRY DESCRIPTION`
- `ru-ip-only.txt` — lines like: `FIRST-LAST`

The app downloads `https://iptoasn.com/data/ip2asn-combined.tsv.gz`, streams/decompresses it, keeps rows with `country == "RU"`, sorts by the first IP, and writes results.

---

## How to run

- Local (writes files into the current working directory):
  - cargo run --release
  - After completion, check the console message for the absolute paths of:
    - ru-ip-full.txt
    - ru-ip-only.txt

- Docker (writes files into ./out on your host):
  - make docker-build
  - make docker-run
  - The files will be in the ./out directory on your machine.

If you “don’t see any files”, verify your working directory and read the printed lines like:
  ru-ip-dump: will write outputs into: C:\path\to\dir (files: ru-ip-full.txt, ru-ip-only.txt)
  ru-ip-dump: wrote N rows to:
    C:\path\to\dir\ru-ip-full.txt
    C:\path\to\dir\ru-ip-only.txt

---

## Requirements

- Rust (stable) with Cargo
- Internet access (HTTPS)
- For Docker usage: Docker 20.10+


