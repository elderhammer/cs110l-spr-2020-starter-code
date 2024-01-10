use std::collections::VecDeque;
#[allow(unused_imports)]
use std::sync::{Arc, Mutex};
use std::time::Instant;
#[allow(unused_imports)]
use std::{env, process, thread};

/// Determines whether a number is prime. This function is taken from CS 110 factor.py.
///
/// You don't need to read or understand this code.
#[allow(dead_code)]
fn is_prime(num: u32) -> bool {
    if num <= 1 {
        return false;
    }
    for factor in 2..((num as f64).sqrt().floor() as u32) {
        if num % factor == 0 {
            return false;
        }
    }
    true
}

/// Determines the prime factors of a number and prints them to stdout. This function is taken
/// from CS 110 factor.py.
///
/// You don't need to read or understand this code.
#[allow(dead_code)]
fn factor_number(num: u32) {
    let start = Instant::now();

    if num == 1 || is_prime(num) {
        println!("{} = {} [time: {:?}]", num, num, start.elapsed());
        return;
    }

    let mut factors = Vec::new();
    let mut curr_num = num;
    for factor in 2..num {
        while curr_num % factor == 0 {
            factors.push(factor);
            curr_num /= factor;
        }
    }
    factors.sort();
    let factors_str = factors
        .into_iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(" * ");
    println!("{} = {} [time: {:?}]", num, factors_str, start.elapsed());
}

/// Returns a list of numbers supplied via argv.
#[allow(dead_code)]
fn get_input_numbers() -> VecDeque<u32> {
    let mut numbers = VecDeque::new();
    for arg in env::args().skip(1) {
        if let Ok(val) = arg.parse::<u32>() {
            numbers.push_back(val);
        } else {
            println!("{} is not a valid number", arg);
            process::exit(1);
        }
    }
    numbers
}

// 同步原语消除竞争
fn pop_number(input_numbers: Arc<Mutex<VecDeque<u32>>>) -> Option<u32> {
    input_numbers.lock().ok()?.pop_back()
}

fn main() {
    let num_threads = num_cpus::get();
    println!("Farm starting on {} CPUs", num_threads);
    let start = Instant::now();

    // call get_input_numbers() and store a queue of numbers to factor
    let input_numbers = get_input_numbers();
    println!("Input numbers: {:?}", input_numbers);

    // 1.将 input_numbers 传递给线程，利用 Arc 获得 Send 能力
    // 2.多个线程从 input_numbers pop 数据，产生竞争状态，利用 Mutex 同步原语消除竞争
    let input_numbers = Arc::new(Mutex::new(input_numbers));

    // spawn `num_threads` threads, each of which pops numbers off the queue and calls
    // factor_number() until the queue is empty
    let mut threads =  vec![];
    for _ in 0..num_threads {
        let input_numbers_clone = input_numbers.clone();
        threads.push(thread::spawn(move || {
            while let Some(number) = pop_number(input_numbers_clone.clone()) {
                factor_number(number);
            }
        }));
    }

    // join all the threads you created
    for handler in threads {
        handler.join().expect("something wrong");
    }

    println!("Total execution time: {:?}", start.elapsed());
}
