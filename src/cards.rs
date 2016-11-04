use rand;
use rand::{Rng};
use std::iter;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Color {
    Red, Yellow, Green, Blue, White
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Number {
    One, Two, Three, Four, Five
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Clone)]
pub struct Card {
    pub color:  Color,
    pub number: Number,
}

impl Card {
    pub fn new(color: Color, number: Number) -> Self {
        Card {
            color:  color,
            number: number,
        }
    }
}

pub struct Deck {
    pub cards: Vec<Card>
}

impl Deck {
    pub fn new() -> Self {
        let mut cards: Vec<Card> = Vec::with_capacity(50);
        let number_arity = &[(3, Number::One), (2, Number::Two), (2, Number::Three), (2, Number::Four), (1, Number::Five)];
        for color in &[Color::Blue, Color::Green, Color::Red, Color::White, Color::Yellow] {
            for &(amount, ref number) in number_arity {
                for card in iter::repeat(Card::new(color.clone(), number.clone())).take(amount) {
                    cards.push(card);
                }
            }
        }
        rand::thread_rng().shuffle(&mut cards);
        Deck { cards: cards }
    }

    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}
