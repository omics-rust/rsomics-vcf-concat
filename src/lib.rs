use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

fn open_reader(path: &Path) -> Result<Box<dyn Read>> {
    let raw = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut probe = [0u8; 2];
    let mut raw2 = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let n = raw2.read(&mut probe).map_err(RsomicsError::Io)?;
    drop(raw2);
    if n == 2 && probe == [0x1f, 0x8b] {
        Ok(Box::new(flate2::read::MultiGzDecoder::new(raw)))
    } else {
        Ok(Box::new(raw))
    }
}

/// Concatenate multiple VCFs that share the same sample set.
///
/// Emits the header from the first file, then all data records from each file
/// in the given order — matching `bcftools concat` default (ordered) behaviour.
/// Records are streamed and passed through unchanged.
pub fn concat_vcfs(inputs: &[&Path], output: &mut dyn io::Write) -> Result<u64> {
    let mut out = BufWriter::new(output);
    let mut total_records: u64 = 0;
    let mut header_written = false;

    for (idx, &path) in inputs.iter().enumerate() {
        let reader = open_reader(path)?;
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = buf_reader.read_line(&mut line).map_err(RsomicsError::Io)?;
            if n == 0 {
                break;
            }
            let trimmed = line.trim_end_matches(['\n', '\r']);
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('#') {
                // Emit header only from the first file.
                if idx == 0 {
                    out.write_all(trimmed.as_bytes())
                        .map_err(RsomicsError::Io)?;
                    out.write_all(b"\n").map_err(RsomicsError::Io)?;
                    header_written = true;
                }
            } else {
                if !header_written {
                    return Err(RsomicsError::InvalidInput(format!(
                        "{}: data record before any header line",
                        path.display()
                    )));
                }
                out.write_all(trimmed.as_bytes())
                    .map_err(RsomicsError::Io)?;
                out.write_all(b"\n").map_err(RsomicsError::Io)?;
                total_records += 1;
            }
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(total_records)
}
