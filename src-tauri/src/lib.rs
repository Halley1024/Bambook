mod application;
mod bootstrap;
mod commands;
mod domain;
mod error;
mod infrastructure;
mod state;

pub fn run() {
    bootstrap::run();
}
