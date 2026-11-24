use futures::AsyncRead;
use std::{
    io,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

const PIECE: usize = 16 * 1024;

#[derive(Debug, Clone, Default)]
pub struct Progress {
    sent: Arc<AtomicUsize>,
    total: usize,
}

impl Progress {
    pub fn of(total: usize) -> Self {
        Self {
            sent: Arc::default(),
            total,
        }
    }

    pub fn share(&self) -> f32 {
        if self.total == 0 {
            return 0.;
        }
        self.sent.load(Ordering::Relaxed) as f32 / self.total as f32
    }
}

pub struct CountedBody {
    bytes: Arc<Vec<u8>>,
    position: usize,
    progress: Progress,
}

impl CountedBody {
    pub fn new(bytes: Arc<Vec<u8>>, progress: Progress) -> Self {
        Self {
            bytes,
            position: 0,
            progress,
        }
    }
}

impl AsyncRead for CountedBody {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let left = self.bytes.len() - self.position;
        let count = left.min(buffer.len()).min(PIECE);
        let end = self.position + count;
        buffer[..count].copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        self.progress.sent.store(end, Ordering::Relaxed);
        Poll::Ready(Ok(count))
    }
}
