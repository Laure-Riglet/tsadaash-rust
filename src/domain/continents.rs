use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Continents {
    Africa,
    America,
    Antarctica,
    Asia,
    Atlantic,
    Australia,
    Europe,
    Indian,
    Pacific,
}

const CONTINENTS: [Continents; 9] = [
    Continents::Africa,
    Continents::America,
    Continents::Antarctica,
    Continents::Asia,
    Continents::Atlantic,
    Continents::Australia,
    Continents::Europe,
    Continents::Indian,
    Continents::Pacific,
];

impl fmt::Display for Continents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Continents::Africa => "Africa",
            Continents::America => "America",
            Continents::Antarctica => "Antarctica",
            Continents::Asia => "Asia",
            Continents::Atlantic => "Atlantic",
            Continents::Australia => "Australia",
            Continents::Europe => "Europe",
            Continents::Indian => "Indian",
            Continents::Pacific => "Pacific",
        };
        write!(f, "{}", s)
    }
}

impl Continents {
    pub fn iter() -> impl Iterator<Item = Continents> {
        CONTINENTS.iter().copied()
    }
}

impl Continents {
    pub fn from_choice(choice: usize) -> Option<Self> {
        CONTINENTS.get(choice.checked_sub(1)?).copied()
    }
}