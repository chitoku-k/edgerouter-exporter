use std::fmt::Debug;

use number_prefix::{NumberPrefix, Prefix};

pub mod bgp;
pub mod ddns;
pub mod interface;
pub mod ipsec;
pub mod load_balance;
pub mod pppoe;
pub mod version;

const NUM_1024: f64 = 1024_f64;

fn convert_size<T>(prefix: NumberPrefix<T>) -> u64
where
    T: Debug + Into<f64>,
{
    let n = match prefix {
        NumberPrefix::Standalone(v) => v.into(),

        NumberPrefix::Prefixed(Prefix::Kilo, v) => v.into() * NUM_1024.powi(1),
        NumberPrefix::Prefixed(Prefix::Mega, v) => v.into() * NUM_1024.powi(2),
        NumberPrefix::Prefixed(Prefix::Giga, v) => v.into() * NUM_1024.powi(3),
        NumberPrefix::Prefixed(Prefix::Tera, v) => v.into() * NUM_1024.powi(4),
        NumberPrefix::Prefixed(Prefix::Peta, v) => v.into() * NUM_1024.powi(5),
        NumberPrefix::Prefixed(Prefix::Exa, v) => v.into() * NUM_1024.powi(6),

        NumberPrefix::Prefixed(Prefix::Kibi, v) => v.into() * NUM_1024.powi(1),
        NumberPrefix::Prefixed(Prefix::Mebi, v) => v.into() * NUM_1024.powi(2),
        NumberPrefix::Prefixed(Prefix::Gibi, v) => v.into() * NUM_1024.powi(3),
        NumberPrefix::Prefixed(Prefix::Tebi, v) => v.into() * NUM_1024.powi(4),
        NumberPrefix::Prefixed(Prefix::Pebi, v) => v.into() * NUM_1024.powi(5),
        NumberPrefix::Prefixed(Prefix::Exbi, v) => v.into() * NUM_1024.powi(6),

        v => panic!("{v:?} overflowed."),
    };

    n as u64
}
