use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use responses::ResponseType;
use responses::ResponseType::*;
use requests::RequestType;
use cards::{Color, Number};
use game_state::Void;

impl Encodable for ResponseType {
    fn encode<E: Encoder>(&self, enc: &mut E) -> Result<Void, E::Error> {
        let (name, index) = match *self {
            ErrorResponseType       => ("ERROR_RESPONSE",        0),
            ConnectionResponseType  => ("CONNECTION_RESPONSE",   1),
            DiscardCardResponseType => ("DISCARD_CARD_RESPONSE", 2),
            PlayCardResponseType    => ("PLAY_CARD_RESPONSE",    3),
            HintColorResposeType    => ("HINT_COLOR_RESPONSE",   4),
            HintNumberResposeType   => ("HINT_NUMBER_RESPONSE",  5),
            GameOverResponseType    => ("GAME_OVER_RESPONSE",    6),
        };
        enc.emit_enum("ResponseType", |enc| {
            enc.emit_enum_variant(name, index, 0, |_| {
                Ok(())
            })
        })
    }
}

impl Encodable for Color {
    fn encode<E: Encoder>(&self, enc: &mut E) -> Result<Void, E::Error> {
        let (name, index) = match *self {
            Color::Red    => ("RED",    0),
            Color::Yellow => ("YELLOW", 1),
            Color::Green  => ("GREEN",  2),
            Color::Blue   => ("BLUE",   3),
            Color::White  => ("WHITE",  4),
        };
        enc.emit_enum("Color", |enc| {
            enc.emit_enum_variant(name, index, 0, |_| {
                Ok(())
            })
        })
    }
}

impl Encodable for Number {
    fn encode<E: Encoder>(&self, enc: &mut E) -> Result<Void, E::Error> {
        let (name, index) = match *self {
            Number::One   => ("ONE",   0),
            Number::Two   => ("TWO",   1),
            Number::Three => ("THREE", 2),
            Number::Four  => ("FOUR",  3),
            Number::Five  => ("FIVE",  4),
        };
        enc.emit_enum("Number", |enc| {
            enc.emit_enum_variant(name, index, 0, |_| {
                Ok(())
            })
        })
    }
}

impl Decodable for Number {
    fn decode<D: Decoder>(d: &mut D) -> Result<Number, D::Error> {
        d.read_enum("Number", |d|  {
            let names = &["ONE",
                          "TWO",
                          "THREE",
                          "FOUR",
                          "FIVE"];
            d.read_enum_variant(names, |_, i| {
                match i {
                    0 => Ok(Number::One),
                    1 => Ok(Number::Two),
                    2 => Ok(Number::Three),
                    3 => Ok(Number::Four),
                    4 => Ok(Number::Five),
                    _ => unreachable!(),
                }
            })
        })
    }
}

impl Decodable for Color {
    fn decode<D: Decoder>(d: &mut D) -> Result<Color, D::Error> {
        d.read_enum("Color", |d|  {
            let names = &["RED",
                          "YELLOW",
                          "GREEN",
                          "BLUE",
                          "WHITE"];
            d.read_enum_variant(names, |_, i| {
                match i {
                    0 => Ok(Color::Red),
                    1 => Ok(Color::Yellow),
                    2 => Ok(Color::Green),
                    3 => Ok(Color::Blue),
                    4 => Ok(Color::White),
                    _ => unreachable!(),
                }
            })
        })
    }
}

impl Decodable for RequestType {
    fn decode<D: Decoder>(d: &mut D) -> Result<RequestType, D::Error> {
        d.read_enum("RequestType", |d|  {
            let names = &["CONNECTION_REQUEST",
                          "DISCARD_REQUEST",
                          "PLAY_CARD_REQUEST",
                          "HINT_COLOR_REQUEST",
                          "HINT_NUMBER_REQUEST"];
            d.read_enum_variant(names, |_, i| {
                match i {
                    0 => Ok(RequestType::ConnectionRequestType),
                    1 => Ok(RequestType::DiscardCardRequestType),
                    2 => Ok(RequestType::PlayCardRequestType),
                    3 => Ok(RequestType::HintColorRequestType),
                    4 => Ok(RequestType::HintNumberRequestType),
                    _ => unreachable!(),
                }
            })
        })
    }
}
