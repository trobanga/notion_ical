# notion_ical
Read a [Notion](notion.so) database, extract meetings, and create an iCalendar.

I wrote it to test [shuttle.rs](shuttle.rs) and [fermyon](https://developer.fermyon.com/) and create subscriptions for my phone's calendar. 

It works with both but for fermyon I had to rewrite part of the notion crate to compile to WASM.
