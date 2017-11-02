#![cfg_attr(all(feature = "bench", test), feature(test))]

#[cfg(all(feature = "bench", test))]
extern crate test;

pub mod board;
pub mod solver;
pub mod hintmap;
pub mod generator;
extern crate rand;
extern crate scoped_threadpool;

