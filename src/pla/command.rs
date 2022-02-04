use std::str::FromStr;
use crate::pla::error::PlaParseError;

#[derive(Clone, PartialEq, Debug)]
pub enum PlaCommand {
    CHILD,
    DEPENDENCY,
    DURATION,
    ENTRY,
    RESOURCE,
    START,
    UNKNOWN,
}

impl ToString for PlaCommand {
    fn to_string(&self) -> String {
        match self {
            PlaCommand::CHILD => String::from("child"),
            PlaCommand::DEPENDENCY => String::from("dep"),
            PlaCommand::DURATION => String::from("duration"),
            PlaCommand::ENTRY => String::from("entry"),
            PlaCommand::RESOURCE => String::from("res"),
            PlaCommand::START => String::from("start"),
            _ => String::from("unknown"),
        }
    }
}

impl FromStr for PlaCommand {
    type Err = PlaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "child" => Ok(PlaCommand::CHILD),
            "dep" => Ok(PlaCommand::DEPENDENCY),
            "duration" => Ok(PlaCommand::DURATION),
            "entry" => Ok(PlaCommand::ENTRY),
            "res" => Ok(PlaCommand::RESOURCE),
            "start" => Ok(PlaCommand::START),
            _ => Ok(PlaCommand::UNKNOWN),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pla::command::PlaCommand;
    use crate::pla::command::FromStr;

    #[test]
    fn it_should_convert_a_command_to_a_string() {
        let strings = vec!["child", "dep", "duration", "entry", "res", "start", "unknown"];
        let commands = vec![
            PlaCommand::CHILD,
            PlaCommand::DEPENDENCY,
            PlaCommand::DURATION,
            PlaCommand::ENTRY,
            PlaCommand::RESOURCE,
            PlaCommand::START,
            PlaCommand::UNKNOWN
        ];

        for i in 0..commands.len() {
            assert_eq!(strings[i], commands[i].to_string());
        }

    }

    #[test]
    fn it_should_convert_a_command_from_a_string() {
        let strings = vec!["child", "dep", "duration", "entry", "res", "start", "wakka"];
        let commands = vec![
            PlaCommand::CHILD,
            PlaCommand::DEPENDENCY,
            PlaCommand::DURATION,
            PlaCommand::ENTRY,
            PlaCommand::RESOURCE,
            PlaCommand::START,
            PlaCommand::UNKNOWN
        ];

        for i in 0..strings.len() {
            assert_eq!(commands[i], PlaCommand::from_str(strings[i]).unwrap());
        }
    }
}