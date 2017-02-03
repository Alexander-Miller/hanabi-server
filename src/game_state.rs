use cards;
use cards::{Card, Color, Number, CardInHand};
use responses::error_messages::*;

use std::collections::HashMap;
use std::mem;
use std::default::Default;

pub type Void = ();

const CARDS_IN_DECK:        usize = 50;
const DEFAULT_HINT_TOKENS:  usize = 8;
const DEFAULT_ERR_TOKENS:   usize = 3;
const FOUR_CARDS_THRESHOLD: usize = 4;

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
    deck:            Vec<Card>,
    discarded_cards: Vec<Card>,
    next_player:     String,
    turns_left:      Option<usize>,
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
            deck:            cards::new_deck(),
            discarded_cards: Vec::with_capacity(CARDS_IN_DECK),
            next_player:     String::new(),
            turns_left:      None,
        }
    }

    pub fn add_player(&mut self, name: &str) -> Result<Void, &'static str> {
        info!("Adding new player {}.", name);
        if self.deck.len() < 5 {
            error!("Not enough cards for new player.");
            return Err(NO_CARDS);
        }

        if self.players.iter().any(|p| p.name == name) {
            error!("Player already exists.");
            return Err(PLAYER_ALREADY_EXISTS);
        }

        let no_of_players = self.players.len() + 1;
        let cards_per_player = match no_of_players >= FOUR_CARDS_THRESHOLD {
            false => {
                debug!("Number of players below threshold. New player will receive 5 cards.");
                5
            }
            true  => {
                if no_of_players == FOUR_CARDS_THRESHOLD {
                    debug!("Threshold fpr number of players crossed. Removing cards from all previously added players.");
                    for player in &mut self.players {
                        self.deck.push(player.cards.pop().unwrap().card);
                    }
                }
                debug!("Number of players above threshold. New player will receive 4 cards.");
                4
            }
        };

        let cards = self.deck
            .drain(0..cards_per_player)
            .map(|c| CardInHand::new(c))
            .collect();

        self.players.push(Player::new(name.into(), cards));
        self.next_player = name.into();

        debug!("Added new player: {}", self.players[self.players.len()-1]);
        debug!("Number of players increased to {}", self.players.len());

        Ok(())
    }

    pub fn discard_card(&mut self, name: &str, discarded_card_id: usize) -> DiscardCardResult {
        info!("Discarding card #{} of player {}.", discarded_card_id, name);

        if let Some(p_index) = self.player_index(name) {
            if let Some(c_index) = self.card_index(p_index, discarded_card_id) {
                return self.do_discard_card(p_index, c_index)
            } else {
                error!("Could not find card #{} on the hand of player {}", discarded_card_id, name);
                return DiscardCardResult::Err(CARD_NOT_FOUND)
            }
        } else {
            error!("Could not find player {}", name);
            return DiscardCardResult::Err(PLAYER_NOT_FOUND)
        }
    }

    fn do_discard_card(&mut self, p_index: usize, c_index: usize) -> DiscardCardResult {
        match self.maybe_turn_has_passed() {
            Err(msg) => return DiscardCardResult::Err(msg),
            _ => {}
        };
        let (discarded_card, drawn_card) = self.maybe_draw_new_card(p_index, c_index);
        self.discarded_cards.push(discarded_card);

        if self.hint_tokens < self.hint_tokens_max {
            debug!("Card discarded - number of hint tokens increased to {}", self.hint_tokens);
            self.hint_tokens += 1;
        }

        DiscardCardResult::Ok {
            discarded_card: discarded_card,
            drawn_card:     drawn_card
        }
    }

    pub fn play_card(&mut self, name: &str, played_card_id: usize) -> CardPlayingResult {
        debug!("Playing card #{} of player {}", played_card_id, name);

        if let Some(p_index) = self.player_index(name) {
            if let Some(c_index) = self.card_index(p_index, played_card_id) {
                return self.do_play_card(p_index, c_index)
            } else {
                error!("Could not find card #{} on the hand of player {}", played_card_id, name);
                return CardPlayingResult::Err(CARD_NOT_FOUND)
            }
        } else {
            error!("Could not find player {}", name);
            return CardPlayingResult::Err(PLAYER_NOT_FOUND)
        }
    }

    fn do_play_card(&mut self, p_index: usize, c_index: usize) -> CardPlayingResult {
        match self.maybe_turn_has_passed() {
            Err(msg) => return CardPlayingResult::Err(msg),
            _ => {}
        };
        let (played_card, drawn_card) = self.maybe_draw_new_card(p_index, c_index);

        if Number::is_next_largest(self.played_cards.get(&played_card.color), &played_card.number) {
            debug!("Play card success. Currently played cards:\n {:?}",
                   self.played_cards.iter().map(|(color, number)| format!("{}: {}\n", color, number)).collect::<Vec<_>>());

            if played_card.number == Number::Five && self.hint_tokens < self.hint_tokens_max {
                self.hint_tokens += 1;
                debug!("Played a Five - number of hint tokens increased to {}.", self.hint_tokens);
            }

            self.played_cards.insert(played_card.color, played_card.number);

            CardPlayingResult::Ok {
                success:     true,
                played_card: played_card,
                drawn_card:  drawn_card,
            }
        } else {
            match self.err_tokens {
                0 => CardPlayingResult::EpicFail,
                _ => {
                    self.discarded_cards.push(played_card);
                    self.err_tokens -= 1;
                    debug!("Play card fail. {} err tokens left.", self.err_tokens);
                    CardPlayingResult::Ok {
                        success:     false,
                        played_card: played_card,
                        drawn_card:  drawn_card,
                    }
                }
            }
        }
    }

    fn maybe_draw_new_card(&mut self, p_index: usize, c_index: usize) -> (Card, Option<Card>) {
        self.set_next_player();
        let mut hand = &mut self.players[p_index].cards;
        match self.deck.pop() {
            Some(new_card) => {
                debug!("Card removed and {} drawn as replacement", new_card);
                let mut new_card_in_hand = CardInHand::new(new_card);
                mem::swap(&mut new_card_in_hand, &mut hand[c_index]);
                (new_card_in_hand.card, Some(hand[c_index].card))
            }
            None => {
                debug!("Deck is empty and removed card will not be replaced.");
                (hand.remove(c_index).card, None)
            }
        }
    }

    pub fn hint_color(&mut self, name: &str, color: &Color) -> Result<Void, &'static str> {
        info!("Hinting color {} for player {}.", color, name);
        self.knowledge_update(name,
                              &|c| { c.card.color == *color },
                              &|c| { c.knowledge.knows_color = true; c.knowledge.knows_color_not.clear(); },
                              &|c| { c.knowledge.knows_color_not.insert(color.clone()); })
    }

    pub fn hint_number(&mut self, name: &str, number: &Number) -> Result<Void, &'static str> {
        info!("Hinting number {} for player {}.", number, name);
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
        try!(self.maybe_turn_has_passed());
        if let Some(p_index) = self.player_index(&name) {
            try!(self.use_hint());
            self.set_next_player();

            for mut card_in_hand in &mut self.players[p_index].cards {
                match predicate(&card_in_hand) {
                    true  => update_positive(&mut card_in_hand),
                    false => update_negative(&mut card_in_hand),
                }
            }
            return Ok(())
        } else {
            error!("Could not find player {}", name);
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

    fn maybe_turn_has_passed(&mut self) -> Result<Void, &'static str> {
        if self.deck.is_empty() {
            self.turns_left = match self.turns_left {
                None => {
                    debug!("Deck is empty, leaving every player with 1 more turn to go.");
                    Some(self.players.len()-1)
                }
                Some(t) if t > 0 => {
                    debug!("Deck is empty and {} more turns are left.", t-1);
                    Some(t-1)
                }
                Some(_) => {
                    error!("Tried to execute an action when the game was already over with 0 turns left.");
                    return Err(GAME_IS_OVER)
                }
            }
        };
        Ok(())
    }

    fn player_index(&self, name: &str) -> Option<usize> {
        self.players.iter().position(|p| p.name == name)
    }

    fn card_index(&self, p_index: usize, id: usize) -> Option<usize> {
        self.players[p_index].cards.iter().position(|c| c.card.id == id)
    }

    pub fn turns_left(&self) -> Option<usize> {
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
    Ok {
        success:     bool,
        played_card: Card,
        drawn_card:  Option<Card>,
    },
    EpicFail,
    Err(&'static str),
}

pub enum DiscardCardResult {
    Ok {
        discarded_card: Card,
        drawn_card:     Option<Card>
    },
    Err(&'static str),
}
