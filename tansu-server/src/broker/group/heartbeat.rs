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

use std::sync::{Arc, Mutex, MutexGuard};

use tansu_kafka_sans_io::Body;

use crate::{coordinator::group::Coordinator, Result};

#[derive(Debug)]
pub struct HeartbeatRequest {
    pub groups: Arc<Mutex<Box<dyn Coordinator>>>,
}

impl HeartbeatRequest {
    pub fn groups_lock(&self) -> Result<MutexGuard<'_, Box<dyn Coordinator>>> {
        self.groups.lock().map_err(|error| error.into())
    }
}

impl HeartbeatRequest {
    pub(crate) fn response(
        &self,
        group_id: &str,
        generation_id: i32,
        member_id: &str,
        group_instance_id: Option<&str>,
    ) -> Result<Body> {
        self.groups_lock().and_then(|mut coordinator| {
            coordinator.heartbeat(group_id, generation_id, member_id, group_instance_id)
        })
    }
}