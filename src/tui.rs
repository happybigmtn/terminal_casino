use crate::baccarat::{BaccaratGame, GameMode, BonusBets, Card};
use crate::card_renderer::{CardRenderer, CardAnimation};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BetType {
    Player,
    Banker,
    Tie,
}

pub struct GameStats {
    rounds_played: u32,
    player_wins: u32,
    banker_wins: u32,
    ties: u32,
    total_wagered: i32,
    total_won: i32,
}

impl GameStats {
    fn new() -> Self {
        Self {
            rounds_played: 0,
            player_wins: 0,
            banker_wins: 0,
            ties: 0,
            total_wagered: 0,
            total_won: 0,
        }
    }
    
    fn win_rate(&self) -> f32 {
        if self.total_wagered == 0 {
            0.0
        } else {
            (self.total_won as f32 / self.total_wagered as f32) * 100.0
        }
    }
}

pub struct RatatuiUI {
    game: BaccaratGame,
    balance: i32,
    current_bet: i32,
    bet_type: BetType,
    bonus_bets: BonusBets,
    game_mode: GameMode,
    stats: GameStats,
    show_stats: bool,
    animation_state: AnimationState,
    last_update: Instant,
}

#[derive(Debug, Clone)]
struct AnimationState {
    dealing: bool,
    cards_to_reveal: Vec<CardAnimation>,
    current_reveal_index: usize,
    deal_start_time: Option<Instant>,
}

impl AnimationState {
    fn new() -> Self {
        Self {
            dealing: false,
            cards_to_reveal: Vec::new(),
            current_reveal_index: 0,
            deal_start_time: None,
        }
    }
    
    fn start_dealing(&mut self, cards: Vec<Card>) {
        self.dealing = true;
        self.cards_to_reveal = cards.into_iter()
            .enumerate()
            .map(|(i, card)| CardAnimation::new(card, i))
            .collect();
        self.current_reveal_index = 0;
        self.deal_start_time = Some(Instant::now());
    }
    
    fn update(&mut self) {
        if !self.dealing {
            return;
        }
        
        if let Some(start_time) = self.deal_start_time {
            let elapsed = start_time.elapsed();
            let reveal_interval = Duration::from_millis(1000); // 1 second per card
            
            let cards_to_reveal = (elapsed.as_millis() / reveal_interval.as_millis()) as usize;
            
            for i in self.current_reveal_index..cards_to_reveal.min(self.cards_to_reveal.len()) {
                if i < self.cards_to_reveal.len() {
                    self.cards_to_reveal[i].reveal();
                }
            }
            
            self.current_reveal_index = cards_to_reveal.min(self.cards_to_reveal.len());
            
            if self.current_reveal_index >= self.cards_to_reveal.len() {
                self.dealing = false;
            }
        }
    }
    
    fn is_complete(&self) -> bool {
        !self.dealing
    }
}

impl RatatuiUI {
    pub fn new() -> Self {
        Self {
            game: BaccaratGame::new(),
            balance: 1000,
            current_bet: 0,
            bet_type: BetType::Player,
            bonus_bets: BonusBets::new(),
            game_mode: GameMode::Classic,
            stats: GameStats::new(),
            show_stats: false,
            animation_state: AnimationState::new(),
            last_update: Instant::now(),
        }
    }
    
    pub async fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let res = self.run_app(&mut terminal).await;
        
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        if let Err(err) = res {
            println!("{err:?}");
        }
        
