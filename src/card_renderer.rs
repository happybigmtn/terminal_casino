use crate::baccarat::{Card, HEARTS, DIAMONDS, CLUBS, SPADES};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

pub struct CardRenderer;

impl CardRenderer {
    pub fn render_card(card: &Card) -> Vec<String> {
        let rank = Self::rank_symbol(card.rank);
        let suit = Self::suit_symbol(card.suit);
        let _color = Self::suit_color(card.suit);
        
        // ASCII art representation of a card
        vec![
            "┌─────────┐".to_string(),
            format!("│ {:<2}      │", rank),
            "│         │".to_string(),
            format!("│    {}    │", suit),
            "│         │".to_string(),
            format!("│      {:>2} │", rank),
            "└─────────┘".to_string(),
        ]
    }
    
    pub fn render_card_back() -> Vec<String> {
        vec![
            "┌─────────┐".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "│░░░░░░░░░│".to_string(),
            "└─────────┘".to_string(),
        ]
    }
    
    pub fn render_mini_card(card: &Card) -> String {
        let rank = Self::rank_symbol(card.rank);
        let suit = Self::suit_symbol(card.suit);
        format!("[{}{}]", rank, suit)
    }
    
    fn rank_symbol(rank: u8) -> &'static str {
        match rank {
            1 => "A",
            10 => "10",
            11 => "J",
            12 => "Q",
            13 => "K",
            n if n <= 9 => {
                match n {
                    2 => "2",
                    3 => "3",
                    4 => "4",
                    5 => "5",
                    6 => "6",
                    7 => "7",
                    8 => "8",
                    9 => "9",
                    _ => "?",
                }
            }
            _ => "?",
        }
    }
    
    fn suit_symbol(suit: u8) -> &'static str {
        match suit {
            HEARTS => "♥",
            DIAMONDS => "♦",
            CLUBS => "♣",
            SPADES => "♠",
            _ => "?",
        }
    }
    
    fn suit_color(suit: u8) -> Color {
        match suit {
            HEARTS | DIAMONDS => Color::Red,
            CLUBS | SPADES => Color::White,
            _ => Color::Gray,
        }
    }
    
    pub fn create_card_widget(card: &Card) -> Paragraph<'static> {
        let lines = Self::render_card(card);
        let color = Self::suit_color(card.suit);
        
        let text = Text::from(
            lines
                .into_iter()
                .map(|line| Line::from(vec![Span::styled(line, Style::default().fg(color))]))
                .collect::<Vec<_>>()
        );
        
        Paragraph::new(text)
    }
    
    pub fn create_hand_display(cards: &[Card], title: String, score: u8) -> Paragraph<'static> {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("(Score: {})", score), Style::default().fg(Color::Yellow)),
            ])
        ];
        
        if cards.is_empty() {
            lines.push(Line::from("No cards dealt"));
        } else {
            // Add card representations horizontally
            let card_lines: Vec<Vec<String>> = cards.iter()
                .map(|c| Self::render_card(c))
                .collect();
            
            // Merge card lines horizontally
            for row in 0..7 {
                let mut row_text = String::new();
                for (i, card_art) in card_lines.iter().enumerate() {
                    if i > 0 {
                        row_text.push(' ');
                    }
                    row_text.push_str(&card_art[row]);
                }
                lines.push(Line::from(row_text));
            }
        }
        
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL))
    }
}

#[derive(Debug, Clone)]
pub struct CardAnimation {
    pub card: Card,
    pub revealed: bool,
    pub position: usize,
}

impl CardAnimation {
    pub fn new(card: Card, position: usize) -> Self {
        Self {
            card,
            revealed: false,
            position,
        }
    }
    
    pub fn reveal(&mut self) {
        self.revealed = true;
    }
    
    pub fn render(&self) -> Vec<String> {
        if self.revealed {
            CardRenderer::render_card(&self.card)
        } else {
            CardRenderer::render_card_back()
        }
    }
}