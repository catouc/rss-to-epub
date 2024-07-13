use crate::config::Config;
use crate::feed_reader::{fetch_feed, FeedRequest};
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub mod config;
pub mod feed_reader;
pub mod transformer;

fn main() -> Result<()> {
    let config_path = PathBuf::from("./config.toml");
    let config = Config::try_from(config_path)?;

    let conn = Connection::open("feed-to-rss.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY,
            feed_url TEXT NOT NULL,
            last_modified TEXT NOT NULL
        )",
        (),
    )?;

    config.feeds.iter()
        .filter_map(|feed| url::Url::parse(&feed.1.url).ok())
        .filter_map(|feed_url| FeedRequest::from_conn_and_url(&conn, feed_url).ok())
        .filter_map(|feed_request| fetch_feed(&conn, feed_request).ok())
        .for_each(|feed| {
            println!("{}", feed.title.unwrap().content);
            feed.entries.iter().for_each(|entry| {
                transformer::entry_to_epub(entry).expect("epub failed to create")
            });
        });
    Ok(())
}
