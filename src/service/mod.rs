use anyhow::Result;

pub trait Runner {
    type Item;

    fn run(&self) -> Result<Self::Item>;
}
