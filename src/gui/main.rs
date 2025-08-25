use eframe::egui;
use std::collections::HashMap;
use rand::SeedableRng;

mod builtin_words;
use builtin_words::{ACCEPTABLE, FINAL};

#[derive(Default)]
struct WordleApp {
    answer: String,
    guesses: Vec<String>,
    feedback: Vec<Vec<char>>,
    current_guess: String,
    game_over: bool,
    message: String,
    keyboard_state: HashMap<char, char>,
    accept_list: Vec<String>,
    final_list: Vec<String>,
    config: GuiConfig,
}

#[derive(Default)]
struct GuiConfig {
    difficult: bool,
    seed: Option<u64>,
}

impl WordleApp {
    fn init(&mut self) {
        self.final_list = FINAL.iter().map(|s| s.to_string()).collect();
        self.accept_list = ACCEPTABLE.iter().map(|s| s.to_string()).collect();
        self.config.difficult = false;
        self.new_game();
    }

    fn new_game(&mut self) {
        use rand::Rng;
        
        let mut rng = if let Some(seed) = self.config.seed {
            rand::rngs::StdRng::seed_from_u64(seed)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        let index = rng.gen_range(0..self.final_list.len());
        self.answer = self.final_list[index].clone().to_uppercase();
        
        self.guesses.clear();
        self.feedback.clear();
        self.current_guess.clear();
        self.game_over = false;
        self.message.clear();
        self.keyboard_state.clear();
    }

    fn submit_guess(&mut self) {
        if self.current_guess.len() != 5 {
            self.message = "Word must be 5 letters".to_string();
            return;
        }

        let guess_lower = self.current_guess.to_lowercase();
        if !self.accept_list.contains(&guess_lower) {
            self.message = "Not in word list".to_string();
            return;
        }

        let feedback = self.calculate_feedback(&self.current_guess);
        let current_guess_clone = self.current_guess.clone();
        self.guesses.push(current_guess_clone.clone());
        self.feedback.push(feedback.clone());
        self.update_keyboard_state(&current_guess_clone, &feedback);
        
        self.current_guess.clear();

        if feedback.iter().all(|&c| c == 'G') {
            self.game_over = true;
            self.message = format!("You won in {} tries!", self.guesses.len());
        } else if self.guesses.len() >= 6 {
            self.game_over = true;
            self.message = format!("Game over! The word was {}", self.answer);
        }
    }

    fn calculate_feedback(&self, guess: &str) -> Vec<char> {
        let mut feedback = vec!['R'; 5];
        let mut answer_chars: Vec<char> = self.answer.chars().collect();
        let guess_chars: Vec<char> = guess.chars().collect();
        for i in 0..5 {
            if guess_chars[i] == answer_chars[i] {
                feedback[i] = 'G';
                answer_chars[i] = ' '; 
            }
        }
        for i in 0..5 {
            if feedback[i] == 'R' {
                if let Some(pos) = answer_chars.iter().position(|&c| c == guess_chars[i]) {
                    feedback[i] = 'Y';
                    answer_chars[pos] = ' '; 
                }
            }
        }

        feedback
    }

    fn update_keyboard_state(&mut self, guess: &str, feedback: &[char]) {
        for (i, c) in guess.chars().enumerate() {
            let current_state = self.keyboard_state.entry(c).or_insert('X');
            
            match feedback[i] {
                'G' => *current_state = 'G',
                'Y' if *current_state != 'G' => *current_state = 'Y',
                'R' if *current_state == 'X' => *current_state = 'R',
                _ => {}
            }
        }
    }

    fn get_key_color(&self, key: char) -> egui::Color32 {
        match self.keyboard_state.get(&key).unwrap_or(&'X') {
            'G' => egui::Color32::from_rgb(106, 170, 100), 
            'Y' => egui::Color32::from_rgb(201, 180, 88),  
            'R' => egui::Color32::from_rgb(120, 124, 126), 
            _ => egui::Color32::from_rgb(211, 214, 218),   
        }
    }

    fn render_game_grid(&self, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        egui::Grid::new("game_grid")
            .spacing([8.0, 50.0])
            .show(ui, |ui| {
                for row in 0..6 {
                    for col in 0..5 {
                        
                        ui.add_space(100.0);
                        let cell_size = egui::vec2(80.0, 50.0);
                        
                        if row < self.guesses.len() {
                            let letter = self.guesses[row].chars().nth(col).unwrap();
                            let feedback_char = self.feedback[row][col];
                            
                            let color = match feedback_char {
                                'G' => egui::Color32::from_rgb(106, 170, 100),
                                'Y' => egui::Color32::from_rgb(201, 180, 88),
                                _ => egui::Color32::from_rgb(120, 124, 126),
                            };
                            
                            let rect = egui::Rect::from_min_size(ui.cursor().min, cell_size);
                            ui.painter().rect_filled(rect, 4.0, color);

                            ui.add_space(25.0);

                            ui.label(
                                egui::RichText::new(letter.to_string())
                                    .color(egui::Color32::WHITE)
                                    .size(40.0)
                                    .strong()
                            );
                        } else if row == self.guesses.len() && col < self.current_guess.len() {
                            let letter = self.current_guess.chars().nth(col).unwrap();
                            
                            let rect = egui::Rect::from_min_size(ui.cursor().min, cell_size);
                            ui.painter().rect_stroke(
                                rect,
                                4.0,
                                egui::Stroke::new(2.0, egui::Color32::GRAY),
                            );

                            ui.add_space(25.0);

                            ui.label(
                                egui::RichText::new(letter.to_string())
                                    .size(40.0)
                                    .strong()
                            );
                        } else {
                            let rect = egui::Rect::from_min_size(ui.cursor().min, cell_size);
                            ui.painter().rect_stroke(
                                rect,
                                4.0,
                                egui::Stroke::new(2.0, egui::Color32::GRAY),
                            );
                            
                            ui.label(" ");
                        }
                    }
                    ui.end_row();
                }
            });
        });
    }

    fn render_keyboard(&mut self, ui: &mut egui::Ui) {
        let keyboard_rows = [
            "QWERTYUIOP",
            "ASDFGHJKL",
            "ZXCVBNM"
        ];

        for row in keyboard_rows.iter() {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 4.0);
                let total_width = row.len() as f32 * 46.0 -4.0; 
                let available_width = ui.available_width();
                let padding = (available_width - total_width) / 2.0;
                ui.add_space(padding);
                for key in row.chars() {
                    let button = egui::Button::new(
                        egui::RichText::new(key.to_string())
                            .size(16.0)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(self.get_key_color(key))
                    .min_size(egui::vec2(36.0, 46.0));
                    
                    if ui.add(button).clicked() && !self.game_over && self.current_guess.len() < 5 {
                        self.current_guess.push(key);
                    }
                }
                ui.add_space(padding);
            });
            
            ui.add_space(6.0);
        }

        ui.horizontal(|ui| {
            let total_width = 520.0; 
            let available_width = ui.available_width();
            let padding = (available_width - total_width) / 2.0;
            ui.add_space(padding);
            let enter_button = ui.add(
                egui::Button::new("ENTER")
                    .min_size(egui::vec2(100.0, 46.0))
            );
            
            if enter_button.clicked() && !self.game_over && self.current_guess.len() == 5 {
                self.submit_guess();
            }
            
            ui.add_space(100.0);

            let backspace_button = ui.add(
                egui::Button::new("BACKSPACE")
                    .min_size(egui::vec2(100.0, 46.0))
            );
            
            if backspace_button.clicked() && !self.game_over && !self.current_guess.is_empty() {
                self.current_guess.pop();
            }

            ui.add_space(100.0);

            let new_game_button = ui.add(
                egui::Button::new("NEW GAME")
                    .min_size(egui::vec2(100.0, 46.0))
            );
            
            if new_game_button.clicked() {
                self.new_game();
            }
        });
    }
}

