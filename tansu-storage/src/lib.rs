// Copyright ⓒ 2024 Peter Morgan <peter.james.morgan@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use async_trait::async_trait;
use glob::{GlobError, PatternError};
use regex::Regex;
use std::{
    array::TryFromSliceError,
    collections::BTreeMap,
    ffi::OsString,
    fmt::Debug,
    fs::DirEntry,
    io,
    num::{ParseIntError, TryFromIntError},
    path::PathBuf,
    result,
    str::FromStr,
    sync::PoisonError,
    time::{Duration, SystemTime, SystemTimeError},
};
use tansu_kafka_sans_io::record::deflated;
use uuid::Uuid;

pub mod index;
pub mod pg;
pub mod segment;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("build")]
    DeadPoolBuild(#[from] deadpool::managed::BuildError),

    #[error("glob")]
    Glob(#[from] GlobError),

    #[error("io")]
    Io(#[from] io::Error),

    #[error("kafka sans io")]
    KafkaSansIo(#[from] tansu_kafka_sans_io::Error),

    #[error("offset: {offset}, is less than base offset: {base_offset}")]
    LessThanBaseOffset { offset: i64, base_offset: i64 },

    #[error("offset: {offset}, is less than last offset: {last_offset:?}")]
    LessThanLastOffset {
        offset: i64,
        last_offset: Option<i64>,
    },

    #[error("time: {time}, is less than max time: {max_time:?}")]
    LessThanMaxTime { time: i64, max_time: Option<i64> },

    #[error("time: {time}, is less than min time: {min_time:?}")]
    LessThanMinTime { time: i64, min_time: Option<i64> },

    #[error("message: {0}")]
    Message(String),

    #[error("no such entry nth: {nth}")]
    NoSuchEntry { nth: u32 },

    #[error("no such offset: {0}")]
    NoSuchOffset(i64),

    #[error("os string {0:?}")]
    OsString(OsString),

    #[error("pattern")]
    Pattern(#[from] PatternError),

    #[error("parse int: {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("poision")]
    Poison,

    #[error("pool")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("postgres")]
    TokioPostgres(#[from] tokio_postgres::error::Error),

    #[error("regex")]
    Regex(#[from] regex::Error),

    #[error("segment empty: {0:?}")]
    SegmentEmpty(Topition),

    #[error("segment missing: {topition:?}, at offset: {offset:?}")]
    SegmentMissing {
        topition: Topition,
        offset: Option<i64>,
    },

    #[error("system time: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("try from int: {0}")]
    TryFromInt(#[from] TryFromIntError),

    #[error("try from slice: {0}")]
    TryFromSlice(#[from] TryFromSliceError),

    #[error("url: {0}")]
    Url(#[from] url::ParseError),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_value: PoisonError<T>) -> Self {
        Self::Poison
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Topition {
    topic: String,
    partition: i32,
}

impl Topition {
    pub fn new(topic: &str, partition: i32) -> Self {
        let topic = topic.to_owned();
        Self { topic, partition }
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn partition(&self) -> i32 {
        self.partition
    }
}

impl TryFrom<&DirEntry> for Topition {
    type Error = Error;

    fn try_from(value: &DirEntry) -> result::Result<Self, Self::Error> {
        Regex::new(r"^(?<topic>.+)-(?<partition>\d{10})$")
            .map_err(Into::into)
            .and_then(|re| {
                value
                    .file_name()
                    .into_string()
                    .map_err(Error::OsString)
                    .and_then(|ref file_name| {
                        re.captures(file_name)
                            .ok_or(Error::Message(format!("no captures for {file_name}")))
                            .and_then(|ref captures| {
                                let topic = captures
                                    .name("topic")
                                    .ok_or(Error::Message(format!("missing topic for {file_name}")))
                                    .map(|s| s.as_str().to_owned())?;

                                let partition = captures
                                    .name("partition")
                                    .ok_or(Error::Message(format!(
                                        "missing partition for: {file_name}"
                                    )))
                                    .map(|s| s.as_str())
                                    .and_then(|s| str::parse(s).map_err(Into::into))?;

                                Ok(Self { topic, partition })
                            })
                    })
            })
    }
}

impl FromStr for Topition {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        i32::from_str(&s[s.len() - 10..])
            .map(|partition| {
                let topic = String::from(&s[..s.len() - 11]);

                Self { topic, partition }
            })
            .map_err(Into::into)
    }
}

impl From<&Topition> for PathBuf {
    fn from(value: &Topition) -> Self {
        let topic = value.topic.as_str();
        let partition = value.partition;
        PathBuf::from(format!("{topic}-{partition:0>10}"))
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TopitionOffset {
    topition: Topition,
    offset: i64,
}

impl TopitionOffset {
    pub fn new(topition: Topition, offset: i64) -> Self {
        Self { topition, offset }
    }

    pub fn topition(&self) -> &Topition {
        &self.topition
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }
}

impl From<&TopitionOffset> for PathBuf {
    fn from(value: &TopitionOffset) -> Self {
        let offset = value.offset;
        PathBuf::from(value.topition()).join(format!("{offset:0>20}"))
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ListOffsetRequest {
    #[default]
    Earliest,
    Latest,
    Timestamp(i64),
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ListOffsetResponse {
    timestamp: Option<i64>,
    offset: Option<i64>,
}

impl From<ListOffsetRequest> for i64 {
    fn from(value: ListOffsetRequest) -> Self {
        match value {
            ListOffsetRequest::Earliest => -2,
            ListOffsetRequest::Latest => -1,
            ListOffsetRequest::Timestamp(timestamp) => timestamp,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OffsetCommitRequest {
    offset: i64,
    leader_epoch: Option<i32>,
    timestamp: Option<SystemTime>,
    metadata: Option<String>,
}

#[async_trait]
pub trait StorageProvider {
    async fn provide_storage(&mut self) -> impl Storage;
}

#[async_trait]
pub trait Storage: Debug + Send + Sync {
    async fn create_topic(
        &self,
        name: &str,
        partitions: i32,
        config: &[(&str, Option<&str>)],
    ) -> Result<Uuid>;

    async fn delete_topic(&self, name: &str) -> Result<u64>;

    async fn produce(&self, topition: &'_ Topition, batch: deflated::Batch) -> Result<i64>;
    async fn fetch(&self, topition: &'_ Topition, offset: i64) -> Result<deflated::Batch>;
    async fn last_stable_offset(&self, topition: &'_ Topition) -> Result<i64>;
    async fn high_watermark(&self, topition: &'_ Topition) -> Result<i64>;

    async fn list_offsets(
        &self,
        offsets: &[(Topition, ListOffsetRequest)],
    ) -> Result<&[(Topition, ListOffsetResponse)]>;

    async fn offset_commit(
        &self,
        group_id: &str,
        retention_time_ms: Option<Duration>,
        offsets: &[(Topition, OffsetCommitRequest)],
    ) -> Result<()>;

    async fn offset_fetch(
        &self,
        group_id: Option<&str>,
        topics: &[Topition],
        require_stable: Option<bool>,
    ) -> Result<BTreeMap<Topition, i64>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topition_from_str() -> Result<()> {
        let topition = Topition::from_str("qwerty-2147483647")?;
        assert_eq!("qwerty", topition.topic());
        assert_eq!(i32::MAX, topition.partition());
        Ok(())
    }

    #[test]
    fn topic_with_dashes_in_name() -> Result<()> {
        let topition = Topition::from_str("test-topic-0000000-eFC79C8-2147483647")?;
        assert_eq!("test-topic-0000000-eFC79C8", topition.topic());
        assert_eq!(i32::MAX, topition.partition());
        Ok(())
    }
}
