use anyhow::anyhow;
use chrono::NaiveDateTime;
use nom::{
    branch::permutation,
    bytes::complete::tag,
    character::complete::{newline, not_line_ending, space1},
    combinator::{map, map_res},
    error::Error,
    sequence::{delimited, tuple},
    Finish,
};

use crate::{
    domain::version::Version,
    infrastructure::cmd::parser::Parser,
    service::version::VersionResult,
};

#[derive(Clone)]
pub struct VersionParser;

impl Parser for VersionParser {
    type Item = VersionResult;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        parse_version(input)
    }
}

fn parse_version(input: &str) -> anyhow::Result<VersionResult> {
    match map(
        permutation((
            delimited(
                tuple((tag("Version:"), space1)),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                tuple((tag("Build ID:"), space1)),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                tuple((tag("Build on:"), space1)),
                map_res(not_line_ending, |s| NaiveDateTime::parse_from_str(s, "%m/%d/%y %H:%M")),
                newline,
            ),
            delimited(
                tuple((tag("Copyright:"), space1)),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                tuple((tag("HW model:"), space1)),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                tuple((tag("HW S/N:"), space1)),
                map(not_line_ending, &str::to_string),
                newline,
            ),
            delimited(
                tuple((tag("Uptime:"), space1)),
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
    )(input).finish() {
        Ok((_, version)) => Ok(version),
        Err::<_, Error<_>>(e) => Err(anyhow!(e.to_string())),
    }
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

        assert!(parser.parse(input).is_err());
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

        let actual = parser.parse(input).unwrap();
        assert_eq!(actual, Version {
            version: "v2.0.6".to_string(),
            build_id: "5208541".to_string(),
            build_on: NaiveDate::from_ymd(2006, 1, 2).and_hms(15, 4, 0),
            copyright: "2012-2018 Ubiquiti Networks, Inc.".to_string(),
            hw_model: "EdgeRouter X 5-Port".to_string(),
            hw_serial_number: "000000000000".to_string(),
            uptime: "01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00".to_string(),
        });
    }
}
