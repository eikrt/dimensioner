use lazy_static::lazy_static;
use std::collections::HashMap;
lazy_static! {
    pub static ref FinWords: HashMap<String, Pos> =
        HashMap::from([("Olla".to_string(), Pos::Verb)]);
    pub static ref EstWords: HashMap<String, Pos> =
        HashMap::from([("Olla".to_string(), Pos::Verb)]);
    pub static ref RusWords: HashMap<String, Pos> = HashMap::from([("".to_string(), Pos::Verb)]);
}
pub enum Pos {
    Verb,
    Noun,
    Subst,
    Adj,
    Num,
}
