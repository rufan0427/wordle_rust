use colored::Colorize;
use std::char;
use std::collections::HashMap;
use std::io::{self, Write};
mod builtin_words;
use builtin_words::ACCEPTABLE;
use builtin_words::FINAL;
use clap::Parser;
use rand::Rng;

fn pr(c: char) {
    match &c {
        'R' => print!("{}", "R".bold().red()),
        'Y' => print!("{}", "Y".bold().yellow()),
        'G' => print!("{}", "G".bold().green()),
        'X' => print!("{}", "X".bold()),
        _ => print!("invaild"),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'w', long = "word")]
    words: Option<String>,
    #[arg(short = 'r', long = "random")]
    rand_verbos: bool,
}

struct GameHistory {
    s_status_history: Vec<char>,
    char_status_history: Vec<char>,
    guesses: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let is_tty = atty::is(atty::Stream::Stdout);
    let mut game_record: Vec<GameHistory> = Vec::new();
    if is_tty {
        println!(
            "I am in a tty. Please print {}!",
            "colorful characters".bold().red()
        );
        print!("{}", "Your name: ".bold().blue());
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        println!("Welcome to wordle, {}!", line.trim());
        io::stdout().flush().unwrap();

        let mut answer_word = String::new();
        if cli.rand_verbos {
            let rand_index = rand::thread_rng().gen_range(0..FINAL.len());
            answer_word = FINAL[rand_index].to_string();
        } else if let Some(x) = cli.words {
            answer_word = x.clone();
        } else {
            print!("Input the answer word:");
            io::stdout().flush().unwrap();
            answer_word = String::new();
            io::stdin().read_line(&mut answer_word)?;
            io::stdout().flush().unwrap();
        }

        let mut chracter_status: Vec<char> = ['X'; 26].to_vec();
        let answer_word_vector: Vec<char> = answer_word.chars().collect();
        let mut appearance: HashMap<char, i32> = HashMap::new();
        for c in answer_word.chars() {
            let pos = appearance.entry(c).or_insert(0);
            *pos += 1;
        }

        let mut guess = String::new();
        let mut turn = 1;
        let mut flag: bool = false;
        while turn < 6 {
            println!("You have {} chance left,Input you guess:", 6 - turn);
            io::stdin().read_line(&mut guess)?;
            guess = guess.to_lowercase();

            let mut guess_word_vector: Vec<char> = guess.chars().collect();
            let mut guess_appearance = appearance.clone();
            let mut s_status: Vec<char> = ['R'; 5].to_vec();
            let mut input_flag: bool = true;

            if !(ACCEPTABLE.contains(&guess.trim())) {
                println!("INVALID");
                input_flag = false;
            } else if guess.len() != 6 {
                println!("INVALID");
                input_flag = false;
            } else {
                for i in 0..5 {
                    if !((guess_word_vector[i] >= 'A' && guess_word_vector[i] <= 'Z')
                        || (guess_word_vector[i] >= 'a' && guess_word_vector[i] <= 'z'))
                    {
                        println!("INVALID");
                        input_flag = false;
                        break;
                    }
                }
            }
            if input_flag {
                for i in 0..5 {
                    if guess_word_vector[i] == answer_word_vector[i] {
                        s_status[i] = 'G';
                        *guess_appearance.entry(answer_word_vector[i]).or_insert(0) -= 1;
                    }
                }
                for i in 0..5 {
                    if guess_word_vector[i] != answer_word_vector[i] {
                        if let Some(&x) = guess_appearance.get(&guess_word_vector[i]) {
                            if x > 0 {
                                s_status[i] = 'Y';
                                *guess_appearance.entry(guess_word_vector[i]).or_insert(0) -= 1;
                            }
                        }
                    }
                    match s_status[i] {
                        'G' => {
                            chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'G'
                        }
                        'Y' if chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                            != 'G' =>
                        {
                            chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'Y'
                        }
                        'R' if (chracter_status
                            [((guess_word_vector[i] as u8) - b'a') as usize]
                            != 'G'
                            && chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                                != 'Y') =>
                        {
                            chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'R';
                        }
                        _ => continue,
                    }
                }

                game_record.push(GameHistory {
                    s_status_history: s_status.clone(),
                    char_status_history: chracter_status.clone(),
                    guesses: guess.clone(),
                });

                for history_iter in game_record.iter() {
                    flag = true;
                    for iter in history_iter.s_status_history.iter() {
                        pr(*iter);
                        if *iter != 'G' {
                            flag = false;
                        }
                    }
                    print!("{}", ' ');
                    for iter in history_iter.char_status_history.iter() {
                        pr(*iter);
                    }
                    println!("");
                }
            }

            if flag {
                break;
            }
            if input_flag {
                turn += 1;
            }
            s_status.clear();
            guess.clear();
            guess_appearance.clear();
            guess_word_vector.clear();
        }

        println!("Guess turns:{}", turn);
        if !flag {
            println!("Answer:{}", answer_word.to_uppercase());
        }

    // // example: print arguments
    // print!("Command line arguments: ");
    // for arg in std::env::args() {
    //     print!("{} ", arg);
    // }
    // // TODO: parse the arguments in `args`
    } else {
        let mut answer_word = String::new();
        if cli.rand_verbos {
            let rand_index = rand::thread_rng().gen_range(0..FINAL.len());
            answer_word = FINAL[rand_index].to_string();
        } else if let Some(x) = cli.words {
            answer_word = x.clone();
        } else {
            answer_word = String::new();
            io::stdin().read_line(&mut answer_word)?;
            io::stdout().flush().unwrap();
        }
        io::stdout().flush().unwrap();

        let mut chracter_status: Vec<char> = ['X'; 26].to_vec();
        let answer_word_vector: Vec<char> = answer_word.chars().collect();
        let mut appearance: HashMap<char, i32> = HashMap::new();
        for c in answer_word.chars() {
            let pos = appearance.entry(c).or_insert(0);
            *pos += 1;
        }

        let mut guess = String::new();
        let mut turn = 1;
        let mut flag: bool = false;
        while turn <= 6 {
            io::stdin().read_line(&mut guess)?;
            guess = guess.to_lowercase();

            let mut guess_word_vector: Vec<char> = guess.chars().collect();
            let mut guess_appearance = appearance.clone();
            let mut s_status: Vec<char> = ['R'; 5].to_vec();
            let mut input_flag: bool = true;

            if !(ACCEPTABLE.contains(&guess.trim())) {
                println!("INVALID");
                input_flag = false;
            } else if guess.len() != 6 {
                println!("INVALID");
                input_flag = false;
            } else {
                for i in 0..5 {
                    if !((guess_word_vector[i] >= 'A' && guess_word_vector[i] <= 'Z')
                        || (guess_word_vector[i] >= 'a' && guess_word_vector[i] <= 'z'))
                    {
                        println!("INVALID");
                        input_flag = false;
                        break;
                    }
                }
            }

            if input_flag {
                for i in 0..5 {
                    if guess_word_vector[i] == answer_word_vector[i] {
                        s_status[i] = 'G';
                        *guess_appearance.entry(answer_word_vector[i]).or_insert(0) -= 1;
                    }
                }
                for i in 0..5 {
                    if guess_word_vector[i] != answer_word_vector[i] {
                        if let Some(&x) = guess_appearance.get(&guess_word_vector[i]) {
                            if x > 0 {
                                s_status[i] = 'Y';
                                *guess_appearance.entry(guess_word_vector[i]).or_insert(0) -= 1;
                            }
                        }
                    }
                    match s_status[i] {
                        'G' => {
                            chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'G'
                        }
                        'Y' if chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                            != 'G' =>
                        {
                            chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'Y'
                        }
                        'R' if (chracter_status
                            [((guess_word_vector[i] as u8) - b'a') as usize]
                            != 'G'
                            && chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                                != 'Y') =>
                        {
                            chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'R';
                        }
                        _ => continue,
                    }
                }
                flag = true;
                for iter in s_status.iter() {
                    pr(*iter);
                    if *iter != 'G' {
                        flag = false;
                    }
                }
                print!("{}", ' ');
                for iter in chracter_status.iter() {
                    pr(*iter);
                }
                println!("");
            }
            if flag {
                break;
            }
            if input_flag {
                turn += 1;
            }
            s_status.clear();
            guess.clear();
            guess_appearance.clear();
            guess_word_vector.clear();
        }

        if flag {
            println!("CORRECT {}", turn);
        } else {
            println!("FAILED {}", answer_word.to_uppercase());
        }
    }
    Ok(())
}
