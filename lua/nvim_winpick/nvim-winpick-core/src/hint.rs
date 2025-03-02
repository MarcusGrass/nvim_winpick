#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "test", derive(Eq, PartialEq))]
pub enum Hint {
    #[default]
    FloatingBigLetter,
    FloatingLetter,
}

impl Hint {
    pub(crate) fn from_str(test: &str) -> anyhow::Result<Self> {
        let matched = match test {
            "floating-big-letter" => Self::FloatingBigLetter,
            "floating-letter" => Self::FloatingLetter,
            unk => anyhow::bail!("unknown hint {unk}"),
        };
        Ok(matched)
    }
}
