// https://github.com/zellij-org/zellij/blob/61a9b06237d1b84a6af5132f43b9f48902e2dc80/zellij-server/src/pty.rs#L426
// https://man7.org/linux/man-pages/man3/termios.3.html
// https://en.wikibooks.org/wiki/Serial_Programming/termios
pub mod os_io;
pub mod command;
pub mod data;
pub mod error;
pub use anyhow;
fn main() {
    println!("Hello, world!");
}
