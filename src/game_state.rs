use cards::{Deck, Card, Color, Number, CardInHand};
use responses::error_messages::*;

use std::collections::HashMap;
use std::mem;
use std::default::Default;

pub type Void = ();

const CARDS_IN_DECK:       usize = 50;
const DEFAULT_HINT_TOKENS: usize = 8;
const DEFAULT_ERR_TOKENS:  usize = 3;

#[derive(RustcEncodable)]
pub struct Player {
    pub name:  String,
    pub cards: Vec<CardInHand>,
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
    hint_tokens:     usize,
    hint_tokens_max: usize,
    err_tokens:      usize,
    played_cards:    HashMap<Color, Number>,
    players:         Vec<Player>,
    deck:            Deck,
    discarded_cards: Vec<Card>,
    next_player:     String,
    turns_left:      usize,
}

impl Default for GameState {
    fn default() -> Self {
        debug!("Creating default game state.");
        GameState::new(DEFAULT_HINT_TOKENS, DEFAULT_ERR_TOKENS)
    }
}

impl GameState {

    pub fn new(hint_tokens_max: usize, err_tokens: usize) -> Self {
        debug!("Creating new game state instace.");
        GameState {
            hint_tokens:     hint_tokens_max,
            hint_tokens_max: hint_tokens_max,
            err_tokens:      err_tokens,
            played_cards:    HashMap::new(),
            players:         Vec::with_capacity(6),
            deck:            Deck::new(),
            discarded_cards: Vec::with_capacity(CARDS_IN_DECK),
            next_player:     String::new(),
            turns_left:      CARDS_IN_DECK,
        }
    }

    pub fn add_player(&mut self, name: &str) -> Result<Void, &'static str> {
        info!("Adding new player \"{}\".", name);
        if self.deck.cards.len() < 5 {
            error!("Not enough cards for new player.");
            return Err(NO_CARDS);
        }

        if self.players.iter().any(|p| p.name == name) {
            error!("Player already exists.");
            return Err(PLAYER_ALREADY_EXISTS);
        }

        self.turns_left += 1;

        let cards = self.deck.cards
            .drain(0..5)
            .map(|c| CardInHand::new(c))
            .collect();
        self.players.push(Player::new(name.into(), cards));

        debug!("Added new player: {}", self.players[self.players.len()-1]);
        debug!("Number of players increased to {:?}", self.turns_left - CARDS_IN_DECK);

