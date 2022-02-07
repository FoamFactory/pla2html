use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;
use std::str::FromStr;
use regex::Regex;
use crate::pla::command::PlaCommand;
use crate::pla::entry::PlaEntry;
use crate::pla::sub_blocks::{PlaChildBlock, PlaDependencyBlock, PlaDurationBlock, PlaStartBlock, PlaSubBlock};

pub struct PlaParser {
    pub entries: Vec<PlaEntry>,

    // mapping of ids to the index in the vector above
    id_map: Option<HashMap<u32, usize>>,
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
        match file.read_to_string(&mut contents) {
            Ok(size) => size,
            Err(e) => panic!("Unable to parse file from contents: {}", e.to_string()),
        };

        // Read the file into a vec, skipping blank lines
        let lines = contents.split("\n").map(|s| String::from(s)).filter(|l| !l.is_empty()).collect();

        PlaParser::parse(lines)
    }

    pub fn get_entry_by_id(&self, id: u32) -> Option<PlaEntry> {
        match &self.id_map {
            Some(x) => {
                let borrowed_entry = x.get(&id);
                let retval = match borrowed_entry {
                    Some(be) => Some(self.entries[*be].clone()),

                    None => None,
                };

                retval
            }
            None => None
        }
    }

    fn create_hierarchy(lines: &Vec<PlaLine>) -> Vec<HeirarchicalPlaLine> {
        let mut id: Option<u32> = None;

        lines
            .into_iter()
            .map(|l| {
                let parent_id = match l.command {
                    PlaCommand::ENTRY => {
                        id = l.get_id();
                        None
                    }                   ,
                    _ => id,
                };

                HeirarchicalPlaLine {
                    command: l.command.clone(),
                    text: String::from(&l.text),
                    parent_id,
                }
            })
            .filter(|hl| {
                hl.command != PlaCommand::UNKNOWN
            })
            .collect()
    }

    fn parse_pla_lines(lines: Vec<String>) -> Vec<PlaLine> {
        let pla_lines: Vec<PlaLine> = lines
            .into_iter()
            .map(|l| PlaLine::parse_line(l))
            .filter(|potential_line| potential_line.is_some())
            .map(|l| l.unwrap())
            .collect();

        pla_lines
    }

    fn parse(lines: Vec<String>) -> Result<PlaParser, Error> {
        // The file format for pla is available here:
        // https://www.arpalert.org/pla.html
        let pla_lines = PlaParser::parse_pla_lines(lines);
        let heirarchy: Vec<HeirarchicalPlaLine> = PlaParser::create_hierarchy(&pla_lines);

        let entries: Vec<PlaEntry> = pla_lines
            .into_iter()
            .filter(|line| line.command == PlaCommand::ENTRY)
            .map(|line| {
                let entry = match line.command {
                    PlaCommand::ENTRY => {
                        let entry_regex = Regex::new(r"^\[(\d*)\](\s)*(.*)").unwrap();
                        let id_capture = entry_regex.captures(&line.text).unwrap().get(1).map_or(String::from(""), |m| String::from(m.as_str()));
                        PlaEntry {
                            id: id_capture.parse::<u32>().unwrap(),
                            description: entry_regex.captures(&line.text).unwrap().get(3).map_or(String::from(""), |m| String::from(m.as_str())),
                            children: None
                        }
                    },
                    _ => PlaEntry {
                        id: 0,
                        description: String::from(""),
                        children: None
                    }
                };
                entry
            }).collect();

        let sub_blocks: Vec<Box<dyn PlaSubBlock>> = heirarchy
            .into_iter()
            .filter(|x|
                x.command == PlaCommand::START
                    || x.command == PlaCommand::CHILD
                    || x.command == PlaCommand::DURATION
                    || x.command == PlaCommand::DEPENDENCY)
            .map(|hl| {
                match hl.command {
                    PlaCommand::START => Box::new(PlaStartBlock::try_from(&hl).unwrap()) as Box<dyn PlaSubBlock>,
                    PlaCommand::CHILD => Box::new(PlaChildBlock::try_from(&hl).unwrap()) as Box<dyn PlaSubBlock>,
                    PlaCommand::DURATION => Box::new(PlaDurationBlock::try_from(&hl).unwrap()) as Box<dyn PlaSubBlock>,
                    PlaCommand::DEPENDENCY => Box::new(PlaDependencyBlock::try_from(&hl).unwrap()) as Box<dyn PlaSubBlock>,
                    _ => panic!("Unable to parse sub block with command: {:?}", hl.command)
                }
            })
            .collect();

        // Now, post-process all entries so that the appropriate children are included
        let hierarchical_entries: Vec<PlaEntry> = entries
            .into_iter()
            .map(|e| {
                let mut entry_sb: Vec<Box<dyn PlaSubBlock>> = vec![];
                sub_blocks
                    .iter()
                    .filter(|sb| {
                        sb.get_parent_id() == e.id
                    })
                    .filter(|sb| {
                        sb.get_command() == PlaCommand::START
                        || sb.get_command() == PlaCommand::CHILD
                        || sb.get_command() == PlaCommand::DURATION
                        || sb.get_command() == PlaCommand::DEPENDENCY
                    })
                    .for_each (|sb| {
                        match sb.get_command() {
                            PlaCommand::START => {
                                let pla_start = PlaStartBlock::try_from(sb);
                                if pla_start.is_err() {
                                    panic!("Encountered an error while trying to create hierarchical entries: {}", pla_start.err().unwrap());
                                }
                                entry_sb.push(Box::new(pla_start.unwrap()));
                            },
                            PlaCommand::CHILD => {
                                let pla_child = PlaChildBlock::try_from(sb);
                                if pla_child.is_err() {
                                    panic!("Encountered an error while trying to create hierarchical entries: {}", pla_child.err().unwrap());
                                }
                                entry_sb.push(Box::new(pla_child.unwrap()));
                            },
                            PlaCommand::DURATION => {
                              let pla_duration = PlaDurationBlock::try_from(sb);
                                if pla_duration.is_err() {
                                    panic!("Encountered an error while trying to create hierarchical entries: {}", pla_duration.err().unwrap());
                                }
                                entry_sb.push(Box::new(pla_duration.unwrap()));
                            },
                            PlaCommand::DEPENDENCY => {
                                let pla_depend = PlaDependencyBlock::try_from(sb);
                                if pla_depend.is_err() {
                                    panic!("Encountered an error while trying to create hierarchical entries: {}", pla_depend.err().unwrap());
                                }
                                entry_sb.push(Box::new(pla_depend.unwrap()));
                            },
                            _ => {}
                        }

                    });
                (e, entry_sb)
            })
            .map(|(e, mut entry_sb)| {
                let sub_block_children: Vec<Box<dyn PlaSubBlock>> = entry_sb.drain(0..).collect();
                let mut children_retval: Option<Vec<Box<dyn PlaSubBlock>>> = None;
                if sub_block_children.len() > 0 {
                    children_retval = Some(sub_block_children);
                }
                PlaEntry {
                    description: String::from(&e.description),
                    id: e.id,
                    children: children_retval
                }
            })
            .collect();
        for next_hentry in &hierarchical_entries {
            println!("Hierarchicial entry: {:?}", next_hentry);
        }

        let map = PlaParser::build_map(&hierarchical_entries);
        Ok(PlaParser {
            entries: hierarchical_entries,
            id_map: Some(map),
        })
    }

    fn build_map(entries: &Vec<PlaEntry>) -> HashMap<u32, usize> {
        let mut map: HashMap<u32, usize> = HashMap::new();
        let entries_with_id: Vec<PlaEntry> = entries
            .into_iter()
            // .filter(|e| !e.id.is_empty())
            .map(|e| PlaEntry {
                id: e.id.clone(),
                description: e.description.clone(),
                children: None
            })
            .collect();

        for next_entry_idx in 0..entries_with_id.len() {
            let next_entry = &entries_with_id[next_entry_idx];
            map.insert(next_entry.id, next_entry_idx);
        }

        map
    }
}

