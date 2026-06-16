// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-06-16
// Description: Smarter Python launcher for Windows — drop-in py.exe replacement with PyPy, Anaconda, and custom alias support
// License: MIT

mod cli;
mod config;
mod launch;
mod list;
mod probe;
mod registry;
mod resolve;

fn main() {
    cli::run();
}
