#[derive(Serialize, Deserialize, Debug, Clone)]
struct DateEntry {
    #[serde(default)] date: String,
    #[serde(default)] dateTime: String,
    #[serde(default)] timeZone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    summary: String,
    description: String,
    htmlLink: String,
    start: DateEntry,
}
