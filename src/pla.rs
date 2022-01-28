use std::collections::HashMap;
use std::fmt::Error;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Map;
use std::path::{Path, PathBuf};

use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct PlaEntry {
    pub id: String,
    pub text: String,
}

pub struct PlaParser {
    pub entries: Vec<PlaEntry>,
    id_map: Option<HashMap<u64, PlaEntry>>,
}

impl PlaParser {
    pub fn new(file_path: &Path) -> Result<PlaParser, Error> {
        let display = file_path.display();

        // Open file for reading
        let mut file = match File::open(file_path) {
            Err(why) => panic!("Couldn't open {}: {}", display, why),
            Ok(f) => f
        };

        // Read all lines and parse the file
        let mut contents: String = String::from("");
        file.read_to_string(&mut contents);

        // Read the file into a vec, skipping blank lines
        let lines = contents.split("\n").map(|s| String::from(s)).filter(|l| !l.is_empty()).collect();

        PlaParser::parse(lines)
    }

    pub fn get_entry_by_id(&self, id: u64) -> Option<PlaEntry> {
        println!("ID IS: {:?}", id);

        match &self.id_map {
            Some(x) => {
                let borrowed_entry = x.get(&id);
                let retval = match borrowed_entry {
                    Some(be) => Some(PlaEntry {
                        id: String::from(&be.id),
                        text: String::from(&be.text)
                    }),
                    None => None,
                };

                retval
            }
            None => None
        }
    }

    fn parse(lines: Vec<String>) -> Result<PlaParser, Error> {
        // The file format for pla is available here:
        // https://www.arpalert.org/pla.html
        let pla_lines: Vec<PlaLine> = lines.into_iter().map(|l| PlaLine::parse_line(l)).collect();
        let entries: Vec<PlaEntry> = pla_lines.into_iter().map(|line| {
            let entry = match line.command {
                PlaCommand::ENTRY => PlaEntry {
                    id: line.id.unwrap_or(String::from("")),
                    text: line.text.unwrap_or(String::from(""))
                },
                _ => PlaEntry {
                    id: String::from(""),
                    text: String::from(""),
                }
            };

            entry
        }).collect();
        // for next_line in pla_lines {
        //     if next_line.command == PlaCommand::ENTRY {
        //         println!("Entry id: {}", next_line.id.unwrap());
        //         println!("Entry text: {}", next_line.text.unwrap());
        //     }
        // }
        let map = PlaParser::build_map(&entries);
        Ok(PlaParser {
            entries,
            id_map: Some(map),
        })
    }

    fn build_map(entries: &Vec<PlaEntry>) -> HashMap<u64, PlaEntry> {
        let mut map = HashMap::new();
        let entries_with_id: Vec<PlaEntry> = entries.into_iter().filter(|e| !e.id.is_empty()).map(|e| PlaEntry {
            id: e.id.clone(),
            text: e.text.clone(),
        }).collect();

        for next_entry in entries_with_id {
            let id = String::from(&next_entry.id);
            let text = next_entry.text;
            map.insert((&next_entry.id).parse::<u64>().unwrap_or(1), PlaEntry {
                id,
                text
            });
        }
        map
    }
}

#[derive(PartialEq)]
enum PlaCommand {
    DEPENDENCY,
    DURATION,
    ENTRY,
    RESOURCE,
    START,
}

struct PlaLine {
    command: PlaCommand,
    id: Option<String>,
    text: Option<String>,
}

impl PlaLine {
    fn parse_line(line: String) -> PlaLine {
        // If the line starts with '[', then we have a new entry
        let mut id;
        let mut command: PlaCommand;
        let mut text: Option<String>;
        let identifier_re = Regex::new(r"^\[(\d*)\](\s)*(.*)").unwrap();
        if identifier_re.is_match(&line) {
            let caps = identifier_re.captures(&line).unwrap();
            command = PlaCommand::ENTRY;
            id = Some(caps.get(1).map_or(String::from(""), |m| String::from(m.as_str())));
            text = Some(caps.get(3).map_or(String::from(""), |m | String::from(m.as_str())));
        } else {
            command = PlaCommand::RESOURCE;
            id = None;
            text = None;
        }

        PlaLine {
            command,
            id,
            text,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;

    #[test]
    fn it_should_parse_a_complicated_pla_file() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let mut path_buf: PathBuf = PathBuf::new();
        path_buf.push("contrib");
        path_buf.push("pla.pla");

        let path: &Path = path_buf.as_path();
        let pla_parser = match PlaParser::new(path) {
            Ok(p) => p,
            Err(why) => panic!("Unable to parse {} due to {}", path.to_str().unwrap_or(""), why),
        };

        assert_eq!(PlaEntry {
            id: String::from("10000"),
            text: String::from("Autumn's Early Arrival Blonde (Batch: 10000)"),
        }, pla_parser.get_entry_by_id(10000).unwrap());
    }
}