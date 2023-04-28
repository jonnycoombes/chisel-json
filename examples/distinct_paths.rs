use chisel_json::{decoders::Encoding, events::Match, sax::Parser};

/// Macro to tidy up the match arm
macro_rules! selected_event {
    () => {
        Match::StartObject
            | Match::StartArray
            | Match::String(_)
            | Match::Integer(_)
            | Match::Float(_)
            | Match::Boolean(_)
            | Match::Null
    };
}

/// Very simple example of using the SAX based parser to extract all the distinct keys in a JSON
/// document
fn main() {
    let parser = Parser::with_encoding(Encoding::Utf8);
    let _result = parser.parse_file("fixtures/json/bench/citm_catalog.json", &mut |evt| {
        match evt.matched {
            selected_event!() => println!("{}", evt.path.unwrap()),
            _ => (),
        }
        Ok(())
    });
}
