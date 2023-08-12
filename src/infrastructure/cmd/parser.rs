use std::{time, str::FromStr};

use derive_more::Into;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u64,
    combinator::map,
    error::Error,
    sequence::{terminated, tuple},
    Finish, IResult,
};

pub mod bgp;
pub mod ddns;
pub mod interface;
pub mod load_balance;
pub mod pppoe;
pub mod version;

pub trait Parser {
    type Context<'a>;
    type Item;

    fn parse(&self, input: &str, context: Self::Context<'_>) -> anyhow::Result<Self::Item>;
}

#[derive(Into)]
pub struct Duration(time::Duration);

impl FromStr for Duration {
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, duration) = parse_duration(s).map_err(|e| e.to_owned()).finish()?;
        Ok(Duration(duration))
    }
}

fn parse_duration(input: &str) -> IResult<&str, time::Duration> {
    alt((
        map(
            tuple((
                terminated(u64, tag(":")),
                terminated(u64, tag(":")),
                u64,
            )),
            |(h, m, s)| time::Duration::new(h * 60 * 60 + m * 60 + s, 0),
        ),
        map(
            tuple((
                terminated(u64, tag("h")),
                terminated(u64, tag("m")),
                terminated(u64, tag("s")),
            )),
            |(h, m, s)| time::Duration::new(h * 60 * 60 + m * 60 + s, 0),
        ),
        map(
            tuple((
                terminated(u64, tag("w")),
                terminated(u64, tag("d")),
                terminated(u64, tag("h")),
            )),
            |(w, d, h)| time::Duration::new(w * 7 * 24 * 60 * 60 + d * 24 * 60 * 60 + h * 60 * 60, 0),
        ),
        map(
            tuple((
                terminated(u64, tag("d")),
                terminated(u64, tag("h")),
                terminated(u64, tag("m")),
            )),
            |(d, h, m)| time::Duration::new(d * 24 * 60 * 60 + h * 60 * 60 + m * 60, 0),
        ),
    ))(input)
}
