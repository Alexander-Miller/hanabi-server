use std::fmt::{Display, Formatter, Result};
use cards::{Card, Color, Number, CardKnowledge};
use responses::ResponseType;
use game_state::Player;

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

impl Display for ResponseType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            ResponseType::ConnectionResponseType  => write!(f, "Connection Response Type"),
            ResponseType::DiscardCardResponseType => write!(f, "Discard Card Response Type"),
            ResponseType::ErrorResponseType       => write!(f, "Error Response Type"),
            ResponseType::GameOverResponseType    => write!(f, "Game Over Response Type"),
            ResponseType::HintColorResposeType    => write!(f, "Hint Color Respose Type"),
            ResponseType::HintNumberResposeType   => write!(f, "Hint Number Respose Type"),
            ResponseType::PlayCardResponseType    => write!(f, "Play Card Response Type"),
            ResponseType::GameStartResponseType   => write!(f, "Game Start Response Type"),
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}:[{}|{}]", self.id, self.color, self.number)
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Player {}:", self.name).unwrap();
        for cih in &self.cards {
            write!(f, "\nCard {} with Knowledge {}", cih.card, cih.knowledge).unwrap();
        }
        Ok(())
    }
}

impl Display for CardKnowledge {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Knows Color: {}, Knows Number: {}, Excluded Colors: {:?}, Excluded Numbers: {:?}",
               self.knows_color,
               self.knows_number,
               self.knows_color_not.iter().map(|c| format!("{}", c)).collect::<Vec<_>>(),
               self.knows_number_not.iter().map(|n| format!("{}", n)).collect::<Vec<_>>())
    }
}
