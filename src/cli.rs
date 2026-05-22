use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_concat::concat_vcfs;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-concat",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input VCF files (plain or .vcf.gz). Must share identical sample columns.
    #[arg(value_name = "INPUT", required = true, num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(BufWriter::new(std::io::stdout().lock()))
        } else {
            Box::new(BufWriter::new(
                std::fs::File::create(&self.output).map_err(RsomicsError::Io)?,
            ))
        };

        let paths: Vec<&std::path::Path> = self.inputs.iter().map(|p| p.as_path()).collect();
        let n = concat_vcfs(&paths, &mut out)?;

        if !self.common.quiet {
            eprintln!("{n} records concatenated");
        }

        Ok(())
    }
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        self.execute()
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Concatenate VCFs (same samples) — Rust port of bcftools concat.",
    origin: Some(Origin {
        upstream: "bcftools concat",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/gigascience/giab008"),
    }),
    usage_lines: &["[OPTIONS] <INPUT.vcf>..."],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "INPUT",
                aliases: &[],
                value: Some("<path>..."),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "One or more input VCF files (plain or gzip-compressed). \
                              All inputs must carry the same sample columns in the same order.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: Some("-"),
                description: "Output VCF file (default: stdout).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Concatenate two per-chromosome VCFs to stdout",
            command: "rsomics-vcf-concat chr1.vcf chr2.vcf",
        },
        Example {
            description: "Concatenate and write to a file",
            command: "rsomics-vcf-concat chr1.vcf chr2.vcf -o combined.vcf",
        },
    ],
    json_result_schema_doc: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
