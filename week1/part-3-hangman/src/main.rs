// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::collections::HashSet;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)

    println!("Welcome to CS110L {}!", secret_word);

    let max_times = 5u8;
    let mut err_times = 0u8;
    let mut input = String::new();
    // let mut word_so_far = String::new();
    let mut word_so_far = Vec::new();
    let mut unknow_word = HashSet::new();
    let mut guessed_word = String::new();
    // let mut guessed_word = HastSet::new();
    for i in 0..secret_word_chars.len() {
        word_so_far.push('-');
        unknow_word.insert(secret_word_chars[i]);
    }
    loop {
        // 清空历史输入
        input.clear();

        // 打印进度
        println!("");
        // println!("The word so far is {}", word_so_far);
        println_word_so_far(&word_so_far);

        // 打印历史输入
        println!("You have guessed the following letters: {}", guessed_word);

        // 打印剩余次数
        println!("You have {} guesses left", max_times - err_times);

        // 提示输入
        println!("Please guess a letter:");

        match io::stdin().read_line(&mut input) {                           // O(n)
            Ok(n) => {
                match n {
                    // 只能输入单个字符 + 一个 0xA 换行字符
                    2 => {
                        let input_char = input.chars().nth(0).unwrap();

                        // 记录历史
                        guessed_word.push(input_char);
                        
                        /// case 1：猜对一个
                        /// case 2：猜对全部
                        /// case 3：没有猜对
                        // match secret_word_chars.contains(&input_char) {  // O(n)
                        //                                                      vs
                        match unknow_word.contains(&input_char) {           // O(1)
                            true => {
                                for i in 0..secret_word_chars.len() {       // O(n)
                                    if input_char == secret_word_chars[i] {
                                        // word_so_far.remove(i);              // O(n)
                                        // word_so_far.insert(i, input_char);  // O(n)
                                        //                                         vs
                                        word_so_far[i] = input_char;           // O(1)
                                    }
                                }

                                // if word_so_far == secret_word {          // O(n)
                                //                                              vs
                                if unknow_word.is_empty() {                 // O(1)
                                    println!("");
                                    println!("Congratulations you guessed the secret word: {}!", secret_word);
                                    break;
                                }
                            }
                            false => {
                                println!("Sorry, that letter is not in the word");
                                // 记录错误次数
                                err_times += 1;
                                if err_times >= max_times {
                                    println!("");
                                    println!("No more try. :-(");
                                    println!("The secret word is {}", secret_word);
                                    break;
                                }
                            }
                        }
                    }
                    1 => {
                        println!("Please input at least one letter.");
                        continue;
                    }
                    _ => {
                        println!("Please input only one letter.");
                        continue;
                    }
                }
            }
            Err(error) => {
                println!("Error: {error}");
            }
        }
    }

    fn println_word_so_far(word_so_far: &Vec<char>) {
        print!("The word so far is ");
        for c in word_so_far {
            print!("{}", c);
        }
        println!("");
    }
}
