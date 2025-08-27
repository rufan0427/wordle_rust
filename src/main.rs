use colored::Colorize;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::char;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
mod builtin_words;
use builtin_words::ACCEPTABLE;
use builtin_words::FINAL;
use clap::Parser;
use config::Config;
use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::PathBuf;

//help print colorful chracters
fn pr(c: char) {
    match &c {
        'R' => print!("{}", "R".bold().red()),
        'Y' => print!("{}", "Y".bold().yellow()),
        'G' => print!("{}", "G".bold().green()),
        'X' => print!("{}", "X".bold()),
        _ => print!("invaild"),
    }
}

//commond-line argments parser
#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'w', long = "word")]
    words: Option<String>,
    #[arg(short = 'r', long = "random")]
    rand_verbos: bool,
    #[arg(short = 'D', long = "difficult")]
    diff_verbos: bool,
    #[arg(short = 't', long = "stats")]
    status_verbos: bool,
    #[arg(short = 'd', long = "day", default_value_t = 1)]
    days: usize,
    #[arg(short = 's', long = "seed")]
    seed: Option<u64>,
    #[arg(short = 'f', long = "final-set")]
    final_repo: Option<PathBuf>,
    #[arg(short = 'a', long = "acceptable-set")]
    accept_repo: Option<PathBuf>,
    #[arg(short = 'S', long = "state")]
    state: Option<PathBuf>,
    #[arg(short = 'c', long = "config")]
    config: Option<PathBuf>,
    #[arg(short = 'p', long = "tips")]
    tips: bool,
}

