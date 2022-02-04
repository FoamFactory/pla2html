use chrono::NaiveDate;
use crate::pla::command::PlaCommand;
use crate::pla::error::PlaParseError;
use crate::pla::parser::{HeirarchicalPlaLine, PlaLine};

pub trait PlaSubBlock {
    fn get_command(&self) -> PlaCommand;
    fn get_parent_id(&self) -> u32;
}

#[derive(Debug)]
pub struct PlaStartBlock {
    parent_id: u32,
    pub date: NaiveDate,
    pub hour: u32,
}

impl PlaSubBlock for PlaStartBlock {
    fn get_command(&self) -> PlaCommand {
        PlaCommand::START
    }

    fn get_parent_id(&self) -> u32 {
        self.parent_id
    }
}

// impl TryFrom<PlaLine> for PlaStartBlock {
//     type Error = ();
//
//     fn try_from(value: PlaLine) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

impl TryFrom<&HeirarchicalPlaLine> for PlaStartBlock {
    type Error = PlaParseError;

    fn try_from(value: &HeirarchicalPlaLine) -> Result<Self, Self::Error> {
        let str_command = value.text.trim_start();
        match value.parent_id {
            Some(x) => {
                match PlaStartBlock::try_from((x, str_command)) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(PlaParseError{ message: String::from("start block parsing failed")}),
                }
            },
            None => Err(PlaParseError{ message: String::from("Unable to parse HeirarchicalPlaLine without parent id as PlaStartBlock") }),
        }
    }
}

impl TryFrom<(u32, &str)> for PlaStartBlock {
    type Error = PlaParseError;

    fn try_from(value: (u32, &str)) -> Result<Self, Self::Error> {
        let (parent_id, command_text) = value;
        let tokens: Vec<String> = command_text.split(' ').map(|s| String::from(s)).collect();

        let str_date: String = match tokens.get(1) {
            Some(x) => String::from(x),
            None => return Err(PlaParseError { message: String::from("unable to parse date for start command") }),
        };

        let date: NaiveDate = match NaiveDate::parse_from_str(&str_date, "%Y-%m-%d") {
            Ok(x) => x,
            Err(_e) => return Err(PlaParseError { message: String::from("unable to parse date from string") })
        };

        // Default to midnight if an hour isn't provided.
        let str_hour = match tokens.get(2) {
            Some(x) => x,
            None => "0"
        };

        let hour = match str_hour.parse::<u32>() {
            Ok(x) => x,
            Err(_e) => return Err(PlaParseError { message: String::from("unable to parse hour of day in start sub block") })
        };

        return Ok(PlaStartBlock {
            date,
            hour,
            parent_id
        });
    }
}