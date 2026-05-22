use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-vcf-concat"))
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn bcftools_path() -> Option<String> {
    let candidates = [
        "bcftools",
        "/opt/homebrew/Caskroom/miniforge/base/envs/imotif-pipeline/bin/bcftools",
        "/usr/bin/bcftools",
        "/usr/local/bin/bcftools",
    ];
    for candidate in &candidates {
        let ok = Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Data (non-header) records only — bcftools adds its own header lines.
fn records(vcf: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(vcf)
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Concatenation order and data records must match `bcftools concat` on the golden fixtures.
///
/// bcftools concat stamps extra header lines; comparison is on data records only.
#[test]
fn concat_matches_bcftools() {
    let Some(bcftools) = bcftools_path() else {
        eprintln!("skipping: bcftools not found");
        return;
    };

    let version = Command::new(&bcftools)
        .arg("--version")
        .output()
        .unwrap()
        .stdout;
    eprintln!(
        "bcftools: {}",
        String::from_utf8_lossy(&version)
            .lines()
            .next()
            .unwrap_or("")
    );

    let dir = fixtures_dir();
    let chr1 = dir.join("chr1.vcf");
    let chr2 = dir.join("chr2.vcf");
    let chr3 = dir.join("chr3.vcf");

    let ours_out = ours().arg(&chr1).arg(&chr2).arg(&chr3).output().unwrap();
    assert!(
        ours_out.status.success(),
        "rsomics-vcf-concat failed: {}",
        String::from_utf8_lossy(&ours_out.stderr)
    );

    let theirs = Command::new(&bcftools)
        .args(["concat"])
        .arg(&chr1)
        .arg(&chr2)
        .arg(&chr3)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .unwrap();
    assert!(theirs.status.success(), "bcftools concat failed");

    let ours_records = records(&ours_out.stdout);
    let their_records = records(&theirs.stdout);

    assert_eq!(
        ours_records, their_records,
        "data records differ from bcftools concat"
    );
}

/// Two-file concat: chr1 + chr2 only.
#[test]
fn concat_two_files_matches_bcftools() {
    let Some(bcftools) = bcftools_path() else {
        eprintln!("skipping: bcftools not found");
        return;
    };

    let dir = fixtures_dir();
    let chr1 = dir.join("chr1.vcf");
    let chr2 = dir.join("chr2.vcf");

    let ours_out = ours().arg(&chr1).arg(&chr2).output().unwrap();
    assert!(
        ours_out.status.success(),
        "rsomics-vcf-concat failed: {}",
        String::from_utf8_lossy(&ours_out.stderr)
    );

    let theirs = Command::new(&bcftools)
        .args(["concat"])
        .arg(&chr1)
        .arg(&chr2)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .unwrap();
    assert!(theirs.status.success(), "bcftools concat failed");

    assert_eq!(
        records(&ours_out.stdout),
        records(&theirs.stdout),
        "data records differ from bcftools concat (two-file)"
    );
}
