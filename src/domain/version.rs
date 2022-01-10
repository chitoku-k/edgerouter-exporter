use chrono::NaiveDateTime;

#[derive(Debug, PartialEq)]
pub struct Version {
    pub version: String,
    pub build_id: String,
    pub build_on: NaiveDateTime,
    pub copyright: String,
    pub hw_model: String,
    pub hw_serial_number: String,
    pub uptime: String,
}