impl eframe::App for WordleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("WORDLE")
                        .size(36.0)
                        .color(egui::Color32::from_rgb(106, 170, 100))
                );
            });

            ui.add_space(20.0);

            self.render_game_grid(ui);

            ui.add_space(40.0);

            self.render_keyboard(ui);

            if !self.game_over {
                ctx.input(|i| {
                    for event in &i.events {
                        if let egui::Event::Text(text) = event {
                            for c in text.chars() {
                                if c.is_ascii_alphabetic() && self.current_guess.len() < 5 {
                                    self.current_guess.push(c.to_ascii_uppercase());
                                }
                            }
                        }
                    }
                    
                    if i.key_pressed(egui::Key::Backspace) && !self.current_guess.is_empty() {
                        self.current_guess.pop();
                        self.message.clear();
                    }
                    
                    if i.key_pressed(egui::Key::Enter) && self.current_guess.len() == 5 {
                        self.submit_guess();
                    }
                });
            }

            ui.add_space(20.0);

            if !self.message.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(&self.message)
                            .size(24.0)
                            .color(egui::Color32::RED)
                    );
                });
            }

        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 1200.0])  
            .with_min_inner_size([500.0, 700.0])
            .with_resizable(false),
        ..Default::default()
    };
    options.centered=true;
    eframe::run_native(
        "Wordle Game",
        options,
        Box::new(|_cc| {
            let mut app = WordleApp::default();
            app.init();
            Box::new(app)
        }),
    )
}