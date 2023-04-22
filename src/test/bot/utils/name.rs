
pub struct NameProvider;

impl NameProvider {
    pub fn woman_first_name() -> &'static str {
        FIRST_NAMES_WOMAN.first().unwrap()
    }

    pub fn men_first_name() -> &'static str {
        FIRST_NAMES_MEN.first().unwrap()
    }

    pub fn random_woman_first_name() -> &'static str {
        let name_i = rand::random::<usize>() % FIRST_NAMES_WOMAN.len();
        FIRST_NAMES_WOMAN[name_i]
    }

    pub fn random_men_first_name() -> &'static str {
        let name_i = rand::random::<usize>() % FIRST_NAMES_MEN.len();
        FIRST_NAMES_MEN[name_i]
    }
}

// https://www.mtvuutiset.fi/artikkeli/tassa-ovat-suomen-kaikkien-aikojen-suosituimmat-etunimet-loydatko-omasi/3198590

pub const FIRST_NAMES_WOMAN: &[&str] = &[
    "Maria",
    "Helana",
    "Johanna",
    "Anneli",
    "Kaarina",
    "Marjatta",
    "Anna",
    "Liisa",
    "Annikki",
    "Hannele",
];

pub const FIRST_NAMES_MEN: &[&str] = &[
    "Juhani",
    "Johannes",
    "Olavi",
    "Antero",
    "Tapani",
    "Kalevi",
    "Tapio",
    "Matti",
    "Mikael",
    "Ilmari",
];
