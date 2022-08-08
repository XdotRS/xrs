// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Contains [Event]s that notify listeners about actions which have already occurred.
pub mod notifications {}
/// Contains [Event]s that are sent to query a particular piece of information.
///
/// Queries only retrieve information, they _do not_ affect the operation of anything else within
/// X.
pub mod queries {}
/// Contains [Event]s sent in reply to a particular request.
pub mod replies {
    use super::{Event, RawEvent};

    pub struct InitConnection<'a> {
        result: xrb::ConnectionInitResult<'a>,
    }

    impl Event for InitConnection<'static> {
        fn raw_event(self) -> Box<dyn RawEvent> {
            Box::new(xrb::ConnectionInitReply::<'static> {
                result: self.result,
            })
        }
    }
}
/// Contains [Event]s that request an action to be completed.
pub mod requests {
    use super::{Event, RawEvent};

    pub struct InitConnection {}

    impl Event for InitConnection {
        fn raw_event(self) -> Box<dyn RawEvent> {
            Box::new(xrb::ConnectionInitRequest {
                byte_order: xrb::ByteOrder::native(),
                protocol_major_version: xrb::PROTOCOL_MAJOR_VERSION,
                protocol_minor_version: xrb::PROTOCOL_MINOR_VERSION,
                auth_protocol_name: "",
                auth_data: "",
            })
        }
    }
}

/// An event that can be received from or sent to the X server.
pub trait Event {
    fn raw_event(self) -> Box<dyn RawEvent>;
}
pub trait RawEvent {}

impl RawEvent for xrb::ConnectionInitRequest<'_> {}
impl RawEvent for xrb::ConnectionInitReply<'_> {}
