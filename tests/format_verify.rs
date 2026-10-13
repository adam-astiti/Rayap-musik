//! Manual decoder verification for the mandatory audio formats.
//!
//! This test is ignored by default; run it with real files to prove the pinned
//! decoder stack actually decodes each format (not merely that the app builds).
//!
//! ```sh
//! RAYAP_AUDIO_FILES="/a.mp3:/b.flac:/c.m4a" cargo test --test format_verify -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use std::time::Duration;

#[test]
#[ignore]
fn verify_decodes_real_files() {
    let files: Vec<PathBuf> = std::env::var("RAYAP_AUDIO_FILES")
        .map(|value| value.split(':').map(PathBuf::from).collect())
        .unwrap_or_default();

    assert!(
        !files.is_empty(),
        "set RAYAP_AUDIO_FILES to a colon-separated list of audio file paths"
    );

    let mut failures = 0;
    for file in &files {
        match decode(file) {
            Ok(duration) => {
                println!("PASS  {:>8}  {}", duration_text(duration), file.display());
            }
            Err(error) => {
                failures += 1;
                println!("FAIL  {:>8}  {}  ({error})", "--", file.display());
            }
        }
    }

    assert_eq!(failures, 0, "{failures} file(s) failed to decode");
}

fn decode(path: &PathBuf) -> Result<Duration, String> {
    use rodio::{Decoder, Source};
    use std::fs::File;

    let file = File::open(path).map_err(|error| error.to_string())?;
    let source = Decoder::try_from(file).map_err(|error| error.to_string())?;
    let duration = source.total_duration();

    // Force a full decode (not just a header read) by consuming every sample.
    let _samples = source.count();

    duration.ok_or_else(|| "decoder reported no duration".to_string())
}

fn duration_text(duration: Duration) -> String {
    let total = duration.as_secs();
    format!("{:02}:{:02}", total / 60, total % 60)
}
