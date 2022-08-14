// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::events::Event;

/// A reply to an [xrs::req::InitConnection](crate::req::InitConnection) request.
///
/// Contains the result of the [xrs::req::InitConnection](crate::req::InitConnection) request
/// and additional data. See [xrb::ConnectionInitResult] for more information on the contents
/// of the result.
///
/// Associated request: [xrs::req::InitConnection](crate::req::InitConnection)
pub struct InitConnectionReply<'a> {
	result: xrb::ConnectionInitResult<'a>,
}

impl Event for InitConnectionReply<'static> {
	fn raw_event(self) -> Box<dyn xrb::RawEvent> {
		Box::new(xrb::replies::ConnectionInit::<'static> {
			result: self.result,
		})
	}
}

/// A request to initialize a connection with the X server.
///
/// Associated reply: [xrs::reply::InitConnection](crate::reply::InitConnection)
pub struct InitConnectionRequest {}

impl Event for InitConnectionRequest {
	fn raw_event(self) -> Box<dyn xrb::RawEvent> {
		Box::new(xrb::requests::ConnectionInit {
			byte_order: xrb::ByteOrder::native(),
			protocol_major_version: xrb::PROTOCOL_MAJOR_VERSION,
			protocol_minor_version: xrb::PROTOCOL_MINOR_VERSION,
			auth_protocol_name: "",
			auth_data: "",
		})
	}
}
