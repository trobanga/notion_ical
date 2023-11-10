use chrono::{DateTime, Utc};
use ical::{
    generator::{IcalEvent, IcalEventBuilder, Property},
    ical_property,
};
use notion::models::{
    properties::{DateOrDateTime, PropertyValue},
    Page,
};

#[derive(Debug)]
pub struct Event {
    id: String,
    title: String,
    changed: DateTime<Utc>,
    start: DateOrDateTime,
    end: Option<DateOrDateTime>,
}

impl TryFrom<Page> for Event {
    type Error = anyhow::Error;

    fn try_from(page: Page) -> std::result::Result<Self, Self::Error> {
        let PropertyValue::Date { id: _, date } = &page.properties.properties["Event time"] else {
            return Err(anyhow::anyhow!("No event time"));
        };
        let Some(date) = date else {
            return Err(anyhow::anyhow!("No event time"));
        };

        Ok(Self {
            id: page.id.to_string(),
            title: page.title().unwrap_or("No Title".to_string()),
            changed: page.last_edited_time,
            start: date.start.clone(),
            end: date.end.clone(),
        })
    }
}

impl Event {
    pub fn to_ical<S: Into<String>>(&self, timezone: S) -> IcalEvent {
        let event = IcalEventBuilder::tzid(timezone)
            .uid(self.id.clone())
            .changed_utc(self.changed.to_string());

        let start = format!("{}", self.start);
        let end = match &self.end {
            Some(date) => format!("{date}"),
            None => String::new(),
        };
        let event = match self.start {
            DateOrDateTime::Date(_) => {
                if self.end.is_some() {
                    event.start_day(start).end_day(end)
                } else {
                    event.one_day(start)
                }
            }
            DateOrDateTime::DateTime(_) => event.start(start).end(end),
        };

        event
            .set(ical_property!("SUMMARY", &self.title.clone()))
            .build()
    }
}
