/// notion-ical
///
/// Create an iCalendar for a given user from Notion DB.
use anyhow::Result;
use notion::{
    ids::{AsIdentifier, DatabaseId, UserId},
    models::{
        paging::Paging,
        search::{DatabaseQuery, FilterCondition, PropertyCondition},
        users::User,
    },
    NotionApi,
};
use std::str::FromStr;

pub mod calendar;
mod event;

pub use event::Event;

#[derive(Clone)]
pub struct NotionIcal {
    db_id: DatabaseId,
    ical_prod_id: String,
    notion_api: NotionApi,
}

impl NotionIcal {
    pub fn new(api_token: String, db_id: &str, ical_prod_id: String) -> Result<Self> {
        let notion_api = NotionApi::new(api_token, None)?;
        let db_id = DatabaseId::from_str(db_id)?;
        Ok(Self {
            db_id,
            ical_prod_id,
            notion_api,
        })
    }

    pub async fn future_events_for_user(&self, user_id: &str) -> Result<Vec<Event>> {
        let mut events = vec![];

        let mut paging: Option<Paging> = None;
        let user_id = UserId::from_str(user_id)?;
        loop {
            let pages = self
                .notion_api
                .query_database(
                    self.db_id.as_id(),
                    DatabaseQuery {
                        sorts: None,
                        filter: Some(FilterCondition::And {
                            and: vec![
                                FilterCondition::Property {
                                    property: "Attendees".to_string(),
                                    condition: PropertyCondition::People(
                                        notion::models::search::PeopleCondition::Contains(
                                            user_id.clone(),
                                        ),
                                    ),
                                },
                                FilterCondition::Property {
                                    property: "Event time".to_string(),
                                    condition: PropertyCondition::Date(
                                        notion::models::search::DateCondition::NextYear,
                                    ),
                                },
                            ],
                        }),
                        paging: paging.clone(),
                    },
                )
                .await?;

            for event in pages.results.into_iter() {
                events.push(Event::try_from(event)?);
            }

            if pages.has_more {
                paging = Some(Paging {
                    start_cursor: pages.next_cursor,
                    page_size: None,
                });
                continue;
            } else {
                break;
            }
        }

        Ok(events)
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let users = self.notion_api.list_users().await?;
        tracing::debug!(?users);
        Ok(users.results)
    }

    pub async fn calendar_for_user<S: AsRef<str>>(&self, user: S) -> Result<String> {
        let events = self.future_events_for_user(user.as_ref()).await?;
        calendar::generate_calendar(events, &self.ical_prod_id)
    }
}
#[cfg(test)]
mod tests {

    use std::env;

    use crate::{calendar, NotionIcal};
    use anyhow::Context;
    use notion::{ids::Identifier, NotionApi};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        dotenv::dotenv()?;

        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "trace")
        }
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let notion_ical = NotionIcal::new(
            std::env::var("NOTION_API_TOKEN").context(
                "No Notion API token found in either the environment variable \
                        `NOTION_API_TOKEN` or the config file!",
            )?,
            "db_id",
            "prod_id".to_string(),
        )?;

        let users = notion_ical.list_users().await?;
        tracing::debug!(?users);
        let user = &users[0];
        let user_id = user.id().value();
        let events = notion_ical.future_events_for_user(user_id).await?;

        tracing::info!(?events);

        let cal = calendar::generate_calendar(events, "prod id")?;
        tracing::info!(cal);
        Ok(())
    }

    use wiremock::MockServer;

    #[tokio::test]
    #[ignore]
    async fn mock() -> anyhow::Result<()> {
        dotenv::dotenv()?;

        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "trace")
        }
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();

        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();
        let _notion_api = NotionApi::new(
            std::env::var("NOTION_API_TOKEN").context(
                "No Notion API token found in either the environment variable \
                        `NOTION_API_TOKEN` or the config file!",
            )?,
            Some(uri.into()),
        )?;

        tracing::info!("ask for events");

        // let events =
        //     future_events_for_user(notion_api, &std::env::var("NOTION_USER_ID").unwrap()).await;
        // tracing::info!(?events);

        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
        Ok(())
    }
}
