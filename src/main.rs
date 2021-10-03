#![allow(dead_code)]

pub mod tokenizer;
pub mod parser;

fn multiply_by_two(input: Vec<i32>) -> Vec<i32> {
    let mut output = Vec::with_capacity(input.len());

    for i in input {
        output.push(i * 2)
    }

    output
}

fn multiply_by_two_iter(input: Vec<i32>) -> Vec<i32> {
    input.into_iter().map(x2).collect()
}

fn multiply_by_two_reduce(input: Vec<i32>) -> i32 {
    input.into_iter().map(x2).fold(0, sum)
}

fn x2(input: i32) -> i32 {
    input * 2
}

fn sum(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("Hello world!");
}