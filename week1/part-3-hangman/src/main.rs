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
use std::iter;
use std::io::Write;

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
    let mut secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);
    let mut guessed_word: String = iter::repeat('-').take(secret_word_chars.len()).collect();
    let mut guessed_letters = String::new();
    // Your code here! :)
    println!("Welcome to CA110L Hangman!");
    let mut n_guess = NUM_INCORRECT_GUESSES;
    while n_guess > 0 && guessed_word.contains('-') {
        println!("The word so far is {}", guessed_word);
        println!("You have guessed the following letters: {}", guessed_letters);
        println!("You have {} guesses left", n_guess);
        print!("Please guess a letter: ");
        io::stdout()
            .flush()
            .expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.");
        let guess = guess.chars().collect::<Vec<char>>()[0];
        guessed_letters.push(guess);
        let res = secret_word_chars.iter().position(|&c| c == guess);
        match res {
            Some(i) => {
                secret_word_chars[i] = '\0';
                guessed_word.replace_range(i..i + 1, &*guess.to_string());
            }
            None => {
                println!("Sorry, that letter is not in the word");
                n_guess -= 1;
            }
        }
        println!();
    }
    if guessed_word.contains('-') {
        println!("Sorry, you ran out of guesses!")
    } else {
        println!("Congratulations you guessed the secret word: {}!", guessed_word);
    }
}