        Ok(())
    }

    pub fn discard_card(&mut self, name: &str, discarded_card_id: usize) -> Result<Void, &'static str> {
        info!("Discarding card with id {} of player {}.", discarded_card_id, name);

        if let Some(p_index) = self.player_index(name) {
            if let Some(c_index) = self.card_index(p_index, discarded_card_id) {
                return self.do_discard_card(p_index, c_index)
            } else {
                error!("Could not find card with id {} on the hand of player {}", discarded_card_id, name);
                return Err(CARD_NOT_FOUND)
            }
        } else {
            error!("Could not find player {}", name);
            return Err(PLAYER_NOT_FOUND)
        }
    }

    fn do_discard_card(&mut self, p_index: usize, c_index: usize) -> Result<Void, &'static str> {
        self.maybe_draw_new_card(p_index, c_index);

        if self.hint_tokens < self.hint_tokens_max {
            self.hint_tokens += 1;
        }

        Ok(())
    }

    pub fn play_card(&mut self, name: &str, played_card_id: usize) -> CardPlayingResult {
        debug!("Playing card with id {} of player {}", played_card_id, name);

        if let Some(p_index) = self.player_index(name) {
            if let Some(c_index) = self.card_index(p_index, played_card_id) {
                return self.do_play_card(p_index, c_index)
            } else {
                return CardPlayingResult::Err(CARD_NOT_FOUND)
            }
        } else {
            return CardPlayingResult::Err(PLAYER_NOT_FOUND)
        }
    }

    fn do_play_card(&mut self, p_index: usize, c_index: usize) -> CardPlayingResult {
        self.maybe_draw_new_card(p_index, c_index);

        let played_card = &self.discarded_cards[self.discarded_cards.len() - 1];

        if Number::is_next_largest(self.played_cards.get(&played_card.color), &played_card.number) {
            self.played_cards.insert(played_card.color.clone(), played_card.number.clone());
            debug!("Play card success. Currently played cards: {:?}",
                   self.played_cards.iter().map(|(color, number)| format!("{}: {}\n", color, number)).collect::<Vec<_>>());
            if played_card.number == Number::Five && self.hint_tokens < self.hint_tokens_max {
                self.hint_tokens += 1;
                debug!("Played a Five - number of hint tokens increased to {}.", self.hint_tokens);
            }
            return CardPlayingResult::Success;
        } else {
            self.err_tokens -= 1;
            debug!("Play card fail. {} err tokens left.", self.err_tokens);
            match self.err_tokens {
                0 => return CardPlayingResult::EpicFail,
                _ => return CardPlayingResult::Failure,
            }
        }
    }

    fn maybe_draw_new_card(&mut self, p_index: usize, c_index: usize) {
        self.set_next_player();
        self.turns_left -= 1;
        let mut hand = &mut self.players[p_index].cards;
        match self.deck.pop() {
            Some(new_card) => {
                debug!("Card removed and {} drawn as replacement", new_card);
                let mut new_card_in_hand = CardInHand::new(new_card);
                mem::swap(&mut new_card_in_hand, &mut hand[c_index]);
                self.discarded_cards.push(new_card_in_hand.card);
            }
            None => {
                debug!("Deck is empty and removed card will not be replaced.");
                self.discarded_cards.push(hand.remove(c_index).card);
            }
        }
    }

    pub fn hint_color(&mut self, name: &str, color: &Color) -> Result<Void, &'static str> {
        debug!("Hinting color {} for player {}.", color, name);
        self.knowledge_update(name,
                              &|c| { c.card.color == *color },
                              &|c| { c.knowledge.knows_color = true; c.knowledge.knows_color_not.clear(); },
                              &|c| { c.knowledge.knows_color_not.insert(color.clone()); })
    }

    pub fn hint_number(&mut self, name: &str, number: &Number) -> Result<Void, &'static str> {
        debug!("Hinting number {} for player {}.", number, name);
        self.knowledge_update(name,
                              &|c| { c.card.number == *number },
                              &|c| { c.knowledge.knows_number = true; c.knowledge.knows_number_not.clear(); },
                              &|c| { c.knowledge.knows_number_not.insert(number.clone()); })
    }

    fn knowledge_update(&mut self,
                        name: &str,
                        predicate: &Fn(&CardInHand) -> bool,
                        update_positive: &Fn(&mut CardInHand),
                        update_negative: &Fn(&mut CardInHand))
                        -> Result<Void, &'static str>
    {
        debug!("Update knowledge for player {}.", name);
        if let Some(p_index) = self.players.iter_mut().position(|p| p.name == name) {
            try!(self.use_hint());
            self.set_next_player();

            if self.deck.cards.is_empty() {
                self.turns_left -= 1;
            }

            for mut card_in_hand in &mut self.players[p_index].cards {
                match predicate(&card_in_hand) {
                    true  => update_positive(&mut card_in_hand),
                    false => update_negative(&mut card_in_hand),
                }
            }
            return Ok(())
        } else {
            return Err(PLAYER_NOT_FOUND)
        }
    }

    fn set_next_player(&mut self) {
        match self.next_player.as_str() {
            "" => self.next_player = self.players[0].name.clone(),
            _  => {
                let index = (self.players.iter().position(|p| p.name == self.next_player).unwrap() + 1) % self.players.len();
                self.next_player = self.players[index].name.clone();
            }
        }
        debug!("Set next player to {:?}", self.next_player);
    }

    fn use_hint(&mut self) -> Result<Void, &'static str> {
        if self.hint_tokens <= 0 {
            error!("Cannot use hint without hint tokens.");
            Err(NO_HINT_TOKENS)
        } else {
            self.hint_tokens -= 1;
            debug!("Hint used, {} tokens left.", self.hint_tokens);
            Ok(())
        }
    }

    fn player_index(&self, name: &str) -> Option<usize> {
        self.players.iter().position(|p| p.name == name)
    }

    fn card_index(&self, p_index: usize, id: usize) -> Option<usize> {
        self.players[p_index].cards.iter().position(|c| c.card.id == id)
    }

    pub fn turns_left(&self) -> usize {
        self.turns_left
    }

    pub fn score(&self) -> usize {
        debug!("Calculate final score");
        self.played_cards
            .values()
            .map(|n| n.score())
            .fold(0, |x, y| x + y)
    }
}

pub enum CardPlayingResult {
    Success,
    Failure,
    EpicFail,
    Err(&'static str),
}
