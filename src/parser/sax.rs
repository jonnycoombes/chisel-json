
use crate::lexer::Lexer;
use crate::errors::ParserResult;
use crate::JsonValue;
use std::io::BufRead;
use crate::paths::PathElementStack;

/// Main JSON parser struct
#[derive(Default)]
pub struct Parser {
    /// A stack for tracking the current path within the parsed JSON
    path: PathElementStack,
}

impl Parser {
    
    pub fn parse<B : BufRead>(&self, input: B, ) -> ParserResult<JsonValue> {
        let  lexer = Lexer::new(input);
        Ok(JsonValue::Null)
    }
    


}

