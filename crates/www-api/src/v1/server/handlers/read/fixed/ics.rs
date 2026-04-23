// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Static Web API for fetching a timeline as a calendar
//!

use crate::v1::error::ApiError;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use ics::components::Property;
use ics::properties::{Description, DtStart, Trigger};
use ics::{Alarm, Event, ICalendar, parameters};
use open_timeline_core::{Date, Entity, HasIdAndName, Name, OpenTimelineId, TimelineView, Year};
use open_timeline_crud::{self, CrudError, FetchById, FetchByName, IdOrName, timeline_id_or_name};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

/// Handle a request to fetch a timeline calendar
pub async fn handle_get_timeline_calendar(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let mut transaction = pool.begin().await?;
    let timeline = match timeline_id_or_name(&mut transaction, id_or_name).await? {
        Some(IdOrName::Id(id)) => Ok(TimelineView::fetch_by_id(&mut transaction, &id).await?),
        Some(IdOrName::Name(name)) => {
            Ok(TimelineView::fetch_by_name(&mut transaction, &name).await?)
        }
        None => Err(CrudError::NotInDb),
    }?;

    // Filter entities
    let entities = timeline
        .entities()
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|entity| entity.end_day().is_some() || entity.start_day().is_some())
        .collect();

    // Create calendar
    let mut cal = OpenTimelineCalendar::new(timeline.name().to_owned());
    cal.add_entities(entities);
    let body = cal.to_ics_cal();

    // Set headers
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/calendar; charset=utf-8"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"Open Timeline - {}.ics\"",
            cal.name.as_str()
        ))
        .unwrap(),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );

    // Return
    Ok((headers, body))
}

/// An OpenTimeline calendar
#[derive(Debug)]
struct OpenTimelineCalendar {
    name: Name,
    events: Vec<ics::Event<'static>>,
}

impl OpenTimelineCalendar {
    /// Create a new OpenTimelineCalendar
    fn new(name: Name) -> Self {
        Self {
            name,
            events: Vec::new(),
        }
    }

    /// Add entities (transformed into events)
    fn add_entities(&mut self, entities: Vec<Entity>) {
        self.events = entities
            .into_iter()
            .filter_map(|entity| entity_to_events(&entity).ok())
            .flatten()
            .collect();
    }

    /// Get the calendar as an ICS calendar
    fn to_ics_cal(&self) -> String {
        let mut calendar = ICalendar::new("2.0", "-//open-timeline//open-timeline//english");
        let calendar_name = format!("Open Timeline - {}", self.name.as_str());
        calendar.push(Property::new("X-WR-CALNAME", calendar_name));
        for event in &self.events {
            calendar.add_event(event.clone());
        }
        calendar.to_string()
    }
}

/// Generate events for an entity
fn entity_to_events(entity: &Entity) -> anyhow::Result<Vec<ics::Event<'static>>> {
    let events = {
        let id = entity.id();
        let name = entity.name();
        match (entity.start(), entity.end()) {
            (start, Some(end)) => {
                if start == end {
                    vec![create_event(StartEndBoth::Both, id, name, start)?]
                } else {
                    vec![
                        create_event(StartEndBoth::Start, id, name, start)?,
                        create_event(StartEndBoth::End, id, name, end)?,
                    ]
                }
            }
            (start, None) => vec![create_event(StartEndBoth::Start, id, name, start)?],
        }
    };
    Ok(events)
}

/// Represent what type of event(s) should be created for an entity
#[derive(Debug, Clone, Copy)]
enum StartEndBoth {
    Start,
    End,
    Both,
}

/// Create an event
fn create_event(
    start_end_both: StartEndBoth,
    id: Option<OpenTimelineId>,
    name: &Name,
    date: Date,
) -> anyhow::Result<Event<'static>> {
    // Datetime
    let datetime = {
        let datetime = date_to_datetime(date)?;
        format_datetime_for_ics(datetime)?
    };

    // ID
    let id = {
        let id = id.ok_or(anyhow::anyhow!("Missing ID"))?.to_string();
        let modifier = match start_end_both {
            StartEndBoth::Start => "-start",
            StartEndBoth::End => "-end",
            StartEndBoth::Both => "",
        };
        format!("{id}{modifier}")
    };

    // Summary/name
    let summary = match start_end_both {
        StartEndBoth::Start => format!("[start {}] {}", date.year(), name.as_str()),
        StartEndBoth::End => format!("[end {}] {}", date.year(), name.as_str()),
        StartEndBoth::Both => format!("[{}] {}", date.year(), name.as_str()),
    };

    // Date
    let date = {
        let mut dtstart = DtStart::new(datetime.clone());
        dtstart.append(parameters!("VALUE" => "DATE-TIME"));
        Property::from(dtstart)
    };

    // Alarm
    let alarm = {
        let mut trigger = Trigger::new(datetime.clone());
        trigger.append(parameters!("VALUE" => "DATE-TIME"));
        let description = Description::new(summary.clone());
        Alarm::display(trigger, description)
    };

    // Create event
    let mut event = { Event::new(id, datetime) };
    event.push(Property::new("SUMMARY", summary));
    event.push(date);
    event.add_alarm(alarm);
    Ok(event)
}

/// Convert a [Date] for use in a calendar
fn date_to_datetime(date: Date) -> anyhow::Result<OffsetDateTime> {
    let year = Year::current().value();
    let month = date.month().ok_or(anyhow::anyhow!("Missing month"))?;
    let day = date.day().ok_or(anyhow::anyhow!("Missing day"))?.value();
    let date = time::Date::from_calendar_date(year, month.into(), day)?;
    let time = time::Time::from_hms(12, 30, 0)?;
    Ok(PrimitiveDateTime::new(date, time).assume_offset(UtcOffset::UTC))
}

/// Format helper (ICS requires YYYYMMDDTHHMMSSZ)
fn format_datetime_for_ics(datetime: OffsetDateTime) -> anyhow::Result<String> {
    let format = "[year][month][day]T[hour][minute][second]Z";
    let parsed = time::format_description::parse(format);
    Ok(datetime.format(&parsed?)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use open_timeline_core::{Date, Name, OpenTimelineId};

    const KNOWN_UUIDV4: &str = "6474cd74-244d-449b-a3d1-3a74019ec6f5";

    fn valid_entity() -> anyhow::Result<Entity> {
        Ok(Entity::from(
            Some(OpenTimelineId::from(KNOWN_UUIDV4)?),
            Name::from("Bob")?,
            Date::from(Some(6), Some(2), 1111)?,
            Some(Date::from(Some(14), Some(9), 2222)?),
            None,
        )?)
    }

    #[test]
    fn test_ics_event_from_entity() -> anyhow::Result<()> {
        let events = entity_to_events(&valid_entity()?)?;
        for event in events {
            println!("{event}");
        }
        Ok(())
    }

    #[test]
    fn test_ics_cal_from_entities() -> anyhow::Result<()> {
        let timeline_name = Name::from("Test Timeline").unwrap();
        let mut cal = OpenTimelineCalendar::new(timeline_name);
        cal.add_entities(vec![valid_entity()?]);
        println!("{}", cal.to_ics_cal());
        Ok(())
    }
}
