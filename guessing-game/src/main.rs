use std::io;
use std::io::Write;
use std::cmp::Ordering;

use rand::Rng;

fn do_guess_round(secret_number: i32) -> bool {
    let ret: bool;
    let mut input = String::new();

    print!("Guess the magic number: ");
    io::stdout().flush().expect("could not flush stdout.");

    io::stdin()
        .read_line(&mut input)
        .expect("could not read from stdin.");

    let guess = match input.trim().parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            println!("Please enter a number.");
            return false;
        }
    };

    ret = guess == secret_number;

    match guess.cmp(&secret_number) {
        Ordering::Less => { println!("Bigger.") },
        Ordering::Equal => { println!("Nice job!") },
        Ordering::Greater => { println!("Less.") }
    };

    ret
}

fn guess_game() -> () {
    println!("My magic number is between 1 and 100 (inclusive).");

    let secret_number = rand::rng().random_range(1..=100);
    println!("the number is {}", secret_number);

    while !do_guess_round(secret_number) {}
}

fn main() {
    guess_game();
}
