use crate::{fs, roll};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Unique,
}

impl Rarity {
    fn parse(str: &str) -> Self {
        match str.to_lowercase().as_str() {
            "unique" => Self::Unique,
            "rare" => Self::Rare,
            "uncommon" => Self::Uncommon,
            "common" | _ => Self::Common,
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum Glyph {
    OneAction,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum SpellDescEl {
    Text(String),
    LineBreak,
    Bold(String),
    Italic(String),
    Glyph(Glyph),
    List(Vec<Vec<SpellDescEl>>),
    Table(Option<Vec<String>>, Vec<Vec<String>>),
    Divider,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SpellDescription(Vec<SpellDescEl>);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Spell {
    name: String,
    rarity: Rarity,
    rank: i8,

    traditions: Vec<String>,
    traits: Vec<String>,

    target: String,
    range: String,
    time: String,
    duration: String,
    sustained: bool,

    description: SpellDescription,

    publication: String,
}

pub type SharedSpellBook = std::sync::Arc<std::sync::RwLock<SpellBook>>;
pub struct SpellBook(Vec<Spell>);

impl SpellBook {
    const FILE: &str = "spellbook.json";

    pub fn load() -> Self {
        if let Ok(reader) = fs::read_data(Self::FILE) {
            if let Ok(spells) = serde_json::de::from_reader(reader) {
                return Self(spells);
            }
        }

        Self(Vec::new())
    }

    fn add(&mut self, spell: Spell) {
        self.0.retain(|s| s.name != spell.name);
        self.0.push(spell);
        self.0.sort_by(|a, b| {
            a.rank.cmp(&b.rank).then_with(|| a.name.cmp(&b.name))
        });
    }

    fn save(&self) -> Result<(), String> {
        let data =
            serde_json::ser::to_vec(&self.0).map_err(|e| e.to_string())?;
        fs::write_data(Self::FILE, data).map_err(|e| e.to_string())
    }
}

#[derive(serde::Deserialize)]
struct SpellsDataEntry {
    name: String,
    rank: i8,
    rarity: String,
    target: String,
    range: String,
    time: String,
    duration: String,
    sustained: bool,
    description: String,
    traditions: Vec<String>,
    traits: Vec<String>,
    publication: String,
}

fn parse_xml_description(desc: String) -> SpellDescription {
    use xml::reader::XmlEvent::*;

    let mut els = Vec::new();
    for event in xml::EventReader::new(desc.as_bytes()) {
        if event.is_err() {
            continue;
        }

        match event.unwrap() {
            StartElement { name, .. }
        }
    }
    SpellDescription(els)
}

fn entry_to_spell(entry: SpellsDataEntry) -> Spell {
    Spell {
        name: entry.name,
        rarity: Rarity::parse(&entry.rarity),
        rank: entry.rank,

        traditions: entry.traditions,
        traits: entry.traits,

        target: entry.target,
        range: entry.range,
        time: entry.time,
        duration: entry.duration,
        sustained: entry.sustained,

        description: parse_xml_description(entry.description),

        publication: entry.publication,
    }
}

fn load_spell_data(json: impl std::io::Read) -> Result<Vec<Spell>, String> {
    let entries: Vec<SpellsDataEntry> =
        serde_json::de::from_reader(json).map_err(|e| e.to_string())?;
    let spells = entries.into_iter().map(entry_to_spell).collect();
    Ok(spells)
}

#[test]
fn test() {
    const PATH: &str = "/home/owen/src/owen/spells_data/pf2e/spells.json";

    let reader = std::fs::File::open(PATH).unwrap();
    dbg!(load_spell_data(reader).unwrap());
    panic!();
}
