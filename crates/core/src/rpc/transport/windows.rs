use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::windows::io::AsRawHandle,
    path::Path,
    ptr, thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::Pipes::PeekNamedPipe;

const POLL: Duration = Duration::from_millis(8);

#[derive(Debug)]
pub struct Stream {
    pipe: File,
    timeout: Duration,
}

impl Stream {
    pub fn open(path: &Path, timeout: Duration) -> io::Result<Self> {
        let pipe = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Self { pipe, timeout })
    }

    fn available(&self) -> io::Result<u32> {
        let mut available = 0;
        let succeeded = unsafe {
            PeekNamedPipe(
                self.pipe.as_raw_handle().cast(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &mut available,
                ptr::null_mut(),
            )
        };
        if succeeded == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(available)
    }
}

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let deadline = Instant::now() + self.timeout;
        while self.available()? == 0 {
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        }
        self.pipe.read(buffer)
    }
}

impl Write for Stream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pipe.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
