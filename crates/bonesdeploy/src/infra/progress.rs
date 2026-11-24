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