        Ok(())
    }
    
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;
            
            // Update animation state
            self.animation_state.update();
            
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('p') => self.bet_type = BetType::Player,
                        KeyCode::Char('b') => self.bet_type = BetType::Banker,
                        KeyCode::Char('t') => self.bet_type = BetType::Tie,
                        KeyCode::Char(' ') => {
                            if self.animation_state.is_complete() {
                                self.play_round().await;
                            }
                        }
                        KeyCode::Char('1') => self.current_bet = 10,
                        KeyCode::Char('2') => self.current_bet = 50,
                        KeyCode::Char('3') => self.current_bet = 100,
                        KeyCode::Char('4') => self.current_bet = 500,
                        KeyCode::Char('5') => self.current_bet = 1000,
                        KeyCode::Char('m') => self.cycle_game_mode(),
                        KeyCode::Char('s') => self.show_stats = !self.show_stats,
                        KeyCode::F(1) => self.toggle_bonus_bet("player_pair"),
                        KeyCode::F(2) => self.toggle_bonus_bet("banker_pair"),
                        _ => {}
                    }
                }
            }
        }
    }
    
    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),    // Title
                Constraint::Length(12),   // Cards display
                Constraint::Length(5),    // Betting info
                Constraint::Min(0),       // Stats/Controls
            ])
            .split(f.area());
        
        // Title
        let title = Paragraph::new(format!("BACCARAT - {:?} Mode", self.game_mode))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);
        
        // Cards display
        self.render_cards(f, chunks[1]);
        
        // Betting info
        self.render_betting_info(f, chunks[2]);
        
        // Stats or Controls
        if self.show_stats {
            self.render_stats(f, chunks[3]);
        } else {
            self.render_controls(f, chunks[3]);
        }
    }
    
    fn render_cards(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // Player cards
        let player_display = CardRenderer::create_hand_display(
            &self.game.player_hand,
            "PLAYER".to_string(),
            self.game.state.player_score
        );
        f.render_widget(player_display, chunks[0]);
        
        // Banker cards
        let banker_display = CardRenderer::create_hand_display(
            &self.game.banker_hand,
            "BANKER".to_string(),
            self.game.state.banker_score
        );
        f.render_widget(banker_display, chunks[1]);
    }
    
    fn render_betting_info(&self, f: &mut Frame, area: Rect) {
        let betting_text = vec![
            Line::from(vec![
                Span::raw("Balance: "),
                Span::styled(format!("${}", self.balance), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Main Bet: "),
                Span::styled(
                    format!("${} on {:?}", self.current_bet, self.bet_type),
                    Style::default().fg(Color::Yellow)
                ),
            ]),
            Line::from(vec![
                Span::raw("Bonus Bets: "),
                Span::styled(
                    format!("${}", self.bonus_bets.total_bet()),
                    Style::default().fg(Color::Magenta)
                ),
            ]),
        ];
        
        let betting_info = Paragraph::new(betting_text)
            .block(Block::default().borders(Borders::ALL).title("Betting"));
        f.render_widget(betting_info, area);
    }
    
    fn render_stats(&self, f: &mut Frame, area: Rect) {
        let stats_text = vec![
            Line::from(format!("Rounds Played: {}", self.stats.rounds_played)),
            Line::from(format!("Win Rate: {:.1}%", self.stats.win_rate())),
            Line::from(format!(
                "P: {} | B: {} | T: {}",
                self.stats.player_wins, self.stats.banker_wins, self.stats.ties
            )),
        ];
        
        let stats = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Statistics"));
        f.render_widget(stats, area);
    }
    
    fn render_controls(&self, f: &mut Frame, area: Rect) {
        let controls = vec![
            "[P] Player  [B] Banker  [T] Tie  [M] Mode",
            "[1] $10  [2] $50  [3] $100  [4] $500  [5] $1000",
            "[F1-F2] Bonus Bets  [S] Stats  [SPACE] Deal",
            "[Q/ESC] Quit",
        ];
        
        let controls_text: Vec<Line> = controls.iter()
            .map(|&s| Line::from(s))
            .collect();
        
        let controls_widget = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(controls_widget, area);
    }
    
    fn cycle_game_mode(&mut self) {
        self.game_mode = match self.game_mode {
            GameMode::Classic => GameMode::NoCommission,
            GameMode::NoCommission => GameMode::Speed,
            GameMode::Speed => GameMode::EzBaccarat,
            GameMode::EzBaccarat => GameMode::Classic,
        };
        self.game = BaccaratGame::with_mode(self.game_mode);
    }
    
    fn toggle_bonus_bet(&mut self, bet_type: &str) {
        match bet_type {
            "player_pair" => {
                self.bonus_bets.player_pair = if self.bonus_bets.player_pair > 0 { 0 } else { 5 };
            }
            "banker_pair" => {
                self.bonus_bets.banker_pair = if self.bonus_bets.banker_pair > 0 { 0 } else { 5 };
            }
            _ => {}
        }
    }
    
    async fn play_round(&mut self) {
        if self.current_bet == 0 || self.current_bet > self.balance {
            return;
        }
        
        let total_bet = self.current_bet + self.bonus_bets.total_bet();
        if total_bet > self.balance {
            return;
        }
        
        self.game = BaccaratGame::with_mode(self.game_mode);
        self.game.set_bonus_bets(self.bonus_bets);
        
        // Start animation for Classic mode
        if self.game_mode == GameMode::Classic {
            // Collect all cards that will be dealt
            let mut all_cards = Vec::new();
            
            // We need to simulate the dealing to know what cards will be shown
            // This is a simplified version - in production you'd want to properly
            // integrate this with the game logic
            self.game.play_round();
            
            for card in &self.game.player_hand {
                all_cards.push(*card);
            }
            for card in &self.game.banker_hand {
                all_cards.push(*card);
            }
            
            self.animation_state.start_dealing(all_cards);
        } else {
            // For other modes, deal immediately
            self.game.play_round();
        }
        
        let bet_type_str = match self.bet_type {
            BetType::Player => "player",
            BetType::Banker => "banker",
            BetType::Tie => "tie",
        };
        
        let payout = self.game.total_payout(bet_type_str, self.current_bet);
        
        self.stats.rounds_played += 1;
        self.stats.total_wagered += total_bet;
        self.stats.total_won += payout;
        
        match self.game.state.winner {
            1 => self.stats.player_wins += 1,
            2 => self.stats.banker_wins += 1,
            3 => self.stats.ties += 1,
            _ => {}
        }
        
        self.balance = self.balance - total_bet + payout;
    }
}