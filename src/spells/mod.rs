use std::sync::{Arc, RwLock};

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
            "common" => Self::Common,
            _ => Self::Common,
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

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct SpellDescTableRow {
    cells: Vec<Vec<SpellDescEl>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct SpellDescTable {
    head: Option<SpellDescTableRow>,
    body: Vec<SpellDescTableRow>,
    foot: Option<SpellDescTableRow>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum SpellDescEl {
    Text(String),
    LineBreak,
    Bold(String),
    Italic(String),
    Glyph(Glyph),
    List(Vec<Vec<SpellDescEl>>),
    Table(SpellDescTable),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SpellDescription(Vec<SpellDescEl>);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Spell {
    pub name: String,
    rarity: Rarity,
    pub rank: i8,

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

pub struct SpellBookQuery(Vec<Arc<Spell>>);

impl SpellBookQuery {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<Spell>> {
        self.0.iter()
    }
}

struct SpellBookInner {
    spells: Vec<Arc<Spell>>,
    status: String,
}

#[derive(Clone)]
pub struct SpellBook(Arc<RwLock<SpellBookInner>>);

impl SpellBook {
    pub fn query_all(&self) -> SpellBookQuery {
        match self.0.read() {
            Ok(inner) => SpellBookQuery(inner.spells.clone()),
            Err(_) => SpellBookQuery(Vec::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.0.read().map(|sb| sb.spells.len()).unwrap_or(0)
    }

    pub fn load_spells(&self) {
        populate_spellbook_in_background(self.clone());
    }

    pub fn status(&self) -> String {
        if let Ok(lock) = self.0.try_read() {
            lock.status.clone()
        } else {
            "Poisoned!".to_string()
        }
    }

    fn set_status(&self, status: impl ToString) {
        if let Ok(mut lock) = self.0.try_write() {
            lock.status = status.to_string();
        }
    }
}

impl Default for SpellBook {
    fn default() -> Self {
        let inner = SpellBookInner {
            spells: Vec::new(),
            status: "Loading...".to_string(),
        };
        Self(Arc::new(RwLock::new(inner)))
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
                    other => todo!("{}", other),
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

fn parse_spells_data_spells(
    json: impl std::io::Read,
) -> Result<Vec<Spell>, String> {
    let entries: Vec<SpellsDataEntry> =
        serde_json::de::from_reader(json).map_err(|e| e.to_string())?;
    let spells = entries.into_iter().map(entry_to_spell).collect();
    Ok(spells)
}

fn download_spell_data() -> Result<Vec<Spell>, String> {
    const URL: &str = "https://raw.githubusercontent.com/OwenFeik/spells_data/refs/heads/master/pf2e/spells.json";

    let response = reqwest::blocking::get(URL)
        .map_err(|e| format!("Failed to download spells.json: {e}"))?;
    parse_spells_data_spells(response)
}

fn merge_into_spellbook(
    spellbook: SpellBook,
    mut spells: Vec<Arc<Spell>>,
) -> Result<(), String> {
    let names: std::collections::HashSet<&str> =
        spells.iter().map(|s| s.name.as_str()).collect();
    let mut old_spells_not_in_new_spells = Vec::new();
    for spell in &spellbook.0.read().map_err(|e| e.to_string())?.spells {
        if !names.contains(spell.name.as_str()) {
            old_spells_not_in_new_spells.push(spell.clone());
        }
    }
    for spell in old_spells_not_in_new_spells {
        spells.push(spell);
    }

    spells.sort_by(|a, b| a.name.cmp(&b.name));
    spellbook.0.write().map_err(|e| e.to_string())?.spells = spells;
    Ok(())
}

const CACHE_FILE: &str = "spellbook.json";
fn load_spells_from_cache() -> Result<Vec<Spell>, String> {
    let reader = fs::read_data(CACHE_FILE).map_err(|e| e.to_string())?;
    serde_json::de::from_reader(reader).map_err(|e| e.to_string())
}

fn save_spells_to_cache(spellbook: SpellBook) {
    if let Ok(sb) = spellbook.0.read() {
        spellbook.set_status("Recording findings...");
        if let Ok(data) = serde_json::ser::to_vec(&sb.spells) {
            fs::write_data(CACHE_FILE, data).ok();
        }
    }
}

/// Load spells into spellbook in background thread, either by loading them
/// from the cache file or by downloading them from github, parsing and then
/// saving to cache file for use in future.
fn populate_spellbook_in_background(spellbook: SpellBook) {
    std::thread::spawn(|| {
        spellbook.set_status("Reading tome...");
        let result = match load_spells_from_cache() {
            Err(_) => {
                spellbook.set_status("Communing...");
                download_spell_data()
            }
            Ok(spells) => Ok(spells),
        };

        match result {
            Ok(spells) => {
                let spells = spells.into_iter().map(Arc::new).collect();
                merge_into_spellbook(spellbook.clone(), spells).ok();
                let status = if let Ok(inner) = spellbook.0.read() {
                    format!("{} spells.", inner.spells.len())
                } else {
                    "Retrieved.".to_string()
                };
                spellbook.set_status(status);
                save_spells_to_cache(spellbook);
            }
            Err(e) => {
                spellbook.set_status(format!("Error: {e}"));
            }
        }
    });
}

#[test]
fn test() {
    download_spell_data().unwrap();
    panic!();
}
