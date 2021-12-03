//! A parser for sed commands
use nom::character::complete::char;
use nom::{
    bytes::complete::{tag, take_till},
    error::{Error, ErrorKind},
    sequence::delimited,
    IResult,
};

use crate::sed_command::SedCommand;

#[allow(clippy::needless_lifetimes)]
pub fn parse_sed_command<'a>(input: &'a str) -> IResult<&'a str, SedCommand<'a>> {
    let take_until_slash = take_till(|x| x == '/');

    // Parse the initial 's' character
    let (remaining, _) = tag("s")(input)?;

    // Parse the first section of the command
    let (remaining, from) = delimited(char('/'), &take_until_slash, char('/'))(remaining)?;

    // Parse the second section of the command
    let (remaining, to) = take_until_slash(remaining)?;

    match remaining {
        "/" | "" => Ok((
            remaining,
            SedCommand {
                from,
                to,
                is_global: false,
            },
        )),
        "/g" => Ok((
            remaining,
            SedCommand {
                from,
                to,
                is_global: true,
            },
        )),
        //               huh ?
        wrong => Err(nom::Err::Error(Error::new(wrong, ErrorKind::Fail))),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_sed_command;

    fn assert_parses(input: &str, from: &str, to: &str, is_global: bool) {
        let command = parse_sed_command(input);
        assert!(command.is_ok());

        let (_, command) = command.unwrap();

        assert_eq!(command.is_global, is_global);
        assert_eq!(command.from, from);
        assert_eq!(command.to, to);
    }

    #[test]
    fn parses_valid_commands() {
        // There are probably a lot of edge cases not covered here
        assert_parses("s/from/to", "from", "to", false);
        assert_parses("s/from/to/g", "from", "to", true);
        assert_parses("s//to/g", "", "to", true);
        assert_parses("s//to", "", "to", false);
        assert_parses("s//", "", "", false);

        // TODO: this one should pass but doesn't
        // assert_parses("s//g", "", "", true);
    }

    #[test]
    fn does_not_parse_invalid_commands() {
        parse_sed_command("").unwrap_err();
        parse_sed_command("g/from/to").unwrap_err();
        parse_sed_command("g/from/to/s").unwrap_err();
        parse_sed_command("s/").unwrap_err();
        parse_sed_command("s/////s").unwrap_err();
        parse_sed_command("s/from/to//").unwrap_err();
    }
}
