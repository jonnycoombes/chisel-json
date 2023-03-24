use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;
use std::rc::Rc;

use crate::lexer::Lexer;
use chisel_stringtable::common::StringTable;

pub struct Parser<'a, Reader: Debug + Read> {
    /// [StringTable] used to intern strings during parsing
    string_table: Rc<RefCell<dyn StringTable<'a, u64>>>,
    /// The [Lexer] instance used to parse out tokens from the input
    lexer: Lexer<'a, Reader>,
}
