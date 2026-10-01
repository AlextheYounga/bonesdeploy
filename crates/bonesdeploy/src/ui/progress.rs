use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};

pub struct ProgressReader<R, F> {
    reader: R,
    total: u64,
    uploaded: u64,
    last_percentage: u8,
    report: F,
}

impl<R, F> ProgressReader<R, F>
where
    F: FnMut(u8),
{
    pub fn new(reader: R, total: u64, mut report: F) -> Self {
        report(0);
        Self { reader, total, uploaded: 0, last_percentage: 0, report }
    }

    fn report_progress(&mut self) {
        let percentage = if self.total == 0 {
            100
        } else {
            u8::try_from((u128::from(self.uploaded) * 100 / u128::from(self.total)).min(100)).unwrap_or(100)
        };
        if percentage != self.last_percentage {
            (self.report)(percentage);
            self.last_percentage = percentage;
        }
    }
}

impl<R, F> AsyncRead for ProgressReader<R, F>
where
    R: AsyncRead + Unpin,
    F: FnMut(u8) + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let filled_before = buffer.filled().len();
        let result = Pin::new(&mut self.reader).poll_read(context, buffer);
        if let Poll::Ready(Ok(())) = result {
            let bytes_read = buffer.filled().len() - filled_before;
            self.uploaded = self.uploaded.saturating_add(u64::try_from(bytes_read).unwrap_or(u64::MAX));
            self.report_progress();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use anyhow::{Context, Result};
    use tokio::io::AsyncReadExt;

    use super::ProgressReader;

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
}
