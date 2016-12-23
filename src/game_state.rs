use std::collections::HashSet;
use std::collections::HashMap;
use std::mem;
use std::default::Default;
use cards::{Color, Number, Card, Deck};
use requests::{HintNumberRequest, HintColorRequest, PlayCardRequest, DiscardCardRequest};
use responses::error_messages::*;

pub type Void = ();

const DEFAULT_HINT_TOKENS: u8 = 7;
const DEFAULT_ERR_TOKENS:  u8 = 3;

#[derive(RustcEncodable)]
pub struct CardKnowledge {
    pub knows_color:      bool,
    pub knows_number:     bool,
    pub knows_color_not:  HashSet<Color>,
    pub knows_number_not: HashSet<Number>,
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
    pub card:      Card,
    pub knowledge: CardKnowledge,
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
    hint_tokens:     u8,
    hint_tokens_max: u8,
    err_tokens:      u8,
    played_cards:    HashMap<Color, Number>,
    players:         Vec<Player>,
    deck:            Deck,
}

impl Default for GameState {
    fn default() -> Self {
        debug!("Creating default game state.");
        GameState::new(DEFAULT_HINT_TOKENS, DEFAULT_ERR_TOKENS)
    }
}

impl GameState {
    pub fn new(hint_tokens_max: u8, err_tokens: u8) -> Self {
        GameState {
            hint_tokens:     hint_tokens_max,
            hint_tokens_max: hint_tokens_max,
            err_tokens:      err_tokens,
            played_cards:    HashMap::new(),
            players:         Vec::with_capacity(6),
            deck:            Deck::new(),
        }
    }

    pub fn add_player(&mut self, name: &str) -> Result<Void, &'static str> {
        debug!("Adding new player \"{}\".", name);
        if self.deck.cards.len() < 5 {
            debug!("Not enough cards for new player.");
            return Err(NO_CARDS);
        }

        if self.players.iter().any(|p| p.name == name) {
            debug!("Player already exists.");
            return Err(PLAYER_ALREADY_EXISTS);
        }

        let cards = self.deck.cards
            .drain(0..5)
            .map(|c| CardInHand::new(c))
            .collect();
        self.players.push(Player::new(name.into(), cards));
        debug!("Added new player: {}", self.players[self.players.len()-1]);
        Ok(())
    }

    pub fn discard_card(&mut self, name: &str, discard_req: &DiscardCardRequest) -> Result<Void, &'static str> {
        debug!("Discarding card {} of player {}.", discard_req.discarded_card, name);

        let mut player = self.players.iter_mut().find(|p| p.name == name).unwrap();
        match player.cards.iter().position(|hc| hc.card == discard_req.discarded_card) {
            None => {
                error!("Card {} could not be found.", discard_req.discarded_card);
                Err(CARD_NOT_FOUND)
            }
            Some(i) => {
                if self.hint_tokens < self.hint_tokens_max {
                    self.hint_tokens += 1;
                }
                match self.deck.pop() {
                    Some(card) => {
                        debug!("Card discarded and new card drawn.");
                        debug!("Player state before discard: {}", player);
                        let mut new_card_in_hand = CardInHand::new(card);
                        mem::swap(&mut new_card_in_hand, &mut player.cards[i]);
                        debug!("Player state after discard: {}", player);
                    }
                    None => {
                        debug!("Deck is empty and removed card will not be replaced.");
                        debug!("Player state before discard: {}", player);
                        player.cards.remove(i);
                        debug!("Player state after discard: {}", player);
                    }
                }
                Ok(())
            }
        }
    }

    pub fn hint_color(&mut self, name: &str, req: &HintColorRequest) -> Result<Void, &'static str> {
        debug!("Hinting color {} for player {}.", req.color, name);
        self.knowledge_update(req.target_player.as_str(),
                              &|c| { c.card.color == req.color },
                              &|c| { c.knowledge.knows_color = true },
                              &|c| { c.knowledge.knows_color_not.insert(req.color.clone()); })
    }

    pub fn hint_number(&mut self, name: &str, req: &HintNumberRequest) -> Result<Void, &'static str> {
        debug!("Hinting number {} for player {}.", req.number, name);
        self.knowledge_update(req.target_player.as_str(),
                              &|c| { c.card.number == req.number },
                              &|c| { c.knowledge.knows_number = true },
                              &|c| { c.knowledge.knows_number_not.insert(req.number.clone()); })
    }

    fn knowledge_update(&mut self,
                        name: &str,
                        predicate: &Fn(&CardInHand) -> bool,
                        update_positive: &Fn(&mut CardInHand),
                        update_negative: &Fn(&mut CardInHand))
                        -> Result<Void, &'static str>
    {
        debug!("Update knowledge for player {}.", name);
        try!(self.use_hint());
        let mut player = self.player_by_name(&name);
        for mut card_in_hand in &mut player.cards {
            match predicate(&card_in_hand) {
                true  => update_positive(&mut card_in_hand),
                false => update_negative(&mut card_in_hand),
            }
        }
        Ok(())
    }

    pub fn play_card(&mut self, name: &str, req: &PlayCardRequest) -> CardPlayingResult {
        debug!("Player {} trying to play card {}.", name, req.played_card);
        let mut player = self.players.iter_mut().find(|p| p.name == name).unwrap();
        if let Some(i) = player.cards.iter().position(|cih| cih.card == req.played_card) {
            debug!("Player state before playing card: {}", player);
            match self.deck.pop() {
                Some(card) => {
                    let mut new_card_in_hand = CardInHand::new(card);
                    mem::swap(&mut new_card_in_hand, &mut player.cards[i]);
                }
                None => {
                    player.cards.remove(i);
                }
            }
            debug!("Player state after playing card: {}", player);
            match Number::is_next_largest(self.played_cards.get(&req.played_card.color), &req.played_card.number) {
                true =>  {
                    self.played_cards.insert(req.played_card.color.clone(), req.played_card.number.clone());
                    debug!("Play card success. Currently played cards: {:?}",
                           self.played_cards.iter().map(|(color, number)| format!("{}: {}", color, number)).collect::<Vec<_>>());
                    return CardPlayingResult::Success;
                }
                false => {
                    self.err_tokens -= 1;
                    debug!("Play card fail. {} err tokens left.", self.err_tokens);
                    match self.err_tokens {
                        0 => return CardPlayingResult::EpicFail,
                        _ => return CardPlayingResult::Failure,
                    }
                }
            }
        } else {
            error!("Card to be played not found.");
            return CardPlayingResult::Err(CARD_NOT_FOUND);
        }
    }

    pub fn score(&self) -> usize {
        debug!("Calculate final score");
        self.played_cards
            .values()
            .map(|n| n.score())
            .fold(0, |x, y| x + y)
    }

    pub fn deck_is_empty(&self) -> bool {
        debug!("Check if deck is empty");
        self.deck.cards.is_empty()
    }

    fn player_by_name(&mut self, name: &str) -> &mut Player {
        debug!("Get player {} by name.", name);
        self.players.iter_mut().find(|p| p.name == name).unwrap()
    }

    fn use_hint(&mut self) -> Result<Void, &'static str> {
        if self.hint_tokens < 1 {
            error!("Cannot use hint without hint tokens.");
            Err(NO_HINT_TOKENS)
        } else {
            self.hint_tokens -= 1;
            debug!("Hint used, {} tokens left.", self.hint_tokens);
            Ok(())
        }
    }
}

pub enum CardPlayingResult {
    Success,
    Failure,
    EpicFail,
    Err(&'static str),
}
