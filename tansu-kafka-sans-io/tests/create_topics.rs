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

use std::collections::BTreeMap;
use tansu_kafka_model::MessageKind;
use tansu_kafka_sans_io::MESSAGE_META;

#[test]
fn check_message_meta() {
    assert!(BTreeMap::from(MESSAGE_META).contains_key("CreateTopicsRequest"));

    let meta = BTreeMap::from(MESSAGE_META);

    let message = meta.get("CreateTopicsRequest").unwrap();
    assert_eq!(19, message.api_key);
    assert_eq!(MessageKind::Request, message.message_kind);

    let structures = message.structures();
    let keys: Vec<&&str> = structures.keys().collect();
    assert_eq!(
        vec![
            &"CreatableReplicaAssignment",
            &"CreatableTopic",
            &"CreateableTopicConfig"
        ],
        keys
    );
}