/// Textual definition of a line within the Pla file.
#[derive(Debug)]
pub struct PlaLine {
    command: PlaCommand,
    text: String,
}

impl PlaLine {
    fn parse_line(line: String) -> Option<PlaLine> {
        let mut line = line;

        // Trim whitespace at the start of the line
        line = String::from(line.trim_start());
        line = String::from(line.trim_end());

        // If there isn't anything in the line, then just return
        if line.is_empty() {
            return None;
        }

        // If the line starts with '[', then we have a new entry
        let command: PlaCommand;
        let identifier_re = Regex::new(r"^\[(\d*)\](\s)*(.*)").unwrap();
        if identifier_re.is_match(&line) {
            command = PlaCommand::ENTRY;
        } else {
            let sub_block_re = Regex::new(r"^(\s)*(.+)$").unwrap();
            if sub_block_re.is_match(&line) {
                let command_text = sub_block_re.captures(&line)
                    .unwrap()
                    .get(2)
                    .map_or(String::from(""), |m | String::from(m.as_str().trim_start()));
                let command_split: Vec<&str> = command_text.split(' ').collect();
                command = PlaCommand::from_str(command_split[0]).unwrap_or(PlaCommand::UNKNOWN);
            } else {
                command = PlaCommand::UNKNOWN;
            }
        }

        let text = String::from(line);
        Some(PlaLine {
            command,
            text,
        })
    }

