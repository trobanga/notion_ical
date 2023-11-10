use ical::generator::{Emitter, IcalCalendarBuilder};

use crate::Event;

pub fn generate_calendar(events: Vec<Event>) -> anyhow::Result<String> {
    let mut cal = IcalCalendarBuilder::version("2.0")
        .gregorian()
        .prodid(std::env::var("ICAL_PROD_ID")?)
        .build();

    for event in events {
        cal.events
            .push(event.to_ical(std::env::var("ICAL_TIMEZONE")?));
    }

    Ok(cal.generate())
}
