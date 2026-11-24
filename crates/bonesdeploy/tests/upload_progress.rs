use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use tokio::io::AsyncReadExt;

use bonesdeploy::infra::progress::ProgressReader;

#[tokio::test]
async fn reports_upload_percentage_as_the_reader_is_consumed() -> Result<()> {
    let percentages = Arc::new(Mutex::new(Vec::new()));
    let reports = Arc::clone(&percentages);
    let mut reader = ProgressReader::new(&b"abcdefghij"[..], 10, move |percentage| {
        reports.lock().unwrap_or_else(|error| error.into_inner()).push(percentage);
    });

    let mut uploaded = Vec::new();
    for _ in 0..5 {
        let mut chunk = [0; 2];
        reader.read_exact(&mut chunk).await?;
        uploaded.extend_from_slice(&chunk);
    }

    assert_eq!(uploaded, b"abcdefghij");
    assert_eq!(
        percentages.lock().map_err(|error| anyhow::anyhow!("progress lock poisoned: {error}"))?.as_slice(),
        [0, 20, 40, 60, 80, 100]
    );
    Ok(())
}

#[tokio::test]
async fn reports_only_the_percentage_reached_by_a_short_reader() -> Result<()> {
    let percentages = Arc::new(Mutex::new(Vec::new()));
    let reports = Arc::clone(&percentages);
    let mut reader = ProgressReader::new(&b"short"[..], 10, move |percentage| {
        reports.lock().unwrap_or_else(|error| error.into_inner()).push(percentage);
    });

    reader.read_to_end(&mut Vec::new()).await.context("failed to consume progress reader")?;

    assert_eq!(
        percentages.lock().map_err(|error| anyhow::anyhow!("progress lock poisoned: {error}"))?.as_slice(),
        [0, 50]
    );
    Ok(())
}
