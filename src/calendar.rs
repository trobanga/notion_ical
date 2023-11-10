use ical::generator::{Emitter, IcalCalendarBuilder};

use crate::Event;

pub fn generate_calendar(events: Vec<Event>, prod_id: &str) -> anyhow::Result<String> {
    let mut cal = IcalCalendarBuilder::version("2.0")
        .gregorian()
        .prodid(prod_id)
        .build();

    for event in events {
        cal.events.push(event.to_ical());
    }

    Ok(cal.generate())
}
