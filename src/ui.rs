use crate::baccarat::{BaccaratGame, Card, GameMode, BonusBets, HEARTS, DIAMONDS, CLUBS, SPADES};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, stdout, Write};

pub struct TerminalUI {
    game: BaccaratGame,
    balance: i32,
    current_bet: i32,
    bet_type: BetType,
    bonus_bets: BonusBets,
    game_mode: GameMode,
    statistics: GameStatistics,
    show_statistics: bool,
}

pub struct GameStatistics {
    pub rounds_played: u32,
    pub player_wins: u32,
    pub banker_wins: u32,
    pub ties: u32,
    pub total_wagered: i32,
    pub total_won: i32,
    pub natural_wins: u32,
    pub pair_hits: u32,
}

impl GameStatistics {
    pub fn new() -> Self {
        Self {
            rounds_played: 0,
            player_wins: 0,
            banker_wins: 0,
            ties: 0,
            total_wagered: 0,
            total_won: 0,
            natural_wins: 0,
            pair_hits: 0,
        }
    }
    
    pub fn win_rate(&self) -> f32 {
        if self.rounds_played == 0 {
            return 0.0;
        }
        (self.total_won as f32) / (self.total_wagered as f32) * 100.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BetType {
    Player,
    Banker,
    Tie,
}

impl TerminalUI {
    pub fn new() -> Self {
        Self {
            game: BaccaratGame::new(),
            balance: 1000,
            current_bet: 0,
            bet_type: BetType::Player,
            bonus_bets: BonusBets::new(),
            game_mode: GameMode::Classic,
            statistics: GameStatistics::new(),
            show_statistics: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        
        // Set panic hook to restore terminal
        let default_panic = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = terminal::disable_raw_mode();
            default_panic(info);
        }));

        loop {
            self.draw_screen()?;

            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('p') => self.bet_type = BetType::Player,
                    KeyCode::Char('b') => self.bet_type = BetType::Banker,
                    KeyCode::Char('t') => self.bet_type = BetType::Tie,
                    KeyCode::Char(' ') => self.play_round(),
                    KeyCode::Char('1') => self.current_bet = 10,
                    KeyCode::Char('2') => self.current_bet = 50,
                    KeyCode::Char('3') => self.current_bet = 100,
                    KeyCode::Char('4') => self.current_bet = 500,
                    KeyCode::Char('5') => self.current_bet = 1000,
                    KeyCode::Char('m') => self.cycle_game_mode(),
                    KeyCode::Char('s') => self.show_statistics = !self.show_statistics,
                    KeyCode::F(1) => self.toggle_bonus_bet("player_pair"),
                    KeyCode::F(2) => self.toggle_bonus_bet("banker_pair"),
                    KeyCode::F(3) => self.toggle_bonus_bet("either_pair"),
                    KeyCode::F(4) => self.toggle_bonus_bet("perfect_pair"),
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn draw_screen(&self) -> io::Result<()> {
        let mut stdout = stdout();
        
        // Clear and reset cursor
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        
        // Build screen buffer with \r\n for proper raw mode line endings
        let mut screen = String::new();
        
        screen.push_str("╔════════════════════════════════════════╗\r\n");
        screen.push_str(&format!("║  BACCARAT - {:?} Mode      ║\r\n", self.game_mode));
        screen.push_str("╚════════════════════════════════════════╝\r\n\r\n");
        
        screen.push_str(&format!("Balance: ${}\r\n", self.balance));
        screen.push_str(&format!("Main Bet: ${} on {:?}\r\n", self.current_bet, self.bet_type));
        
        if self.bonus_bets.total_bet() > 0 {
            screen.push_str("Bonus Bets: ");
            if self.bonus_bets.player_pair > 0 {
                screen.push_str(&format!("Player Pair ${} ", self.bonus_bets.player_pair));
            }
            if self.bonus_bets.banker_pair > 0 {
                screen.push_str(&format!("Banker Pair ${} ", self.bonus_bets.banker_pair));
            }
            if self.bonus_bets.either_pair > 0 {
                screen.push_str(&format!("Either Pair ${} ", self.bonus_bets.either_pair));
            }
            if self.bonus_bets.perfect_pair > 0 {
                screen.push_str(&format!("Perfect Pair ${} ", self.bonus_bets.perfect_pair));
            }
            screen.push_str("\r\n");
        }
        screen.push_str("\r\n");
        
        if self.game.state.round_complete == 1 {
            screen.push_str("PLAYER HAND:\r\n");
            for card in &self.game.player_hand {
                screen.push_str(&format!("{} ", self.card_display(card)));
            }
            screen.push_str(&format!(" (Score: {})\r\n", self.game.state.player_score));
            
            screen.push_str("\r\nBANKER HAND:\r\n");
            for card in &self.game.banker_hand {
                screen.push_str(&format!("{} ", self.card_display(card)));
            }
            screen.push_str(&format!(" (Score: {})\r\n", self.game.state.banker_score));
            
            screen.push_str("\r\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\r\n");
            
            match self.game.state.winner {
                1 => screen.push_str(&format!("🎉 PLAYER WINS! (Score: {})\r\n", self.game.state.player_score)),
                2 => screen.push_str(&format!("🎉 BANKER WINS! (Score: {})\r\n", self.game.state.banker_score)),
                3 => screen.push_str(&format!("🤝 TIE! (Both: {})\r\n", self.game.state.player_score)),
                _ => {}
            }
        }
        
        if self.show_statistics && self.statistics.rounds_played > 0 {
            screen.push_str("\r\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\r\n");
            screen.push_str("STATISTICS:\r\n");
            screen.push_str(&format!("  Rounds: {} | Win Rate: {:.1}%\r\n", 
                self.statistics.rounds_played, 
                self.statistics.win_rate()));
            screen.push_str(&format!("  Player Wins: {} | Banker Wins: {} | Ties: {}\r\n",
                self.statistics.player_wins,
                self.statistics.banker_wins,
                self.statistics.ties));
            screen.push_str(&format!("  Natural Wins: {} | Pair Hits: {}\r\n",
                self.statistics.natural_wins,
                self.statistics.pair_hits));
        }
        
        screen.push_str("\r\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\r\n");
        screen.push_str("CONTROLS:\r\n");
        screen.push_str("  [P] Player  [B] Banker  [T] Tie  [M] Change Mode\r\n");
        screen.push_str("  [1] $10  [2] $50  [3] $100  [4] $500  [5] $1000\r\n");
        screen.push_str("  [F1-F4] Toggle Bonus Bets  [S] Stats\r\n");
        screen.push_str("  [SPACE] Deal Cards  [Q/ESC] Quit\r\n");
        
        // Single print command
        execute!(stdout, Print(screen))?;
        stdout.flush()?;
        
        Ok(())
    }


    fn card_display(&self, card: &Card) -> String {
        let suit_symbol = match card.suit {
            HEARTS => "♥",
            DIAMONDS => "♦",
            CLUBS => "♣",
            SPADES => "♠",
            _ => "?",
        };

        let rank_str = match card.rank {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            n => n.to_string(),
        };

        format!("{}{}", rank_str, suit_symbol)
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
            "either_pair" => {
                self.bonus_bets.either_pair = if self.bonus_bets.either_pair > 0 { 0 } else { 5 };
            }
            "perfect_pair" => {
                self.bonus_bets.perfect_pair = if self.bonus_bets.perfect_pair > 0 { 0 } else { 5 };
            }
            _ => {}
        }
    }

    fn play_round(&mut self) {
        if self.current_bet == 0 || self.current_bet > self.balance {
            return;
        }

        let total_bet = self.current_bet + self.bonus_bets.total_bet();
        if total_bet > self.balance {
            return;
        }

        self.game = BaccaratGame::with_mode(self.game_mode);
        self.game.set_bonus_bets(self.bonus_bets);
        self.game.play_round();

        let bet_type_str = match self.bet_type {
            BetType::Player => "player",
            BetType::Banker => "banker",
            BetType::Tie => "tie",
        };

        let payout = self.game.total_payout(bet_type_str, self.current_bet);
        
        self.statistics.rounds_played += 1;
        self.statistics.total_wagered += total_bet;
        self.statistics.total_won += payout;
        
        match self.game.state.winner {
            1 => self.statistics.player_wins += 1,
            2 => self.statistics.banker_wins += 1,
            3 => self.statistics.ties += 1,
            _ => {}
        }
        
        if (self.game.state.player_score >= 8 || self.game.state.banker_score >= 8) 
            && (self.game.player_hand.len() == 2 || self.game.banker_hand.len() == 2) {
            self.statistics.natural_wins += 1;
        }
        
        if self.game.is_player_pair() || self.game.is_banker_pair() {
            self.statistics.pair_hits += 1;
        }

        self.balance = self.balance - total_bet + payout;
    }
}
