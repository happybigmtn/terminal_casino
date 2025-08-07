use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;

pub type Suit = u8;
pub const HEARTS: u8 = 0;
pub const DIAMONDS: u8 = 1;
pub const CLUBS: u8 = 2;
pub const SPADES: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    Classic,
    NoCommission,
    Speed,
    EzBaccarat,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct Card {
    pub suit: Suit,
    pub rank: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct GameState {
    pub player_score: u8,
    pub banker_score: u8,
    pub round_complete: u8, // 0 for ongoing, 1 for complete
    pub winner: u8,         // 0 for none, 1 for player, 2 for banker, 3 for tie
}

impl Card {
    pub fn new(suit: Suit, rank: u8) -> Self {
        Self { suit, rank }
    }

    pub fn baccarat_value(&self) -> u8 {
        match self.rank {
            1..=9 => self.rank,
            _ => 0, // Face cards and tens count as zero in Baccarat
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_score: 0,
            banker_score: 0,
            round_complete: 0,
            winner: 0,
        }
    }

    pub fn calculate_hand_score(cards: &[Card]) -> u8 {
        let total: u8 = cards.iter().map(|card| card.baccarat_value()).sum();
        total % 10
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in 0..4 {
            for rank in 1..=13 {
                cards.push(Card::new(suit, rank));
            }
        }
        Self { cards }
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

pub struct Shoe {
    cards: Vec<Card>,
    num_decks: usize,
    cut_card_position: usize,
    cards_dealt: usize,
}

impl Shoe {
    pub fn new(num_decks: usize) -> Self {
        let mut cards = Vec::with_capacity(52 * num_decks);
        for _ in 0..num_decks {
            for suit in 0..4 {
                for rank in 1..=13 {
                    cards.push(Card::new(suit, rank));
                }
            }
        }
        
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        cards.shuffle(&mut rng);
        
        let cut_card_position = cards.len() - (cards.len() / 10).max(15);
        
        Self {
            cards,
            num_decks,
            cut_card_position,
            cards_dealt: 0,
        }
    }
    
    pub fn deal(&mut self) -> Option<Card> {
        if self.cards.is_empty() {
            return None;
        }
        self.cards_dealt += 1;
        self.cards.pop()
    }
    
    pub fn needs_reshuffle(&self) -> bool {
        self.cards.len() <= (52 * self.num_decks) - self.cut_card_position
    }
    
    pub fn reshuffle(&mut self) {
        *self = Self::new(self.num_decks);
    }
    
    pub fn cards_remaining(&self) -> usize {
        self.cards.len()
    }
}

pub enum CardSource {
    SingleDeck(Deck),
    Shoe(Shoe),
}

impl CardSource {
    pub fn deal(&mut self) -> Option<Card> {
        match self {
            CardSource::SingleDeck(deck) => deck.deal(),
            CardSource::Shoe(shoe) => shoe.deal(),
        }
    }
    
    pub fn needs_reshuffle(&self) -> bool {
        match self {
            CardSource::SingleDeck(deck) => deck.cards.len() < 6,
            CardSource::Shoe(shoe) => shoe.needs_reshuffle(),
        }
    }
    
    pub fn reshuffle(&mut self) {
        match self {
            CardSource::SingleDeck(deck) => {
                *deck = Deck::new();
                deck.shuffle();
            }
            CardSource::Shoe(shoe) => shoe.reshuffle(),
        }
    }
}

pub struct BaccaratGame {
    pub card_source: CardSource,
    pub player_hand: Vec<Card>,
    pub banker_hand: Vec<Card>,
    pub state: GameState,
    pub mode: GameMode,
    pub bonus_bets: BonusBets,
}

impl BaccaratGame {
    pub fn new() -> Self {
        Self::with_mode(GameMode::Classic)
    }

    pub fn with_mode(mode: GameMode) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        Self {
            card_source: CardSource::SingleDeck(deck),
            player_hand: Vec::new(),
            banker_hand: Vec::new(),
            state: GameState::new(),
            mode,
            bonus_bets: BonusBets::new(),
        }
    }
    
    pub fn with_shoe(mode: GameMode, num_decks: usize) -> Self {
        Self {
            card_source: CardSource::Shoe(Shoe::new(num_decks)),
            player_hand: Vec::new(),
            banker_hand: Vec::new(),
            state: GameState::new(),
            mode,
            bonus_bets: BonusBets::new(),
        }
    }

    pub fn deal_initial_cards(&mut self) {
        self.player_hand.push(self.card_source.deal().unwrap());
        self.banker_hand.push(self.card_source.deal().unwrap());
        self.player_hand.push(self.card_source.deal().unwrap());
        self.banker_hand.push(self.card_source.deal().unwrap());
        self.update_scores();
    }

    fn update_scores(&mut self) {
        self.state.player_score = GameState::calculate_hand_score(&self.player_hand);
        self.state.banker_score = GameState::calculate_hand_score(&self.banker_hand);
    }

    pub fn play_round(&mut self) {
        self.deal_initial_cards();
        if self.state.player_score >= 8 || self.state.banker_score >= 8 {
            self.determine_winner();
            return;
        }

        let player_third_card = if self.state.player_score <= 5 {
            let card = self.card_source.deal().unwrap();
            self.player_hand.push(card);
            self.update_scores();
            Some(card.baccarat_value())
        } else {
            None
        };

        self.banker_draw_logic(player_third_card);

        self.determine_winner();
    }

    fn banker_draw_logic(&mut self, player_third_value: Option<u8>) {
        let should_draw = match self.state.banker_score {
            0..=2 => true,
            3 => player_third_value != Some(8),
            4 => matches!(player_third_value, Some(2..=7)),
            5 => matches!(player_third_value, Some(4..=7)),
            6 => matches!(player_third_value, Some(6 | 7)),
            _ => false,
        };

        if should_draw {
            self.banker_hand.push(self.card_source.deal().unwrap());
            self.update_scores();
        }
    }

    fn determine_winner(&mut self) {
        self.state.round_complete = 1;

        self.state.winner = if self.state.player_score > self.state.banker_score {
            1 // Player wins
        } else if self.state.banker_score > self.state.player_score {
            2 // Banker wins
        } else {
            3 // Tie
        };
    }

    pub fn is_player_pair(&self) -> bool {
        self.player_hand.len() >= 2 && self.player_hand[0].rank == self.player_hand[1].rank
    }

    pub fn victory_margin(&self) -> u8 {
        if self.state.winner == 0 || self.state.winner == 3 {
            0
        } else {
            let higher = self.state.player_score.max(self.state.banker_score);
            let lower = self.state.player_score.min(self.state.banker_score);
            higher - lower
        }
    }

    pub fn is_banker_pair(&self) -> bool {
        self.banker_hand.len() >= 2 && self.banker_hand[0].rank == self.banker_hand[1].rank
    }

    pub fn is_either_pair(&self) -> bool {
        self.is_player_pair() || self.is_banker_pair()
    }

    pub fn is_perfect_pair(&self) -> bool {
        let player_perfect = self.player_hand.len() >= 2 
            && self.player_hand[0].rank == self.player_hand[1].rank 
            && self.player_hand[0].suit == self.player_hand[1].suit;
        
        let banker_perfect = self.banker_hand.len() >= 2 
            && self.banker_hand[0].rank == self.banker_hand[1].rank 
            && self.banker_hand[0].suit == self.banker_hand[1].suit;
        
        player_perfect || banker_perfect
    }

    pub fn calculate_main_bet_payout(&self, bet_type: &str, bet_amount: i32) -> i32 {
        match self.mode {
            GameMode::Classic => self.classic_payout(bet_type, bet_amount),
            GameMode::NoCommission => self.no_commission_payout(bet_type, bet_amount),
            GameMode::Speed => self.speed_payout(bet_type, bet_amount),
            GameMode::EzBaccarat => self.ez_baccarat_payout(bet_type, bet_amount),
        }
    }

    fn classic_payout(&self, bet_type: &str, bet_amount: i32) -> i32 {
        match (bet_type, self.state.winner) {
            ("player", 1) => bet_amount * 2,
            ("banker", 2) => (bet_amount as f32 * 1.95) as i32,
            ("tie", 3) => bet_amount * 9,
            _ => 0,
        }
    }

    fn no_commission_payout(&self, bet_type: &str, bet_amount: i32) -> i32 {
        match (bet_type, self.state.winner) {
            ("player", 1) => bet_amount * 2,
            ("banker", 2) => {
                if self.state.banker_score == 6 {
                    (bet_amount as f32 * 1.5) as i32
                } else {
                    bet_amount * 2
                }
            }
            ("tie", 3) => bet_amount * 9,
            _ => 0,
        }
    }

    fn speed_payout(&self, bet_type: &str, bet_amount: i32) -> i32 {
        match (bet_type, self.state.winner) {
            ("player", 1) => bet_amount * 2,
            ("banker", 2) => bet_amount * 2,
            ("tie", 3) => bet_amount * 8,
            _ => 0,
        }
    }

    fn ez_baccarat_payout(&self, bet_type: &str, bet_amount: i32) -> i32 {
        match (bet_type, self.state.winner) {
            ("player", 1) => bet_amount * 2,
            ("banker", 2) => {
                if self.banker_hand.len() == 3 
                    && self.state.banker_score == 7 
                    && self.banker_hand.iter().all(|c| c.baccarat_value() == 0 || c.baccarat_value() >= 10) {
                    bet_amount
                } else {
                    bet_amount * 2
                }
            }
            ("tie", 3) => bet_amount * 9,
            ("dragon7", 2) if self.is_dragon_7() => bet_amount * 40,
            ("panda8", 1) if self.is_panda_8() => bet_amount * 25,
            _ => 0,
        }
    }

    pub fn is_dragon_7(&self) -> bool {
        self.state.winner == 2 
            && self.state.banker_score == 7 
            && self.banker_hand.len() == 3
    }

    pub fn is_panda_8(&self) -> bool {
        self.state.winner == 1 
            && self.state.player_score == 8 
            && self.player_hand.len() == 3
    }

    pub fn set_bonus_bets(&mut self, bets: BonusBets) {
        self.bonus_bets = bets;
    }

    pub fn total_payout(&self, main_bet_type: &str, main_bet_amount: i32) -> i32 {
        let main_payout = self.calculate_main_bet_payout(main_bet_type, main_bet_amount);
        let bonus_payout = self.bonus_bets.calculate_payouts(self);
        main_payout + bonus_payout
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BonusBets {
    pub player_pair: u8,
    pub banker_pair: u8,
    pub either_pair: u8,
    pub perfect_pair: u8,
    pub player_dragon: u8,
    pub banker_dragon: u8,
    pub lucky_6: u8,
}

impl BonusBets {
    pub fn new() -> Self {
        Self {
            player_pair: 0,
            banker_pair: 0,
            either_pair: 0,
            perfect_pair: 0,
            player_dragon: 0,
            banker_dragon: 0,
            lucky_6: 0,
        }
    }
    pub fn calculate_payouts(&self, game: &BaccaratGame) -> i32 {
        let mut total_payout = 0;

        if self.player_pair > 0 && game.is_player_pair() {
            total_payout += (self.player_pair as i32) * 11;
        }

        if self.banker_pair > 0 && game.is_banker_pair() {
            total_payout += (self.banker_pair as i32) * 11;
        }

        if self.either_pair > 0 && game.is_either_pair() {
            total_payout += (self.either_pair as i32) * 5;
        }

        if self.perfect_pair > 0 && game.is_perfect_pair() {
            total_payout += (self.perfect_pair as i32) * 25;
        }

        if self.player_dragon > 0 && game.state.winner == 1 {
            let margin = game.victory_margin();
            let payout_ratio = match margin {
                9 => 30,
                8 => 10,
                7 => 6,
                6 => 4,
                5 => 2,
                4 => 1,
                _ => 0,
            };
            if payout_ratio > 0 {
                total_payout += (self.player_dragon as i32) * payout_ratio;
            }
        }

        if self.banker_dragon > 0 && game.state.winner == 2 {
            let margin = game.victory_margin();
            let payout_ratio = match margin {
                9 => 30,
                8 => 10,
                7 => 6,
                6 => 4,
                5 => 2,
                4 => 1,
                _ => 0,
            };
            if payout_ratio > 0 {
                total_payout += (self.banker_dragon as i32) * payout_ratio;
            }
        }

        if self.lucky_6 > 0 && game.state.winner == 2 && game.state.banker_score == 6 {
            let payout_ratio = if game.banker_hand.len() == 3 { 20 } else { 12 };
            total_payout += (self.lucky_6 as i32) * payout_ratio;
        }

        total_payout
    }

    pub fn total_bet(&self) -> i32 {
        (self.player_pair
            + self.banker_pair
            + self.either_pair
            + self.perfect_pair
            + self.player_dragon
            + self.banker_dragon
            + self.lucky_6) as i32
    }
}

pub struct BettingRound {
    pub main_bet_type: String,
    pub main_bet_amount: i32,
    pub bonus_bets: BonusBets,
    pub balance: i32,
    pub round_stats: RoundStatistics,
}

pub struct RoundStatistics {
    pub hands_played: u32,
    pub amount_wagered: i32,
    pub amount_won: i32,
    pub bonus_hits: HashMap<String, u32>,
}

impl RoundStatistics {
    pub fn new() -> Self {
        Self {
            hands_played: 0,
            amount_wagered: 0,
            amount_won: 0,
            bonus_hits: HashMap::new(),
        }
    }
    
    pub fn record_bonus_hit(&mut self, bonus_type: &str) {
        *self.bonus_hits.entry(bonus_type.to_string()).or_insert(0) += 1;
    }
}

impl BettingRound {
    pub fn new(balance: i32) -> Self {
        Self {
            main_bet_type: String::new(),
            main_bet_amount: 0,
            bonus_bets: BonusBets::new(),
            balance,
            round_stats: RoundStatistics::new(),
        }
    }
    
    pub fn place_main_bet(&mut self, bet_type: &str, amount: i32) -> Result<(), &str> {
        if amount > self.balance {
            return Err("Insufficient balance");
        }
        
        if amount <= 0 {
            return Err("Bet amount must be positive");
        }
        
        if !["player", "banker", "tie"].contains(&bet_type) {
            return Err("Invalid bet type");
        }
        
        self.main_bet_type = bet_type.to_string();
        self.main_bet_amount = amount;
        Ok(())
    }
    
    pub fn place_bonus_bet(&mut self, bet_type: &str, amount: u8) -> Result<(), &str> {
        let total_bet = self.main_bet_amount + self.bonus_bets.total_bet() + amount as i32;
        
        if total_bet > self.balance {
            return Err("Insufficient balance for bonus bet");
        }
        
        match bet_type {
            "player_pair" => self.bonus_bets.player_pair = amount,
            "banker_pair" => self.bonus_bets.banker_pair = amount,
            "either_pair" => self.bonus_bets.either_pair = amount,
            "perfect_pair" => self.bonus_bets.perfect_pair = amount,
            "player_dragon" => self.bonus_bets.player_dragon = amount,
            "banker_dragon" => self.bonus_bets.banker_dragon = amount,
            "lucky_6" => self.bonus_bets.lucky_6 = amount,
            _ => return Err("Invalid bonus bet type"),
        }
        
        Ok(())
    }
    
    pub fn settle_round(&mut self, game: &BaccaratGame) -> i32 {
        let total_bet = self.main_bet_amount + self.bonus_bets.total_bet();
        let payout = game.total_payout(&self.main_bet_type, self.main_bet_amount);
        
        self.balance = self.balance - total_bet + payout;
        self.round_stats.hands_played += 1;
        self.round_stats.amount_wagered += total_bet;
        self.round_stats.amount_won += payout;
        
        if game.is_player_pair() && self.bonus_bets.player_pair > 0 {
            self.round_stats.record_bonus_hit("player_pair");
        }
        if game.is_banker_pair() && self.bonus_bets.banker_pair > 0 {
            self.round_stats.record_bonus_hit("banker_pair");
        }
        
        payout
    }
}
