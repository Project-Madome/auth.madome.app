extern crate util;

pub mod app;
pub mod command;
pub mod config;
pub mod database;
pub mod entity;
pub mod error;
pub mod json;
pub mod model;
pub mod msg;
pub mod registry;
pub mod repository;
pub mod usecase;

pub use registry::RootRegistry;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

pub fn release() -> bool {
    cfg!(not(debug_assertions))
}

pub fn debug() -> bool {
    cfg!(debug_assertions)
}

pub fn test() -> bool {
    cfg!(test)
}
