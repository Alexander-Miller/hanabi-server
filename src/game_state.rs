use std::collections::HashSet;
use std::collections::HashMap;
use std::mem;
use std::default::Default;
use cards::{Color, Number, Card, Deck};
use requests::{HintNumberRequest, HintColorRequest, PlayCardRequest};
use responses::error_messages::*;

pub struct CardKnowledge {
    knows_color:      bool,
    knows_number:     bool,
    knows_color_not:  HashSet<Color>,
    knows_number_not: HashSet<Number>,
}

impl CardKnowledge {
    pub fn new() -> Self {
        CardKnowledge {
            knows_color:      false,
            knows_number:     false,
            knows_color_not:  HashSet::new(),
            knows_number_not: HashSet::new(),
        }
    }
}

pub struct CardInHand {
    card:      Card,
    knowledge: CardKnowledge,
}

impl CardInHand {
    pub fn new(card: Card) -> Self {
        CardInHand {
            card:      card,
            knowledge: CardKnowledge::new(),
        }
    }
}

pub struct Player {
    name:  String,
    cards: Vec<CardInHand>,
}

impl Player {
    pub fn new(name: String, cards: Vec<CardInHand>) -> Self {
        Player {
            name:  name,
            cards: cards,
        }
    }

    pub fn discard_card(&mut self, discarded_card: &Card, new_card: Option<Card>) -> Result<(), &'static str> {
        match self.cards.iter().position(|hc| hc.card == *discarded_card) {
            None     => Err(TODO),
            Some(i)  => {
                match new_card {
                    Some(card) => {
                        let mut new_card_in_hand = CardInHand::new(card);
                        mem::swap(&mut new_card_in_hand, &mut self.cards[i]);
                    }
                    None => {
                        self.cards.remove(i);
                    }
                }
                Ok(())
            }
        }
    }

}

pub struct GameState {
    hint_tokens:     u8,
    hint_tokens_max: u8,
    err_tokens:      u8,
    deck:            Deck,
    played_cards:    HashMap<Color, Number>,
    players:         HashMap<u8, Player>
}

impl Default for GameState {
    fn default() -> Self {
        GameState::new(7, 3)
    }
}

impl GameState {
    pub fn new(hint_tokens_max: u8, err_tokens: u8) -> Self {
        GameState {
            hint_tokens:     hint_tokens_max,
            hint_tokens_max: hint_tokens_max,
            err_tokens:      err_tokens,
            deck:            Deck::new(),
            played_cards:    HashMap::new(),
            players:         HashMap::new(),
        }
    }

    pub fn already_connected(&mut self, id: u8) -> bool {
        self.player_by_id(id).is_some()
    }

    pub fn add_player(&mut self, id: u8, name: &str) -> Result<(), &'static str> {
        if self.deck.cards.len() < 5 {
            return Err(NO_CARDS);
        }

        if self.players.values().any(|p| p.name == name) {
            return Err(PLAYER_ALREADY_EXISTS);
        }

        let cards = (0..5)
            .map(|_| CardInHand::new(self.deck.cards.pop().unwrap()))
            .collect();
        self.players.insert(id, Player::new(name.into(), cards));
        Ok(())
    }

    pub fn discard_card(&mut self, discarding_player_id: u8, discarded_card: &Card) -> Result<(), &'static str> {
        self.players.get_mut(&discarding_player_id).unwrap().discard_card(&discarded_card, self.deck.pop())
    }

    pub fn hint_color(&mut self, req: &HintColorRequest) -> Result<(), &'static str> {
        if let Some(player) = self.players.values_mut().find(|p| p.name == req.target_player) {
            if self.hint_tokens < 1 {
                return Err(TODO);
            } else {
                self.hint_tokens -= 1;
            }

            match req.positive {
                true => {
                    for card_in_hand in player.cards.iter_mut().filter(|cih| cih.card.color == req.color) {
                        card_in_hand.knowledge.knows_color = true;
                    }
                    // TODO: implicit know_not?
                }
                false => {
                    for card_in_hand in player.cards.iter_mut().filter(|cih| cih.card.color != req.color) {
                        card_in_hand.knowledge.knows_color_not.insert(req.color.clone());
                    }
                }
            }
            Ok(())
        } else {
            return Err(TODO);
        }
    }


    pub fn hint_number(&mut self, req: &HintNumberRequest) -> Result<(), &'static str> {
        if let Some(player) = self.players.values_mut().find(|p| p.name == req.target_player) {
            if self.hint_tokens < 1 {
                return Err(TODO);
            } else {
                self.hint_tokens -= 1;
            }

            match req.positive {
                true => {
                    for card_in_hand in player.cards.iter_mut().filter(|cih| cih.card.number == req.number) {
                        card_in_hand.knowledge.knows_number = true;
                    }
                    // TODO: implicit know_not?
                }
                false => {
                    for card_in_hand in player.cards.iter_mut().filter(|cih| cih.card.number != req.number) {
                        card_in_hand.knowledge.knows_number_not.insert(req.number.clone());
                    }
                }
            }

            Ok(())
        } else {
            return Err(TODO);
        }
    }

    pub fn play_card(&mut self, playing_player_id: u8, req: &PlayCardRequest) -> Result<(), &'static str> {
        Ok(())
    }

    fn player_by_id(&mut self, id: u8) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

}
