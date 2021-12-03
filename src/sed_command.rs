use std::borrow::Cow;

use regex::RegexBuilder;

#[derive(Debug, Clone)]
pub struct SedCommand<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub is_global: bool,
}

impl<'a> SedCommand<'a> {
    pub fn execute(&self, input: &'a str) -> anyhow::Result<Cow<'a, str>> {
        let re = RegexBuilder::new(self.from).build()?;

        let replaced = if self.is_global {
            re.replace_all(input, self.to)
        } else {
            re.replace(input, self.to)
        };

        Ok(replaced)
    }
}
