use std::{collections::BTreeSet, ops::Deref, time::Instant};

use chisel_json::{decoders::Encoding, sax::Parser};

/// Very simple example of using the SAX based parser to extract all the distinct object keys
/// within a JSON document and stash them away into a BTree
fn main() {
    let start = Instant::now();
    let parser = Parser::with_encoding(Encoding::Utf8);
    let mut distinct_paths: BTreeSet<String> = BTreeSet::new();
    let _result = parser.parse_file("fixtures/json/bench/citm_catalog.json", &mut |evt| {
        match evt.path {
            Some(p) => {
                if !distinct_paths.contains(p.as_string().deref()) {
                    match evt.matched {
                        chisel_json::events::Match::ObjectKey(_) => {
                            distinct_paths.insert(p.as_string().to_string());
                        }
                        _ => (),
                    };
                }
            }
            _ => (),
        }
        Ok(())
    });
    distinct_paths
        .iter()
        .for_each(|path| println!("Found distinct path: {}", path));
    println!("Extract all unique paths in: {:?}", start.elapsed());
}