    pub fn get_id(&self) -> Option<u32> {
        let mut id: Option<u32> = None;
        let identifier_re = Regex::new(r"^\[(\d*)\](\s)*(.*)").unwrap();
        if identifier_re.is_match(&self.text) {
            id = identifier_re.captures(&self.text)
                .unwrap()
                .get(1)
                .map_or(None, |m| {
                    match m.as_str().parse::<u32>() {
                        Ok(x) => Some(x),
                        Err(_e) => None,
                    }
                });
        }

        id
    }
}

/// Post-processed, hierarchically ordered line within a Pla file.
#[derive(Debug)]
pub struct HeirarchicalPlaLine {
    pub command: PlaCommand,
    pub text: String,
    pub parent_id: Option<u32>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use chrono::NaiveDate;
    use super::*;

    #[test]
    #[should_panic]
    fn when_parsing_start_blocks_it_should_fail_without_a_date() {
        todo!()
        // let start_block = "start 15";
        // let pla_start = PlaStartBlock::from_str(start_block);
        // let _unwrapped = pla_start.unwrap();
    }

    #[test]
    fn it_should_trim_whitespace_at_start_of_file() {
        let text = r#"



        [10000] Autumn's Early Arrival Blonde (Batch: 10000)

        start 2021-01-15 15
        "#;
        let lines: Vec<String> = text.split("\n").map(|s| String::from(s)).collect();
        let pla_lines: Vec<PlaLine> = PlaParser::parse_pla_lines(lines);
        assert_eq!(2, pla_lines.len());
    }

    #[test]
    fn it_should_parse_a_valid_start_block() {
        let text = r#"[10000] Autumn's Early Arrival Blonde (Batch: 10000)

        start 2021-01-15 15
        "#;
        let lines: Vec<String> = text.split("\n").map(|s| String::from(s)).collect();
        let pla_lines: Vec<PlaLine> = PlaParser::parse_pla_lines(lines);
        let heir_pla_lines: Vec<HeirarchicalPlaLine> = PlaParser::create_hierarchy(&pla_lines);
        let pla_start = PlaStartBlock::try_from(&heir_pla_lines[1]);
        let unwrapped_ps = pla_start.unwrap();
        assert_eq!(NaiveDate::from_ymd(2021, 1, 15), unwrapped_ps.date);
        assert_eq!(15, unwrapped_ps.hour);
    }

    #[test]
    fn it_should_parse_a_simple_pla_file() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let mut path_buf: PathBuf = PathBuf::new();
        path_buf.push(dir);
        path_buf.push("contrib");
        path_buf.push("pla_simple.pla");

        let path: &Path = path_buf.as_path();
        let pla_parser = match PlaParser::new(path) {
            Ok(p) => p,
            Err(why) => panic!("Unable to parse {} due to {}", path.to_str().unwrap_or(""), why),
        };

        println!("{:?}", pla_parser.get_entry_by_id(10000).unwrap());
        println!("{:?}", pla_parser.get_entry_by_id(122).unwrap());
        assert!(pla_parser.get_entry_by_id(10000).unwrap().has_children());
        assert!(pla_parser.get_entry_by_id(122).unwrap().has_children());
    }

    #[test]
    fn it_should_parse_a_complicated_pla_file() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let mut path_buf: PathBuf = PathBuf::new();
        path_buf.push(dir);
        path_buf.push("contrib");
        path_buf.push("pla_complicated.pla");

        let path: &Path = path_buf.as_path();
        let pla_parser = match PlaParser::new(path) {
            Ok(p) => p,
            Err(why) => panic!("Unable to parse {} due to {}", path.to_str().unwrap_or(""), why),
        };

        assert_eq!(PlaEntry {
            id: 10000,
            description: String::from("Autumn's Early Arrival Blonde (Batch: 10000)"),
            children: None
        }, pla_parser.get_entry_by_id(10000).unwrap());

        println!("{:?}", pla_parser.get_entry_by_id(10000).unwrap());

        // We shouldn't be able to get a nonexistent id
        assert_eq!(None, pla_parser.get_entry_by_id(2018271));
    }
}