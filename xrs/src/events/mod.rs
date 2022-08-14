// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod init_connection;

/// Contains [Event]s that notify listeners about actions which have already occurred.
pub mod notif {}

/// Contains [Event]s that are sent to query a particular piece of information.
///
/// Queries only retrieve information, they _do not_ affect the operation of anything else within
/// X.
pub mod query {}

/// Contains [Event]s sent in reply to a particular request.
pub mod reply {
	pub use super::init_connection::InitConnectionReply as InitConnection;
}

/// Contains [Event]s that request an action to be completed.
pub mod req {
	pub use super::init_connection::InitConnectionRequest as InitConnection;
}

/// An event that can be received from or sent to the X server.
pub trait Event {
	fn raw_event(self) -> Box<dyn xrb::RawEvent>;
}
