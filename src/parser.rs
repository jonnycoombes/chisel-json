use chisel_stringtable::btree_string_table::BTreeStringTable;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;
use std::rc::Rc;

use crate::lexer::Lexer;
use chisel_stringtable::common::StringTable;

/// Main JSON parser struct
pub struct Parser {
    /// [StringTable] used to intern strings during parsing
    strings: Rc<RefCell<dyn StringTable<'static, u64>>>,
}

impl Parser {}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            strings: Rc::new(RefCell::new(BTreeStringTable::new())),
        }
    }
}

mod tests {
    use crate::parser::Parser;

    #[test]
    fn should_provide_a_sensible_default() {
        let parser = Parser::default();
        assert_eq!(0, parser.strings.borrow().len());
    }
}
