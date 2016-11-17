use std::collections::HashSet;
use std::collections::HashMap;
use std::mem;
use std::default::Default;
use cards::{Color, Number, Card, Deck};
use requests::{HintNumberRequest, HintColorRequest, PlayCardRequest, DiscardCardRequest};
use responses::error_messages::*;

pub type Void = ();

#[derive(RustcEncodable)]
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

#[derive(RustcEncodable)]
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

#[derive(RustcEncodable)]
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
}

#[derive(RustcEncodable)]
pub struct GameState {
    hint_tokens:     u8,
    hint_tokens_max: u8,
    err_tokens:      u8,
    deck:            Deck,
    played_cards:    HashMap<Color, Number>,
    players:         Vec<Player>,
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
            players:         Vec::with_capacity(6),
        }
    }

    pub fn add_player(&mut self, name: &str) -> Result<Void, &'static str> {
        if self.deck.cards.len() < 5 {
            return Err(NO_CARDS);
        }

        if self.players.iter().any(|p| p.name == name) {
            return Err(PLAYER_ALREADY_EXISTS);
        }

        let cards = self.deck.cards
            .drain(0..5)
            .map(|c| CardInHand::new(c))
            .collect();
        self.players.push(Player::new(name.into(), cards));
        Ok(())
    }

    pub fn discard_card(&mut self, name: &str, discard_req: &DiscardCardRequest) -> CardDrawingResult {
        let mut player = self.players.iter_mut().find(|p| p.name == name).unwrap();
        match player.cards.iter().position(|hc| hc.card == discard_req.discarded_card) {
            None => {
                error!("Card {} could not be found.", discard_req.discarded_card);
                CardDrawingResult::Err(CARD_NOT_FOUND)
            }
            Some(i) => {
                self.hint_tokens += 1;
                match self.deck.pop() {
                    Some(card) => {
                        let mut new_card_in_hand = CardInHand::new(card);
                        mem::swap(&mut new_card_in_hand, &mut player.cards[i]);
                        CardDrawingResult::Ok(false)
                    }
                    None => {
                        player.cards.remove(i);
                        CardDrawingResult::Ok(true)
                    }
                }
            }
        }
    }

    pub fn hint_color(&mut self, name: &str, req: &HintColorRequest) -> Result<Void, &'static str> {
        self.knowledge_update(&name,
                              req.positive,
                              &|c| { c.card.color == req.color },
                              &|c| { c.knowledge.knows_color = true },
                              &|c| { c.knowledge.knows_color_not.insert(req.color.clone()); })
    }

    pub fn hint_number(&mut self, name: &str, req: &HintNumberRequest) -> Result<Void, &'static str> {
        self.knowledge_update(&name,
                              req.positive,
                              &|c| { c.card.number == req.number },
                              &|c| { c.knowledge.knows_number = true },
                              &|c| { c.knowledge.knows_number_not.insert(req.number.clone()); })
    }

    fn knowledge_update(&mut self,
                        name: &str,
                        hint_is_positive: bool,
                        predicate: &Fn(&CardInHand) -> bool,
                        update_positive: &Fn(&mut CardInHand),
                        update_negative: &Fn(&mut CardInHand))
                        -> Result<Void, &'static str>
    {
        try!(self.use_hint());
        let mut player = self.player_by_name(&name);
        if hint_is_positive {
            for mut card_in_hand in player.cards.iter_mut().filter(|c| predicate(c)) {
                update_positive(&mut card_in_hand);
            }
        } else {
            for mut card_in_hand in player.cards.iter_mut().filter(|c| !predicate(c)) {
                update_negative(&mut card_in_hand);
            }
        }
        Ok(())
    }

    pub fn play_card(&mut self, name: &str, req: &PlayCardRequest) -> CardPlayingResult {
        let mut player = self.players.iter_mut().find(|p| p.name == name).unwrap();
        if let Some(i) = player.cards.iter().position(|cih| cih.card == req.played_card) {
            match self.deck.pop() {
                Some(card) => {
                    let mut new_card_in_hand = CardInHand::new(card);
                    mem::swap(&mut new_card_in_hand, &mut player.cards[i]);
                }
                None => {
                    player.cards.remove(i);
                }
            }
            match Number::is_next_largest(self.played_cards.get(&req.played_card.color), &req.played_card.number) {
                true =>  {
                    self.played_cards.insert(req.played_card.color.clone(), req.played_card.number.clone());
                    return CardPlayingResult::Success;
                }
                false => {
                    self.err_tokens -= 1;
                    return CardPlayingResult::Failure;
                }
            }
        } else {
            return CardPlayingResult::Err(CARD_NOT_FOUND);
        }
    }

    fn player_by_name(&mut self, name: &str) -> &mut Player {
        self.players.iter_mut().find(|p| p.name == name).unwrap()
    }

    fn use_hint(&mut self) -> Result<Void, &'static str> {
        if self.hint_tokens < 1 {
            Err(NO_HINT_TOKENS)
        } else {
            self.hint_tokens -= 1;
            Ok(())
        }
    }
}

pub enum CardDrawingResult {
    Ok(bool),
    Err(&'static str),
}

pub enum CardPlayingResult {
    Success,
    Failure,
    Err(&'static str),
}
