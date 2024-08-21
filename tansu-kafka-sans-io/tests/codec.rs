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

use std::{fs::File, sync::Arc, thread};
use tansu_kafka_sans_io::{Error, Frame, Result};
use tracing::subscriber::DefaultGuard;

#[cfg(miri)]
fn init_tracing() -> Result<()> {
    Ok(())
}

#[cfg(not(miri))]
fn init_tracing() -> Result<DefaultGuard> {
    Ok(tracing::subscriber::set_default(
        tracing_subscriber::fmt()
            .with_level(true)
            .with_line_number(true)
            .with_thread_names(false)
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(
                thread::current()
                    .name()
                    .ok_or(Error::Message(String::from("unnamed thread")))
                    .and_then(|name| {
                        File::create(format!("tests/codec-{}.log", name)).map_err(Into::into)
                    })
                    .map(Arc::new)?,
            )
            .finish(),
    ))
}

#[test]
fn api_versions_request_v3_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 52, 0, 18, 0, 3, 0, 0, 0, 3, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 112,
        114, 111, 100, 117, 99, 101, 114, 0, 18, 97, 112, 97, 99, 104, 101, 45, 107, 97, 102, 107,
        97, 45, 106, 97, 118, 97, 6, 51, 46, 54, 46, 49, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn api_versions_response_v1_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 242, 0, 0, 0, 0, 0, 0, 0, 0, 0, 38, 0, 0, 0, 0, 0, 5, 0, 1, 0, 0, 0, 6, 0, 2, 0,
        0, 0, 2, 0, 3, 0, 0, 0, 5, 0, 4, 0, 0, 0, 1, 0, 5, 0, 0, 0, 0, 0, 6, 0, 0, 0, 4, 0, 7, 0,
        0, 0, 1, 0, 8, 0, 0, 0, 3, 0, 9, 0, 0, 0, 3, 0, 10, 0, 0, 0, 1, 0, 11, 0, 0, 0, 2, 0, 12,
        0, 0, 0, 1, 0, 13, 0, 0, 0, 1, 0, 14, 0, 0, 0, 1, 0, 15, 0, 0, 0, 1, 0, 16, 0, 0, 0, 1, 0,
        17, 0, 0, 0, 1, 0, 18, 0, 0, 0, 1, 0, 19, 0, 0, 0, 2, 0, 20, 0, 0, 0, 1, 0, 21, 0, 0, 0, 0,
        0, 22, 0, 0, 0, 0, 0, 23, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 26, 0, 0, 0,
        0, 0, 27, 0, 0, 0, 0, 0, 28, 0, 0, 0, 0, 0, 29, 0, 0, 0, 0, 0, 30, 0, 0, 0, 0, 0, 31, 0, 0,
        0, 0, 0, 32, 0, 0, 0, 0, 0, 33, 0, 0, 0, 0, 0, 34, 0, 0, 0, 0, 0, 35, 0, 0, 0, 0, 0, 36, 0,
        0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let api_key = 18;
    let api_version = 1;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn api_versions_response_v3_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 1, 201, 0, 0, 0, 0, 0, 0, 56, 0, 0, 0, 0, 0, 9, 0, 0, 1, 0, 0, 0, 15, 0, 0, 2, 0, 0,
        0, 8, 0, 0, 3, 0, 0, 0, 12, 0, 0, 8, 0, 0, 0, 8, 0, 0, 9, 0, 0, 0, 8, 0, 0, 10, 0, 0, 0, 4,
        0, 0, 11, 0, 0, 0, 9, 0, 0, 12, 0, 0, 0, 4, 0, 0, 13, 0, 0, 0, 5, 0, 0, 14, 0, 0, 0, 5, 0,
        0, 15, 0, 0, 0, 5, 0, 0, 16, 0, 0, 0, 4, 0, 0, 17, 0, 0, 0, 1, 0, 0, 18, 0, 0, 0, 3, 0, 0,
        19, 0, 0, 0, 7, 0, 0, 20, 0, 0, 0, 6, 0, 0, 21, 0, 0, 0, 2, 0, 0, 22, 0, 0, 0, 4, 0, 0, 23,
        0, 0, 0, 4, 0, 0, 24, 0, 0, 0, 4, 0, 0, 25, 0, 0, 0, 3, 0, 0, 26, 0, 0, 0, 3, 0, 0, 27, 0,
        0, 0, 1, 0, 0, 28, 0, 0, 0, 3, 0, 0, 29, 0, 0, 0, 3, 0, 0, 30, 0, 0, 0, 3, 0, 0, 31, 0, 0,
        0, 3, 0, 0, 32, 0, 0, 0, 4, 0, 0, 33, 0, 0, 0, 2, 0, 0, 34, 0, 0, 0, 2, 0, 0, 35, 0, 0, 0,
        4, 0, 0, 36, 0, 0, 0, 2, 0, 0, 37, 0, 0, 0, 3, 0, 0, 38, 0, 0, 0, 3, 0, 0, 39, 0, 0, 0, 2,
        0, 0, 40, 0, 0, 0, 2, 0, 0, 41, 0, 0, 0, 3, 0, 0, 42, 0, 0, 0, 2, 0, 0, 43, 0, 0, 0, 2, 0,
        0, 44, 0, 0, 0, 1, 0, 0, 45, 0, 0, 0, 0, 0, 0, 46, 0, 0, 0, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0,
        48, 0, 0, 0, 1, 0, 0, 49, 0, 0, 0, 1, 0, 0, 50, 0, 0, 0, 0, 0, 0, 51, 0, 0, 0, 0, 0, 0, 55,
        0, 0, 0, 1, 0, 0, 57, 0, 0, 0, 1, 0, 0, 60, 0, 0, 0, 0, 0, 0, 61, 0, 0, 0, 0, 0, 0, 64, 0,
        0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 23, 2, 17, 109,
        101, 116, 97, 100, 97, 116, 97, 46, 118, 101, 114, 115, 105, 111, 110, 0, 1, 0, 14, 0, 1,
        8, 0, 0, 0, 0, 0, 0, 0, 76, 2, 23, 2, 17, 109, 101, 116, 97, 100, 97, 116, 97, 46, 118,
        101, 114, 115, 105, 111, 110, 0, 14, 0, 14, 0,
    ];

    let api_key = 18;
    let api_version = 3;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn create_topics_request_v7_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 73, 0, 19, 0, 7, 0, 0, 1, 42, 0, 13, 97, 100, 109, 105, 110, 99, 108, 105, 101,
        110, 116, 45, 49, 0, 2, 9, 98, 97, 108, 97, 110, 99, 101, 115, 255, 255, 255, 255, 255,
        255, 1, 2, 15, 99, 108, 101, 97, 110, 117, 112, 46, 112, 111, 108, 105, 99, 121, 8, 99,
        111, 109, 112, 97, 99, 116, 0, 0, 0, 0, 117, 48, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn create_topics_response_v7_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 4, 92, 0, 0, 1, 42, 0, 0, 0, 0, 0, 2, 9, 98, 97, 108, 97, 110, 99, 101, 115, 222,
        159, 182, 217, 102, 152, 68, 189, 174, 152, 214, 59, 29, 216, 240, 198, 0, 0, 0, 0, 0, 0,
        1, 0, 1, 32, 15, 99, 108, 101, 97, 110, 117, 112, 46, 112, 111, 108, 105, 99, 121, 8, 99,
        111, 109, 112, 97, 99, 116, 0, 1, 0, 0, 17, 99, 111, 109, 112, 114, 101, 115, 115, 105,
        111, 110, 46, 116, 121, 112, 101, 9, 112, 114, 111, 100, 117, 99, 101, 114, 0, 5, 0, 0, 20,
        100, 101, 108, 101, 116, 101, 46, 114, 101, 116, 101, 110, 116, 105, 111, 110, 46, 109,
        115, 9, 56, 54, 52, 48, 48, 48, 48, 48, 0, 5, 0, 0, 21, 102, 105, 108, 101, 46, 100, 101,
        108, 101, 116, 101, 46, 100, 101, 108, 97, 121, 46, 109, 115, 6, 54, 48, 48, 48, 48, 0, 5,
        0, 0, 15, 102, 108, 117, 115, 104, 46, 109, 101, 115, 115, 97, 103, 101, 115, 20, 57, 50,
        50, 51, 51, 55, 50, 48, 51, 54, 56, 53, 52, 55, 55, 53, 56, 48, 55, 0, 5, 0, 0, 9, 102,
        108, 117, 115, 104, 46, 109, 115, 20, 57, 50, 50, 51, 51, 55, 50, 48, 51, 54, 56, 53, 52,
        55, 55, 53, 56, 48, 55, 0, 5, 0, 0, 40, 102, 111, 108, 108, 111, 119, 101, 114, 46, 114,
        101, 112, 108, 105, 99, 97, 116, 105, 111, 110, 46, 116, 104, 114, 111, 116, 116, 108, 101,
        100, 46, 114, 101, 112, 108, 105, 99, 97, 115, 1, 0, 5, 0, 0, 21, 105, 110, 100, 101, 120,
        46, 105, 110, 116, 101, 114, 118, 97, 108, 46, 98, 121, 116, 101, 115, 5, 52, 48, 57, 54,
        0, 5, 0, 0, 38, 108, 101, 97, 100, 101, 114, 46, 114, 101, 112, 108, 105, 99, 97, 116, 105,
        111, 110, 46, 116, 104, 114, 111, 116, 116, 108, 101, 100, 46, 114, 101, 112, 108, 105, 99,
        97, 115, 1, 0, 5, 0, 0, 22, 108, 111, 99, 97, 108, 46, 114, 101, 116, 101, 110, 116, 105,
        111, 110, 46, 98, 121, 116, 101, 115, 3, 45, 50, 0, 5, 0, 0, 19, 108, 111, 99, 97, 108, 46,
        114, 101, 116, 101, 110, 116, 105, 111, 110, 46, 109, 115, 3, 45, 50, 0, 5, 0, 0, 22, 109,
        97, 120, 46, 99, 111, 109, 112, 97, 99, 116, 105, 111, 110, 46, 108, 97, 103, 46, 109, 115,
        20, 57, 50, 50, 51, 51, 55, 50, 48, 51, 54, 56, 53, 52, 55, 55, 53, 56, 48, 55, 0, 5, 0, 0,
        18, 109, 97, 120, 46, 109, 101, 115, 115, 97, 103, 101, 46, 98, 121, 116, 101, 115, 8, 49,
        48, 52, 56, 53, 56, 56, 0, 5, 0, 0, 30, 109, 101, 115, 115, 97, 103, 101, 46, 100, 111,
        119, 110, 99, 111, 110, 118, 101, 114, 115, 105, 111, 110, 46, 101, 110, 97, 98, 108, 101,
        5, 116, 114, 117, 101, 0, 5, 0, 0, 23, 109, 101, 115, 115, 97, 103, 101, 46, 102, 111, 114,
        109, 97, 116, 46, 118, 101, 114, 115, 105, 111, 110, 8, 51, 46, 48, 45, 73, 86, 49, 0, 5,
        0, 0, 31, 109, 101, 115, 115, 97, 103, 101, 46, 116, 105, 109, 101, 115, 116, 97, 109, 112,
        46, 97, 102, 116, 101, 114, 46, 109, 97, 120, 46, 109, 115, 20, 57, 50, 50, 51, 51, 55, 50,
        48, 51, 54, 56, 53, 52, 55, 55, 53, 56, 48, 55, 0, 5, 0, 0, 32, 109, 101, 115, 115, 97,
        103, 101, 46, 116, 105, 109, 101, 115, 116, 97, 109, 112, 46, 98, 101, 102, 111, 114, 101,
        46, 109, 97, 120, 46, 109, 115, 20, 57, 50, 50, 51, 51, 55, 50, 48, 51, 54, 56, 53, 52, 55,
        55, 53, 56, 48, 55, 0, 5, 0, 0, 36, 109, 101, 115, 115, 97, 103, 101, 46, 116, 105, 109,
        101, 115, 116, 97, 109, 112, 46, 100, 105, 102, 102, 101, 114, 101, 110, 99, 101, 46, 109,
        97, 120, 46, 109, 115, 20, 57, 50, 50, 51, 51, 55, 50, 48, 51, 54, 56, 53, 52, 55, 55, 53,
        56, 48, 55, 0, 5, 0, 0, 23, 109, 101, 115, 115, 97, 103, 101, 46, 116, 105, 109, 101, 115,
        116, 97, 109, 112, 46, 116, 121, 112, 101, 11, 67, 114, 101, 97, 116, 101, 84, 105, 109,
        101, 0, 5, 0, 0, 26, 109, 105, 110, 46, 99, 108, 101, 97, 110, 97, 98, 108, 101, 46, 100,
        105, 114, 116, 121, 46, 114, 97, 116, 105, 111, 4, 48, 46, 53, 0, 5, 0, 0, 22, 109, 105,
        110, 46, 99, 111, 109, 112, 97, 99, 116, 105, 111, 110, 46, 108, 97, 103, 46, 109, 115, 2,
        48, 0, 5, 0, 0, 20, 109, 105, 110, 46, 105, 110, 115, 121, 110, 99, 46, 114, 101, 112, 108,
        105, 99, 97, 115, 2, 49, 0, 5, 0, 0, 12, 112, 114, 101, 97, 108, 108, 111, 99, 97, 116,
        101, 6, 102, 97, 108, 115, 101, 0, 5, 0, 0, 22, 114, 101, 109, 111, 116, 101, 46, 115, 116,
        111, 114, 97, 103, 101, 46, 101, 110, 97, 98, 108, 101, 6, 102, 97, 108, 115, 101, 0, 5, 0,
        0, 16, 114, 101, 116, 101, 110, 116, 105, 111, 110, 46, 98, 121, 116, 101, 115, 3, 45, 49,
        0, 5, 0, 0, 13, 114, 101, 116, 101, 110, 116, 105, 111, 110, 46, 109, 115, 10, 54, 48, 52,
        56, 48, 48, 48, 48, 48, 0, 4, 0, 0, 14, 115, 101, 103, 109, 101, 110, 116, 46, 98, 121,
        116, 101, 115, 11, 49, 48, 55, 51, 55, 52, 49, 56, 50, 52, 0, 5, 0, 0, 20, 115, 101, 103,
        109, 101, 110, 116, 46, 105, 110, 100, 101, 120, 46, 98, 121, 116, 101, 115, 9, 49, 48, 52,
        56, 53, 55, 54, 48, 0, 5, 0, 0, 18, 115, 101, 103, 109, 101, 110, 116, 46, 106, 105, 116,
        116, 101, 114, 46, 109, 115, 2, 48, 0, 5, 0, 0, 11, 115, 101, 103, 109, 101, 110, 116, 46,
        109, 115, 10, 54, 48, 52, 56, 48, 48, 48, 48, 48, 0, 5, 0, 0, 31, 117, 110, 99, 108, 101,
        97, 110, 46, 108, 101, 97, 100, 101, 114, 46, 101, 108, 101, 99, 116, 105, 111, 110, 46,
        101, 110, 97, 98, 108, 101, 6, 102, 97, 108, 115, 101, 0, 5, 0, 0, 0, 0,
    ];

    let api_key = 19;
    let api_version = 7;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn delete_topics_request_v6_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 52, 0, 20, 0, 6, 0, 0, 0, 4, 0, 13, 97, 100, 109, 105, 110, 99, 108, 105, 101,
        110, 116, 45, 49, 0, 2, 5, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 117, 48, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn describe_cluster_request_v1_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 27, 0, 60, 0, 1, 0, 0, 0, 7, 0, 13, 97, 100, 109, 105, 110, 99, 108, 105, 101,
        110, 116, 45, 49, 0, 0, 1, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn describe_configs_request_v4_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 36, 0, 32, 0, 4, 0, 0, 0, 5, 0, 13, 97, 100, 109, 105, 110, 99, 108, 105, 101,
        110, 116, 45, 49, 0, 2, 2, 5, 116, 101, 115, 116, 0, 0, 0, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn describe_groups_request_v1_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 22, 0, 15, 0, 1, 0, 0, 0, 0, 255, 255, 0, 0, 0, 1, 0, 6, 97, 98, 99, 97, 98, 99,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn describe_groups_response_v1_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 16, 0, 6, 97, 98, 99, 97, 98, 99, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let api_key = 15;
    let api_version = 1;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn fetch_request_v6_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 72, 0, 1, 0, 6, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 19, 136, 0, 0, 4,
        0, 0, 0, 16, 0, 1, 0, 0, 0, 1, 0, 11, 97, 98, 99, 97, 98, 99, 97, 98, 99, 97, 98, 0, 0, 0,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn fetch_request_v12_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 162, 0, 1, 0, 12, 0, 0, 0, 8, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 255, 255, 255, 255, 0, 0, 1, 244, 0, 0, 0, 1, 3, 32,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 5, 116, 101, 115, 116, 4, 0, 0, 0, 1, 255, 255, 255,
        255, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
        16, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 0, 16, 0, 0, 0, 0, 0, 0, 2, 255, 255, 255, 255, 0,
        0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 16, 0,
        0, 0, 0, 1, 1, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn fetch_request_v16_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 52, 0, 1, 0, 16, 0, 0, 0, 12, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 0, 0, 1, 244, 0, 0, 0, 1, 3, 32, 0, 0, 0, 0, 0, 0, 0,
        255, 255, 255, 255, 1, 1, 1, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn fetch_response_v12_000() -> Result<()> {
    let _guard = init_tracing()?;

    let api_key = 1;
    let api_version = 12;

    let expected = vec![
        0, 0, 0, 135, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 52, 239, 167, 250, 2, 5, 116, 101, 115, 116,
        4, 0, 0, 0, 1, 0, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 1, 0, 0, 0, 0, 0,
        0, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 1, 0, 0, 0, 0, 2, 0, 3, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 1, 255, 255, 255, 255, 1, 0, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn fetch_response_v12_001() -> Result<()> {
    let _guard = init_tracing()?;

    let api_key = 1;
    let api_version = 12;

    let expected = vec![
        0, 0, 1, 28, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 124, 92, 221, 217, 2, 5, 116, 101, 115, 116,
        4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 255, 255, 255, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0,
        0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 149, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 62, 0, 0, 0, 0, 2, 173, 206, 144, 5, 0, 0, 0, 0, 0, 0, 0, 0, 1, 141, 116, 152, 137, 53,
        0, 0, 1, 141, 116, 152, 137, 53, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 24,
        0, 0, 0, 6, 97, 98, 99, 6, 112, 113, 114, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 62, 0, 0, 0,
        0, 2, 173, 206, 144, 5, 0, 0, 0, 0, 0, 0, 0, 0, 1, 141, 116, 152, 137, 53, 0, 0, 1, 141,
        116, 152, 137, 53, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 24, 0, 0, 0, 6,
        97, 98, 99, 6, 112, 113, 114, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 1, 0, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn fetch_response_v12_002() -> Result<()> {
    let _guard = init_tracing()?;

    let api_key = 1;
    let api_version = 12;

    let expected = vec![
        0, 0, 1, 64, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 58, 96, 28, 234, 2, 5, 116, 101, 115, 116, 4,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 255, 255, 255, 255, 185, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 172, 0, 0, 0, 0, 2, 143,
        254, 2, 228, 0, 0, 0, 0, 0, 10, 0, 0, 1, 141, 116, 152, 137, 53, 0, 0, 1, 141, 116, 152,
        137, 53, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 20, 0, 0, 0, 4, 107, 49, 4,
        118, 49, 0, 20, 0, 0, 2, 4, 107, 50, 4, 118, 50, 0, 20, 0, 0, 4, 4, 107, 49, 4, 118, 51, 0,
        20, 0, 0, 6, 4, 107, 49, 4, 118, 52, 0, 20, 0, 0, 8, 4, 107, 51, 4, 118, 53, 0, 20, 0, 0,
        10, 4, 107, 50, 4, 118, 54, 0, 20, 0, 0, 12, 4, 107, 52, 4, 118, 55, 0, 20, 0, 0, 14, 4,
        107, 53, 4, 118, 56, 0, 20, 0, 0, 16, 4, 107, 53, 4, 118, 57, 0, 22, 0, 0, 18, 4, 107, 50,
        6, 118, 49, 48, 0, 22, 0, 0, 20, 4, 107, 54, 6, 118, 49, 49, 0, 0, 0, 0, 0, 1, 0, 3, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 1, 0, 0, 0, 0, 2, 0, 3, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 1, 255, 255, 255, 255, 1, 0, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn find_coordinator_request_v1_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 19, 0, 10, 0, 1, 0, 0, 0, 0, 255, 255, 0, 6, 97, 98, 99, 100, 101, 102, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn find_coordinator_request_v4_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 50, 0, 10, 0, 4, 0, 0, 0, 0, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99, 111,
        110, 115, 117, 109, 101, 114, 0, 0, 2, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn find_coordinator_response_v1_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 3, 234, 0, 40, 105, 112, 45, 49,
        48, 45, 50, 45, 57, 49, 45, 54, 54, 46, 101, 117, 45, 119, 101, 115, 116, 45, 49, 46, 99,
        111, 109, 112, 117, 116, 101, 46, 105, 110, 116, 101, 114, 110, 97, 108, 0, 0, 35, 132,
    ];

    let api_key = 10;
    let api_version = 1;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn heartbeat_request_v4_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 58, 0, 12, 0, 4, 0, 0, 40, 48, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 0, 0, 0, 0, 5, 49, 48, 48, 48, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn init_producer_id_request_v4_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 43, 0, 22, 0, 4, 0, 0, 0, 2, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 112,
        114, 111, 100, 117, 99, 101, 114, 0, 0, 127, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn join_group_request_v9_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 154, 0, 11, 0, 9, 0, 0, 0, 5, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 0, 0, 175, 200, 0, 4, 147, 224, 1, 0, 9, 99,
        111, 110, 115, 117, 109, 101, 114, 3, 6, 114, 97, 110, 103, 101, 27, 0, 3, 0, 0, 0, 1, 0,
        4, 116, 101, 115, 116, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 19,
        99, 111, 111, 112, 101, 114, 97, 116, 105, 118, 101, 45, 115, 116, 105, 99, 107, 121, 31,
        0, 3, 0, 0, 0, 1, 0, 4, 116, 101, 115, 116, 0, 0, 0, 4, 255, 255, 255, 255, 0, 0, 0, 0,
        255, 255, 255, 255, 255, 255, 0, 1, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn leave_group_request_v5_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 85, 0, 13, 0, 5, 0, 0, 0, 11, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 2, 5, 49, 48, 48, 48, 0, 29, 116, 104, 101, 32,
        99, 111, 110, 115, 117, 109, 101, 114, 32, 105, 115, 32, 98, 101, 105, 110, 103, 32, 99,
        108, 111, 115, 101, 100, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn list_groups_request_v4_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 26, 0, 16, 0, 4, 0, 0, 0, 84, 0, 13, 97, 100, 109, 105, 110, 99, 108, 105, 101,
        110, 116, 45, 49, 0, 1, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn list_offsets_response_v0_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 67, 0, 0, 0, 0, 0, 0, 0, 1, 0, 11, 97, 98, 99, 97, 98, 99, 97, 98, 99, 97, 98, 0,
        0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 18, 37, 164, 0, 0, 0, 0, 0, 17, 233,
        252, 0, 0, 0, 0, 0, 17, 198, 100, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let api_key = 2;
    let api_version = 0;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn list_partition_reassignments_request_v0_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 49, 0, 46, 0, 0, 0, 0, 0, 7, 0, 13, 97, 100, 109, 105, 110, 99, 108, 105, 101,
        110, 116, 45, 49, 0, 0, 0, 117, 48, 2, 5, 116, 101, 115, 116, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        0, 0, 2, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn metadata_request_v12_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 53, 0, 3, 0, 12, 0, 0, 0, 5, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 112,
        114, 111, 100, 117, 99, 101, 114, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
        116, 101, 115, 116, 0, 1, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn metadata_request_v12_001() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 31, 0, 3, 0, 12, 0, 0, 0, 1, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 112,
        114, 111, 100, 117, 99, 101, 114, 0, 1, 1, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn metadata_response_v12_0000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 92, 0, 0, 0, 5, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 13, 107, 97, 102, 107, 97, 45, 115,
        101, 114, 118, 101, 114, 0, 0, 35, 132, 0, 0, 23, 82, 118, 81, 119, 114, 89, 101, 103, 83,
        85, 67, 107, 73, 80, 107, 97, 105, 65, 90, 81, 108, 81, 0, 0, 0, 0, 2, 0, 3, 5, 116, 101,
        115, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 128, 0, 0, 0, 0, 0,
    ];

    let api_key = 3;
    let api_version = 12;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
fn offset_commit_request_v9_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 120, 0, 8, 0, 9, 0, 0, 0, 10, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 0, 0, 0, 0, 5, 49, 48, 48, 48, 0, 2, 5, 116,
        101, 115, 116, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
        0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn offset_fetch_request_v3_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 65, 0, 9, 0, 3, 0, 0, 0, 0, 255, 255, 0, 3, 97, 98, 99, 0, 0, 0, 2, 0, 5, 116,
        101, 115, 116, 50, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 5, 116, 101, 115,
        116, 49, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn offset_fetch_request_v9_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 76, 0, 9, 0, 9, 0, 0, 0, 7, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99, 111,
        110, 115, 117, 109, 101, 114, 0, 2, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 0, 255, 255, 255, 255, 2, 5, 116, 101, 115,
        116, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 1, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn offset_for_leader_request_v0_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 31, 0, 23, 0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 0, 1, 0, 11, 97, 98, 99, 97, 98, 99,
        97, 98, 99, 97, 98, 0, 0, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn produce_request_v9_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 120, 0, 0, 0, 9, 0, 0, 0, 6, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 112,
        114, 111, 100, 117, 99, 101, 114, 0, 0, 255, 255, 0, 0, 5, 220, 2, 5, 116, 101, 115, 116,
        2, 0, 0, 0, 0, 72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59, 255, 255, 255, 255, 2, 67, 41, 231,
        61, 0, 0, 0, 0, 0, 0, 0, 0, 1, 141, 116, 152, 137, 53, 0, 0, 1, 141, 116, 152, 137, 53, 0,
        0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 18, 0, 0, 0, 1, 6, 100, 101, 102, 0, 0,
        0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}

#[test]
fn produce_response_v9_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 51, 0, 0, 0, 6, 0, 2, 5, 116, 101, 115, 116, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 2, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        0, 0, 0,
    ];

    let api_key = 0;
    let api_version = 9;

    assert_eq!(
        expected,
        Frame::response_from_bytes(&expected, api_key, api_version)
            .and_then(|frame| Frame::response(frame.header, frame.body, api_key, api_version))?
    );

    Ok(())
}

#[test]
pub fn sync_group_request_v5_000() -> Result<()> {
    let _guard = init_tracing()?;

    let expected = vec![
        0, 0, 0, 113, 0, 14, 0, 5, 0, 0, 0, 6, 0, 16, 99, 111, 110, 115, 111, 108, 101, 45, 99,
        111, 110, 115, 117, 109, 101, 114, 0, 20, 116, 101, 115, 116, 45, 99, 111, 110, 115, 117,
        109, 101, 114, 45, 103, 114, 111, 117, 112, 0, 0, 0, 0, 5, 49, 48, 48, 48, 0, 9, 99, 111,
        110, 115, 117, 109, 101, 114, 6, 114, 97, 110, 103, 101, 2, 5, 49, 48, 48, 48, 33, 0, 3, 0,
        0, 0, 1, 0, 4, 116, 101, 115, 116, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 255,
        255, 255, 255, 0, 0,
    ];

    assert_eq!(
        expected,
        Frame::request_from_bytes(&expected)
            .and_then(|frame| Frame::request(frame.header, frame.body))?
    );

    Ok(())
}