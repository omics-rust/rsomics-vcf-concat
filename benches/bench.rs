use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;

fn bench_vcf_concat(c: &mut Criterion) {
    let bin = env!("CARGO_BIN_EXE_rsomics-vcf-concat");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let chr1 = manifest.join("tests/golden/chr1.vcf");
    let chr2 = manifest.join("tests/golden/chr2.vcf");
    let chr3 = manifest.join("tests/golden/chr3.vcf");
    c.bench_function("rsomics-vcf-concat golden", |b| {
        b.iter(|| {
            let out = Command::new(black_box(bin))
                .args([
                    chr1.to_str().unwrap(),
                    chr2.to_str().unwrap(),
                    chr3.to_str().unwrap(),
                ])
                .output()
                .unwrap();
            assert!(out.status.success());
        });
    });
}

criterion_group!(benches, bench_vcf_concat);
criterion_main!(benches);
