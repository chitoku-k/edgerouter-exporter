use anyhow::Context;
use chrono::NaiveDateTime;
use nom::{
    branch::permutation,
    bytes::complete::tag,
    character::complete::{newline, not_line_ending, space1},
    combinator::{map, map_res},
    error::Error,
    sequence::delimited,
    Finish, IResult, Parser as _,
};

use crate::{
    domain::version::Version,
    infrastructure::cmd::parser::Parser,
    service::version::VersionResult,
};

pub struct VersionParser;

impl Parser for VersionParser {
    type Context<'a> = ();
    type Item = VersionResult;

    fn parse(&self, input: &str, _context: ()) -> anyhow::Result<Self::Item> {
        parse_version(input)
            .finish()
            .map(|(_, version)| version)
            .map_err(|e| Error::new(e.input.to_string(), e.code))
            .context("failed to parse version")
    }
}

fn parse_version(input: &str) -> IResult<&str, VersionResult> {
    map(
        permutation((
            delimited(
                (tag("Version:"), space1),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                (tag("Build ID:"), space1),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                (tag("Build on:"), space1),
                map_res(not_line_ending, |s| NaiveDateTime::parse_from_str(s, "%m/%d/%y %H:%M")),
                newline,
            ),
            delimited(
                (tag("Copyright:"), space1),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                (tag("HW model:"), space1),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                (tag("HW S/N:"), space1),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                (tag("Uptime:"), space1),
                map(not_line_ending, &str::to_string),
                newline,
            ),
        )),
        |(
            version,
            build_id,
            build_on,
            copyright,
            hw_model,
            hw_serial_number,
            uptime,
        )| {
            Version {
                version,
                build_id,
                build_on,
                copyright,
                hw_model,
                hw_serial_number,
                uptime,
            }
        },
    ).parse_complete(input)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let parser = VersionParser;
        let input = "";

        assert!(parser.parse(input, ()).is_err());
    }

    #[test]
    fn version() {
        let parser = VersionParser;
        let input = indoc! {"
            Version:      v2.0.6
            Build ID:     5208541
            Build on:     01/02/06 15:04
            Copyright:    2012-2018 Ubiquiti Networks, Inc.
            HW model:     EdgeRouter X 5-Port
            HW S/N:       000000000000
            Uptime:       01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00
        "};

        let actual = parser.parse(input, ()).unwrap();
        assert_eq!(actual, Version {
            version: "v2.0.6".to_string(),
            build_id: "5208541".to_string(),
            build_on: NaiveDate::from_ymd_opt(2006, 1, 2).and_then(|d| d.and_hms_opt(15, 4, 0)).unwrap(),
            copyright: "2012-2018 Ubiquiti Networks, Inc.".to_string(),
            hw_model: "EdgeRouter X 5-Port".to_string(),
            hw_serial_number: "000000000000".to_string(),
            uptime: "01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00".to_string(),
        });
    }
}
