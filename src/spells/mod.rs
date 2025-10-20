use crate::fs;

mod widget;

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
    Unknown,
}

impl Glyph {
    fn parse(name: &str) -> Self {
        match name {
            "action-glyph" => Self::OneAction,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct SpellDescTableRow {
    cells: Vec<Vec<SpellDescEl>>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct SpellDescTable {
    head: Option<SpellDescTableRow>,
    body: Vec<SpellDescTableRow>,
    foot: Option<SpellDescTableRow>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum SpellDescEl {
    Text(String),
    LineBreak,
    Bold(String),
    Italic(String),
    Glyph(Glyph),
    List(Vec<Vec<SpellDescEl>>),
    Table(SpellDescTable),
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

fn parse_xml_description(desc: impl std::io::Read) -> SpellDescription {
    use xml::reader::XmlEvent::*;
    type El = SpellDescEl;

    let mut attr_stack = Vec::new();
    let mut els = Vec::new();
    let mut text = String::new();
    let mut in_list = false;
    let mut ul = Vec::new();
    let mut li = Vec::new();
    let mut in_table = false;
    let mut in_head = false;
    let mut in_foot = false;
    let mut table = SpellDescTable::default();
    let mut tr = SpellDescTableRow::default();
    let mut td = Vec::new();

    fn push_nonempty(dst: &mut Vec<El>, el: El) -> bool {
        let empty = match &el {
            El::Text(str) | El::Bold(str) | El::Italic(str) => str.is_empty(),
            El::LineBreak => false,
            El::Glyph(_) => false,
            El::List(items) => items.is_empty(),
            El::Table(table) => {
                table.head.is_none()
                    && table.body.is_empty()
                    && table.foot.is_none()
            }
        };
        if !empty {
            dst.push(el);
            true
        } else {
            false
        }
    }

    for event in xml::EventReader::new(desc) {
        if event.is_err() {
            continue;
        }

        let dst = if in_table {
            &mut td
        } else if in_list {
            &mut li
        } else {
            &mut els
        };

        match event.unwrap() {
            StartElement {
                name, attributes, ..
            } => {
                push_nonempty(dst, El::Text(std::mem::take(&mut text)));
                attr_stack.push(attributes);

                match name.local_name.as_str() {
                    "ul" | "ol" => in_list = true,
                    "table" => in_table = true,
                    "thead" => in_head = true,
                    "tfoot" => in_foot = true,
                    _ => {}
                }
            }
            EndElement { name } => {
                let attributes = attr_stack.pop();
                let text = std::mem::take(&mut text);
                match name.local_name.as_str() {
                    "p" => {
                        if push_nonempty(dst, El::Text(text)) {
                            dst.push(El::LineBreak);
                        }
                    }
                    "strong" | "b" => {
                        push_nonempty(dst, El::Bold(text));
                    }
                    "em" => {
                        push_nonempty(dst, El::Italic(text));
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" => {
                        dst.push(El::LineBreak);
                        if push_nonempty(dst, El::Bold(text)) {
                            dst.push(El::LineBreak);
                        }
                    }
                    "li" => ul.push(std::mem::take(&mut li)),
                    "ul" | "ol" => {
                        els.push(El::List(std::mem::take(&mut ul)));
                        in_list = false;
                    }
                    "span" => {
                        let name = attributes
                            .and_then(|a| {
                                a.iter()
                                    .find(|a| a.name.local_name == "class")
                                    .cloned()
                            })
                            .map(|a| a.value)
                            .unwrap_or(String::new());
                        dst.push(El::Glyph(Glyph::parse(&name)));
                    }
                    "hr" | "br" => dst.push(El::LineBreak),
                    "table" => {
                        els.push(El::Table(std::mem::take(&mut table)));
                        in_table = false;
                    }
                    "thead" | "tfoot" | "tbody" => {
                        in_head = false;
                        in_foot = false;
                    }
                    "tr" => {
                        if in_head {
                            table.head = Some(std::mem::take(&mut tr));
                        } else if in_foot {
                            table.foot = Some(std::mem::take(&mut tr));
                        } else {
                            table.body.push(std::mem::take(&mut tr));
                        }
                    }
                    "td" | "th" => {
                        // N.B. treating <th> as equivalent to <td> for now.
                        tr.cells.push(std::mem::take(&mut td));
                    }
                    other => {
                        dbg!(other);
                    }
                }
            }
            Characters(cs) => text.push_str(&cs),
            Whitespace(cs) => text.push_str(&cs),
            EndDocument => {
                if !text.is_empty() {
                    els.push(El::Text(std::mem::take(&mut text)));
                }
            }
            _ => {}
        }
    }
    SpellDescription(els)
}

fn entry_to_spell(entry: SpellsDataEntry) -> Spell {
    if entry.name == "Avatar" {
        dbg!(parse_xml_description(entry.description.as_bytes()));
    }

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

        description: parse_xml_description(entry.description.as_bytes()),

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
    load_spell_data(reader).unwrap();
    panic!();
}
