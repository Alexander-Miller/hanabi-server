use rustc_serialize::{Encodable, Encoder};
use responses::ResponseType;
use responses::ResponseType::*;

impl Encodable for ResponseType {
    fn encode<E: Encoder>(&self, enc: &mut E) -> Result<(), E::Error> {
        let (name, index) = match *self {
            ErrorResponseType       => ("ERROR_RESPONSE",        0),
            ConnectionResponseType  => ("CONNECTION_RESPONSE",   1),
            DiscardCardResponseType => ("DISCARD_CARD_RESPONSE", 2),
            HintColorResposeType    => ("HINT_COLOR_RESPONSE",   3),
            HintNumberResposeType   => ("HINT_NUMBER_RESPONSE",  4),
        };
        enc.emit_enum("ResponseType", |enc| {
            enc.emit_enum_variant(name, index, 0, |_| {
                Ok(())
            })
        })
    }
}
