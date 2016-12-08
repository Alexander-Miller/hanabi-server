use rand;
use rand::{Rng};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Color {
    Red, Yellow, Green, Blue, White
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Number {
    One, Two, Three, Four, Five
}

impl Number {
    pub fn is_next_largest(this: Option<&Number>, that: &Number) -> bool {
        match (this, that) {
            (None,                 &Number::One)   => true,
            (Some(&Number::One),   &Number::Two)   => true,
            (Some(&Number::Two),   &Number::Three) => true,
            (Some(&Number::Three), &Number::Four)  => true,
            (Some(&Number::Four),  &Number::Five)  => true,
            _                                      => false,
        }
    }

    pub fn score(&self) -> usize {
        match *self {
            Number::One   => 1,
            Number::Two   => 2,
            Number::Three => 3,
            Number::Four  => 4,
            Number::Five  => 5,
        }
    }
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Clone)]
pub struct Card {
    pub id:     u8,
    pub color:  Color,
    pub number: Number,
}

impl Card {
    pub fn new(id: u8, color: Color, number: Number) -> Self {
        Card {
            id:     id,
            color:  color,
            number: number,
        }
    }
}

#[derive(RustcEncodable)]
pub struct Deck {
    pub cards: Vec<Card>
}

impl Deck {
    pub fn new() -> Self {
        let mut cards: Vec<Card> = Vec::with_capacity(50);
        let mut id = 1;
        let number_arity = &[(3, Number::One), (2, Number::Two), (2, Number::Three), (2, Number::Four), (1, Number::Five)];
        for color in &[Color::Blue, Color::Green, Color::Red, Color::White, Color::Yellow] {
            for &(amount, ref number) in number_arity {
                for _ in 0..amount {
                    cards.push(Card::new(id, color.clone(), number.clone()));
                    id += 1;
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
