#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(chrono::DateTime<chrono::FixedOffset>);

impl DateTime {
    pub fn parse_rfc2822(date: impl AsRef<str>) -> Option<Self> {
        let date = chrono::DateTime::parse_from_rfc2822(date.as_ref()).ok()?;

        Some(Self(date))
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
