use std::fmt::{Display, Formatter, Result};
use cards::{Card, Color, Number, Deck};

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Color::Red    => write!(f, "R"),
            Color::Yellow => write!(f, "Y"),
            Color::Green  => write!(f, "G"),
            Color::Blue   => write!(f, "B"),
            Color::White  => write!(f, "W"),
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Number::One   => write!(f, "1"),
            Number::Two   => write!(f, "2"),
            Number::Three => write!(f, "3"),
            Number::Four  => write!(f, "4"),
            Number::Five  => write!(f, "5"),
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[{}|{}]", self.color, self.number)
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut i = 1;
        if self.cards.is_empty() {
            write!(f, "Empty Deck")
        } else {
            try!(write!(f, "Deck:\n"));
            for ref card in &self.cards {
                try!(write!(f, "{}: {}\n", i, card));
                i += 1;
            }
            Ok(())
        }
    }
}
