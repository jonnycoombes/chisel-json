#![feature(test)]
#![allow(unused_imports)]
#![allow(dead_code)]
mod lexer;
mod scanner;
mod parser_errors;
mod parser_coords;

#[cfg(test)]
mod test;
mod string_table;
