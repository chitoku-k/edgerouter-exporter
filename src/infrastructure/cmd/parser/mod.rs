use std::time::Duration;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u64,
    combinator::map,
    sequence::{terminated, tuple},
    IResult,
};

pub mod bgp;
pub mod ddns;
pub mod interface;
pub mod load_balance;
pub mod pppoe;
pub mod version;

pub trait Parser {
    type Input;
    type Item;

    fn parse(&self, input: Self::Input) -> anyhow::Result<Self::Item>;
}

pub fn parse_duration(input: &str) -> IResult<&str, Duration> {
    alt((
        map(
            tuple((
                terminated(u64, tag(":")),
                terminated(u64, tag(":")),
                u64,
            )),
            |(h, m, s)| Duration::new(h * 60 * 60 + m * 60 + s, 0),
        ),
        map(
            tuple((
                terminated(u64, tag("h")),
                terminated(u64, tag("m")),
                terminated(u64, tag("s")),
            )),
            |(h, m, s)| Duration::new(h * 60 * 60 + m * 60 + s, 0),
        ),
        map(
            tuple((
                terminated(u64, tag("w")),
                terminated(u64, tag("d")),
                terminated(u64, tag("h")),
            )),
            |(w, d, h)| Duration::new(w * 7 * 24 * 60 * 60 + d * 24 * 60 * 60 + h * 60 * 60, 0),
        ),
        map(
            tuple((
                terminated(u64, tag("d")),
                terminated(u64, tag("h")),
                terminated(u64, tag("m")),
            )),
            |(d, h, m)| Duration::new(d * 24 * 60 * 60 + h * 60 * 60 + m * 60, 0),
        ),
    ))(input)
}
