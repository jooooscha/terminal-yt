use serde::{Deserialize, Serialize};
use chrono::Weekday;

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Date {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
    Workday,
    Weekend,
    Always,
    Never,
}

impl Default for Date {
    fn default() -> Self {
        Self::Always
    }
}

impl Date {
    pub fn eq_to(&self, other: &Weekday) -> bool {
        matches!((self, other), (Date::Mon, Weekday::Mon)
            | (Date::Tue, Weekday::Tue)
            | (Date::Wed, Weekday::Wed)
            | (Date::Thu, Weekday::Thu)
            | (Date::Fri, Weekday::Fri)
            | (Date::Sat, Weekday::Sat)
            | (Date::Sun, Weekday::Sun)
            | (Date::Workday, Weekday::Mon)
            | (Date::Workday, Weekday::Tue)
            | (Date::Workday, Weekday::Wed)
            | (Date::Workday, Weekday::Thu)
            | (Date::Workday, Weekday::Fri)
            | (Date::Weekend, Weekday::Sat)
            | (Date::Weekend, Weekday::Sun)
            | (Date::Always, _)
        )
    }
}
