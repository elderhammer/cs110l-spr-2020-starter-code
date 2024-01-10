use std::env;
use std::process;
use std::fs::File;
use std::io::{self, BufRead}; // For read_file_lines()

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)

    // The number of lines.
    let lines = read_file_lines(filename).expect("Failed to open file.");
    let number_of_lines = lines.len();

    // The number of words.
    let mut number_of_words = 0;
    let mut number_of_chars = 0;
    for line in lines.iter() {
        for word in read_line_words(line) {
            number_of_chars = number_of_chars + word.chars().count();
            number_of_words = number_of_words + 1;
        }
    }

    println!("lines: {}, words: {}, chars: {}", number_of_lines, number_of_words, number_of_chars);
}

fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    let file = File::open(filename)?;

    let mut lines = Vec::new();
    for line in io::BufReader::new(file).lines() {
        lines.push(line?);
    }

    Ok(lines)
}

fn read_line_words(line: &String) -> Vec<&str> {
    let mut words = Vec::new();
    for word in line.split_ascii_whitespace() {
        words.push(word);
    }

    words
}