//for reactive mood,output the guess result history
struct GameHistory {
    s_status_history: Vec<char>,
    char_status_history: Vec<char>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonState {
    #[serde(default)]
    total_rounds: i32,
    #[serde(default)]
    games: Vec<Game>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    answer: String,
    guesses: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct AppConfig {
    random: Option<bool>,
    difficult: Option<bool>,
    stats: Option<bool>,
    day: Option<usize>,
    seed: Option<u64>,
    final_set: Option<PathBuf>,
    acceptable_set: Option<PathBuf>,
    state: Option<PathBuf>,
    word: Option<String>,
}

fn merge_config(cli: &Cli) -> Result<Cli, Box<dyn std::error::Error>> {
    // merge config.json and commond line arguments

    let mut merged_cli = cli.clone();

    if let Some(config_path) = &cli.config {
        let settings = Config::builder()
            .add_source(config::File::with_name(config_path.to_str().unwrap()))
            .build()?;

        let app_config: AppConfig = settings.try_deserialize()?;

        if merged_cli.words.is_none() {
            merged_cli.words = app_config.word;
        }
        if !merged_cli.rand_verbos {
            merged_cli.rand_verbos = app_config.random.unwrap_or(false);
        }
        if !merged_cli.diff_verbos {
            merged_cli.diff_verbos = app_config.difficult.unwrap_or(false);
        }
        if !merged_cli.status_verbos {
            merged_cli.status_verbos = app_config.stats.unwrap_or(false);
        }
        if merged_cli.days == 1 {
            merged_cli.days = app_config.day.unwrap_or(1);
        }
        if merged_cli.seed.is_none() {
            merged_cli.seed = app_config.seed;
        }
        if merged_cli.final_repo.is_none() {
            merged_cli.final_repo = app_config.final_set;
        }
        if merged_cli.accept_repo.is_none() {
            merged_cli.accept_repo = app_config.acceptable_set;
        }
        if merged_cli.state.is_none() {
            merged_cli.state = app_config.state;
        }
    }

    Ok(merged_cli)
}

fn play_tty(
    cli: &Cli,
    answer_list: &mut Vec<String>,
    guess_list: &mut BTreeMap<String, i32>,
    final_list: &[String],
    accept_list: &[String],
    json_data: &mut JsonState,
    id: usize,
) -> i32 {
    let mut game: Game = Game {
        answer: String::new(),
        guesses: Vec::new(),
    };
    let mut game_record: Vec<GameHistory> = Vec::new();
    if cli.words.is_some() {
        if cli.days != 1 {
            return 10000;
        }
        if cli.seed.is_some() {
            return 10000;
        }
    }
    if cli.rand_verbos && cli.words.is_some() {
        return 10000;
    }
    println!(
        "I am in a tty. Please print {}!",
        "colorful characters".bold().red()
    );
    print!("{}", "Your name: ".bold().blue());
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("cannot read");
    println!("Welcome to wordle, {}!", line.trim());
    io::stdout().flush().unwrap();

    // word-given mood switch
    let mut answer_word: String;

    if cli.rand_verbos {
        if cli.days == 1 {
            let rand_index = rand::thread_rng().gen_range(0..FINAL.len());
            answer_word = FINAL[rand_index].to_string();
            while answer_list.contains(&answer_word.clone()) {
                let rand_index = rand::thread_rng().gen_range(0..FINAL.len());
                answer_word = FINAL[rand_index].to_string();
            }
        } else {
            answer_word = final_list[id % final_list.len()].to_string()
        }
    } else if let Some(x) = &cli.words {
        answer_word = x.clone();
    } else {
        print!("Input the answer word:");
        io::stdout().flush().unwrap();
        answer_word = String::new();
        io::stdin()
            .read_line(&mut answer_word)
            .expect("cannot read");
        //io::stdout().flush().unwrap();
    }
    answer_list.push(answer_word.clone());
    game.answer = answer_word.to_uppercase().clone();

    let mut chracter_status: Vec<char> = ['X'; 26].to_vec(); //total status for 26 characters
    let answer_word_vector: Vec<char> = answer_word.chars().collect(); //transform str into list
    let mut appearance: HashMap<char, i32> = HashMap::new(); // each chracter's number in answer word
    for c in answer_word.chars() {
        let pos = appearance.entry(c).or_insert(0);
        *pos += 1;
    }

    let mut guess = String::new();
    let mut turn = 1;
    let mut flag: bool = false;
    let mut green_list: Vec<(char, i32)> = Vec::new();
    let mut yellow_list: Vec<char> = Vec::new();
    let mut red_list: Vec<char> = Vec::new();

    while turn <= 6 {
        println!("You have {} chance left,Input you guess:", 7 - turn);
        io::stdin().read_line(&mut guess).expect("cannot read");
        guess = guess.to_lowercase(); //convenient for vertify

        let mut guess_word_vector: Vec<char> = guess.chars().collect(); //transform str into list
        let mut guess_appearance = appearance.clone(); // required number in guess word,for vertify overplus 
        let mut s_status: Vec<char> = ['R'; 5].to_vec(); //// each chracter's status in guess word,default='R'
        let mut input_flag: bool = true; //legal input

        if !(accept_list.contains(&guess.trim().to_string())) {
            //not  in ACCEPTABLE
            input_flag = false;
        } else if guess.len() != 6 {
            input_flag = false;
        } else if cli.diff_verbos {
            for iter in green_list.iter() {
                if guess_word_vector[iter.1 as usize] != iter.0 {
                    input_flag = false;
                    break;
                }
            }
            for iter in yellow_list.iter() {
                if !guess.contains(*iter) {
                    input_flag = false;
                    break;
                }
            }
        } else {
            for i in guess_word_vector.iter().take(5) {
                //chracter
                if !((*i >= 'A' && *i <= 'Z') || (*i >= 'a' && *i <= 'z')) {
                    input_flag = false;
                    break;
                }
            }
        }
        if !input_flag {
            println!("INVALID");
        }

        if input_flag {
            *guess_list
                .entry(guess.trim().to_string().to_uppercase())
                .or_insert(0) += 1;
            game.guesses.push(guess.trim().to_uppercase().to_string());

            for i in 0..5 {
                if guess_word_vector[i] == answer_word_vector[i] {
                    s_status[i] = 'G';
                    *guess_appearance.entry(answer_word_vector[i]).or_insert(0) -= 1;
                } //give 'G'
            }
            for i in 0..5 {
                if guess_word_vector[i] != answer_word_vector[i]
                    && let Some(&x) = guess_appearance.get(&guess_word_vector[i])
                    && x > 0
                {
                    //don't overplus
                    s_status[i] = 'Y';
                    *guess_appearance.entry(guess_word_vector[i]).or_insert(0) -= 1;
                }

                match s_status[i] {
                    //renew the character's status
                    'G' => {
                        green_list.push((guess_word_vector[i], i as i32));
                        chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'G'
                    }
                    'Y' if chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                        != 'G' =>
                    {
                        yellow_list.push(guess_word_vector[i]);
                        chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'Y'
                    }
                    'R' if (chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                        != 'G'
                        && chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                            != 'Y') =>
                    {
                        red_list.push(guess_word_vector[i]);
                        chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'R';
                    }
                    _ => continue,
                }
            }

            game_record.push(GameHistory {
                s_status_history: s_status.clone(),
                char_status_history: chracter_status.clone(),
            });

            for history_iter in game_record.iter() {
                flag = true;
                for iter in history_iter.s_status_history.iter() {
                    pr(*iter);
                    if *iter != 'G' {
                        flag = false;
                    }
                }
                print!(" ");
                for iter in history_iter.char_status_history.iter() {
                    pr(*iter);
                }
                println!();
            }

            if cli.tips && turn != 6 {
                let mut pos_flag: bool;
                let mut pos_word_list: Vec<&str> = Vec::new();
                for &possible_word in ACCEPTABLE {
                    pos_flag = true;
                    let pos_word_vec: Vec<char> = possible_word.chars().collect();
                    for iter in green_list.iter() {
                        if pos_word_vec[iter.1 as usize] != iter.0 {
                            pos_flag = false;
                            break;
                        }
                    }
                    for iter in yellow_list.iter() {
                        if !possible_word.contains(*iter) || !pos_flag {
                            pos_flag = false;
                            break;
                        }
                    }
                    for iter in red_list.iter() {
                        if possible_word.contains(*iter) || !pos_flag {
                            pos_flag = false;
                            break;
                        }
                    }
                    if pos_flag {
                        pos_word_list.push(possible_word);
                    }
                }
                calculate_entropy(&mut pos_word_list);
                println!("{:?}", pos_word_list);
            }
        }
        if input_flag {
            turn += 1;
        }
        if flag {
            break;
        }

        s_status.clear();
        guess.clear();
        guess_appearance.clear();
        guess_word_vector.clear();
    }
    json_data.games.push(game);
    turn -= 1;
    println!("Guess turns:{}", turn);
    if !flag {
        println!("Answer:{}", answer_word.to_uppercase());
        return 0;
    }
    turn
}

use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

fn calculate_entropy(pos_word_list: &mut Vec<&str>) {
    let len = pos_word_list.len();
    let mut recommond_list = PriorityQueue::new();
    for i in pos_word_list.iter() {
        let mut entropy = 0.0;
        let mut possible_analyse: [i32; 300] = [0; 300];
        for j in pos_word_list.iter() {
            let mut pos_num = 0;
            for pos in 0..5 {
                if i.chars().nth(pos) == j.chars().nth(pos) {
                    pos_num *= 3;
                    continue;
                }
                if let Some(target_char) = j.chars().nth(pos)
                    && i.find(target_char).is_some()
                {
                    pos_num = pos_num * 3 + 1;
                    continue;
                }
                pos_num = pos_num * 3 + 2;
            }
            possible_analyse[pos_num] += 1;
        }
        for k in possible_analyse.iter_mut().take(244) {
            if *k > 0 {
                let p = *k as f64 / len as f64;
                entropy += -p * f64::log2(p);
            }
            *k = 0;
        }
        recommond_list.push(*i, OrderedFloat(entropy));
    }
    println!("Top 5 words by entropy:");
    for _ in 0..5 {
        if let Some((word, entropy)) = recommond_list.pop() {
            println!("{}: {:.4}", word, entropy.0);
        }
    }
}

fn play_dis_tty(
    cli: &Cli,
    answer_list: &mut Vec<String>,
    guess_list: &mut BTreeMap<String, i32>,
    final_list: &[String],
    accept_list: &[String],
    json_data: &mut JsonState,
    id: usize,
) -> i32 {
    let mut game: Game = Game {
        answer: String::new(),
        guesses: Vec::new(),
    };
    let mut answer_word: String;
    if cli.words.is_some() {
        if cli.days != 1 {
            return 10000;
        }
        if cli.seed.is_some() {
            return 10000;
        }
    }
    if cli.rand_verbos && cli.words.is_some() {
        return 10000;
    }
    if cli.rand_verbos {
        if cli.days == 1 {
            let rand_index = rand::thread_rng().gen_range(0..FINAL.len());
            answer_word = FINAL[rand_index].to_string();
            while answer_list.contains(&answer_word.clone()) {
                let rand_index = rand::thread_rng().gen_range(0..FINAL.len());
                answer_word = FINAL[rand_index].to_string();
            }
        } else {
            //println!("{} {} {:?}",final_list.len(),id,final_list.iter().position(|&x| x == "photo"));
            answer_word = final_list[id % final_list.len()].to_string()
        }
    } else if let Some(x) = &cli.words {
        answer_word = x.clone();
    } else {
        answer_word = String::new();
        io::stdin()
            .read_line(&mut answer_word)
            .expect("cannot read");
        //io::stdout().flush().unwrap();
    }
    answer_list.push(answer_word.clone());
    game.answer = answer_word.to_uppercase().clone();
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
    let mut green_list: Vec<(char, i32)> = Vec::new();
    let mut yellow_list: Vec<char> = Vec::new();

    while turn <= 6 {
        io::stdin().read_line(&mut guess).expect("cannot read");

        let mut guess_word_vector: Vec<char> = guess.chars().collect();
        let mut guess_appearance = appearance.clone();
        let mut s_status: Vec<char> = ['R'; 5].to_vec();
        let mut input_flag: bool = true;

        if !(accept_list.contains(&guess.trim().to_string())) {
            //not  in ACCEPTABLE
            input_flag = false;
        } else if guess.len() != 6 {
            input_flag = false;
        } else if cli.diff_verbos {
            for iter in green_list.iter() {
                if guess_word_vector[iter.1 as usize] != iter.0 {
                    input_flag = false;
                    break;
                }
            }
            for iter in yellow_list.iter() {
                if !guess.contains(*iter) {
                    input_flag = false;
                    break;
                }
            }
        } else {
            for i in guess_word_vector.iter().take(5) {
                //chracter
                if !((*i >= 'A' && *i <= 'Z') || (*i >= 'a' && *i <= 'z')) {
                    input_flag = false;
                    break;
                }
            }
        }
        if !input_flag {
            println!("INVALID");
        }

        if input_flag {
            *guess_list
                .entry(guess.trim().to_uppercase().clone())
                .or_insert(0) += 1;
            game.guesses.push(guess.trim().to_uppercase().to_string());

            for i in 0..5 {
                if guess_word_vector[i] == answer_word_vector[i] {
                    s_status[i] = 'G';
                    *guess_appearance.entry(answer_word_vector[i]).or_insert(0) -= 1;
                }
            }
            for i in 0..5 {
                if guess_word_vector[i] != answer_word_vector[i]
                    && let Some(&x) = guess_appearance.get(&guess_word_vector[i])
                    && x > 0
                {
                    s_status[i] = 'Y';
                    *guess_appearance.entry(guess_word_vector[i]).or_insert(0) -= 1;
                }

                match s_status[i] {
                    'G' => {
                        green_list.push((guess_word_vector[i], i as i32));
                        chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'G'
                    }
                    'Y' if chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
                        != 'G' =>
                    {
                        yellow_list.push(guess_word_vector[i]);
                        chracter_status[((guess_word_vector[i] as u8) - b'a') as usize] = 'Y'
                    }
                    'R' if (chracter_status[((guess_word_vector[i] as u8) - b'a') as usize]
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
            print!(" ");
            for iter in chracter_status.iter() {
                pr(*iter);
            }
            println!();
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
    json_data.games.push(game);
    if flag {
        println!("CORRECT {}", turn);
        turn
    } else {
        println!("FAILED {}", answer_word.trim().to_uppercase());
        0
    }
}

fn load_word_list(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let words: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_lowercase())
        .filter(|word| !word.is_empty())
        .collect();

    if words.is_empty() {
        return Err("Empty".into());
    }
    let unique_words: HashSet<&str> = words.iter().map(|x| &x[..]).collect();
    if unique_words.len() != words.len() {
        return Err("repeat".into());
    }
    let final_set: HashSet<&str> = FINAL.iter().copied().collect();
    if !unique_words.is_subset(&final_set) {
        return Err("not subset".into());
    }
    let mut sorted_words = words;
    sorted_words.sort();
    Ok(sorted_words)
}

fn load_accept_list(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let words: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_lowercase())
        .filter(|word| !word.is_empty())
        .collect();

    if words.is_empty() {
        return Err("Empty".into());
    }
    let unique_words: HashSet<&str> = words.iter().map(|x| &x[..]).collect();
    if unique_words.len() != words.len() {
        return Err("repeat".into());
    }
    let final_set: HashSet<&str> = ACCEPTABLE.iter().copied().collect();
    if !unique_words.is_subset(&final_set) {
        return Err("not subset".into());
    }
    let mut sorted_words = words;
    sorted_words.sort();
    Ok(sorted_words)
}

fn load_state_json(path: &PathBuf) -> Result<JsonState, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn write_state_json(
    path: &PathBuf,
    json_data: &JsonState,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &json_data)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);
    let cli = Cli::parse();
    let merged_cli = merge_config(&cli)?;

    let mut json_data: JsonState = JsonState {
        total_rounds: 0,
        games: Vec::new(),
    };

    if let Some(ref __) = merged_cli.state {
        match load_state_json(&merged_cli.state.clone().unwrap()) {
            Result::Ok(x) => json_data = x,
            Err(st) => {
                json_data.total_rounds = 0;
                return Err(st);
            }
        }
    }
    let mut answer_list: Vec<String> = Vec::new();
    let mut guess_list: BTreeMap<String, i32> = BTreeMap::new();

    let mut final_list: Vec<String>;
    if let Some(ref x) = merged_cli.final_repo {
        match load_word_list(x) {
            std::result::Result::Ok(x) => final_list = x,
            std::result::Result::Err(_x) => return Err(String::from("load error").into()),
        } // or use .unwrap() directly
    } else {
        final_list = FINAL.iter().map(|&s| s.to_string()).collect();
    }
    let mut rng = if let Some(seed) = merged_cli.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::seed_from_u64(42)
    };
    final_list.shuffle(&mut rng);

    let accept_list: Vec<String>;
    if let Some(ref x) = merged_cli.accept_repo {
        accept_list = load_accept_list(x).unwrap();
    } else {
        accept_list = ACCEPTABLE.iter().map(|&s| s.to_string()).collect();
    }

    if is_tty {
        match merged_cli.words {
            Some(ref _x) => {
                let success_flag = play_tty(
                    &merged_cli,
                    &mut answer_list,
                    &mut guess_list,
                    &final_list,
                    &accept_list,
                    &mut json_data,
                    merged_cli.days - 1,
                );
                if success_flag == 10000 {
                    return Err(String::from("mood mix!").into());
                }
                Ok(())
            }

            None => {
                let mut turns_record: i32 = 0;
                let mut success_record: i32 = 0;
                let mut try_record: i32 = 0;
                if let Some(__) = &merged_cli.state {
                    for iter in json_data.games.iter() {
                        if iter.guesses[iter.guesses.len() - 1] == iter.answer {
                            success_record += 1;
                            try_record += iter.guesses.len() as i32;
                        }
                        for words_iter in 0..iter.guesses.len() {
                            *guess_list
                                .entry(iter.guesses[words_iter].trim().to_lowercase().to_string())
                                .or_insert(0) += 1;
                        }
                    }
                }
                loop {
                    let success_flag = play_tty(
                        &merged_cli,
                        &mut answer_list,
                        &mut guess_list,
                        &final_list,
                        &accept_list,
                        &mut json_data,
                        merged_cli.days - 1 + turns_record as usize,
                    );
                    turns_record += 1;
                    json_data.total_rounds += 1;
                    if success_flag == 10000 {
                        return Err(String::from("mood mix!").into());
                    }
                    if success_flag > 0 {
                        success_record += 1;
                        try_record += success_flag;
                    }
                    if let Some(x) = &merged_cli.state {
                        write_state_json(x, &json_data)?;
                    }
                    if merged_cli.status_verbos {
                        //io::stdout().flush().unwrap();
                        if success_record > 0 {
                            println!(
                                "{} {} {:.2}",
                                success_record,
                                json_data.total_rounds - success_record,
                                try_record as f32 / success_record as f32
                            );
                        } else {
                            println!("0 {} 0.00", json_data.total_rounds - success_record);
                        }
                        //io::stdout().flush().unwrap();
                        let mut entries: Vec<(&String, &i32)> = guess_list.iter().collect();
                        entries.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
                        for iter in entries.iter().take(5) {
                            io::stdout().flush().unwrap();
                            print!("{} {} ", iter.0.to_uppercase(), iter.1);
                        }
                        println!(" ");
                    }

                    //io::stdout().flush().unwrap();
                    println!("Do you want a new try? [Y/n]");
                    let mut cont = String::new();
                    std::io::stdin().read_line(&mut cont).expect("cannot read");
                    if cont.trim() != "Y" {
                        break Ok(());
                    }
                }
            }
        }
    } else {
        match merged_cli.words {
            Some(ref _x) => {
                let success_flag = play_dis_tty(
                    &merged_cli,
                    &mut answer_list,
                    &mut guess_list,
                    &final_list,
                    &accept_list,
                    &mut json_data,
                    merged_cli.days - 1,
                );
                if success_flag == 10000 {
                    return Err(String::from("mood mix!").into());
                }
                Ok(())
            }
            None => {
                let mut turns_record: i32 = 0;
                let mut success_record: i32 = 0;
                let mut try_record: i32 = 0;
                if let Some(__) = &merged_cli.state {
                    for iter in json_data.games.iter() {
                        if iter.guesses[iter.guesses.len() - 1] == iter.answer {
                            success_record += 1;
                            try_record += iter.guesses.len() as i32;
                        }
                        for words_iter in 0..iter.guesses.len() {
                            let key = iter.guesses[words_iter].trim().to_string();
                            *guess_list.entry(key).or_insert(0) += 1;
                        }
                    }
                }
                loop {
                    let success_flag = play_dis_tty(
                        &merged_cli,
                        &mut answer_list,
                        &mut guess_list,
                        &final_list,
                        &accept_list,
                        &mut json_data,
                        (merged_cli.days - 1) + turns_record as usize,
                    );
                    turns_record += 1;
                    json_data.total_rounds += 1;
                    if success_flag == 10000 {
                        return Err(String::from("mood mix!").into());
                    }
                    if success_flag > 0 {
                        success_record += 1;
                        try_record += success_flag;
                    }

                    if merged_cli.status_verbos {
                        //io::stdout().flush().unwrap();
                        if success_record > 0 {
                            println!(
                                "{} {} {:.2}",
                                success_record,
                                json_data.total_rounds - success_record,
                                try_record as f32 / success_record as f32
                            );
                        } else {
                            println!("0 {} 0.00", json_data.total_rounds - success_record);
                        }
                        io::stdout().flush().unwrap();
                        let mut entries: Vec<(&String, &i32)> = guess_list.iter().collect();
                        entries.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
                        let output: String = entries
                            .iter()
                            .take(5)
                            .map(|iter| format!("{} {}", iter.0.to_uppercase(), iter.1))
                            .collect::<Vec<String>>()
                            .join(" ");

                        println!("{}", output);
                    }

                    if let Some(x) = &merged_cli.state {
                        write_state_json(x, &json_data)?;
                    }

                    io::stdout().flush().unwrap();
                    let mut cont = String::new();
                    std::io::stdin().read_line(&mut cont).expect("cannot read");
                    if cont.trim() != "Y" {
                        break;
                    }
                }
                match &merged_cli.state {
                    Some(_x) => Ok(()),
                    _ => Ok(()),
                }
            }
        }
    }
}
