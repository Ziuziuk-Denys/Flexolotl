#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::Stream;
#[cfg(windows)]
pub use windows::Stream;
