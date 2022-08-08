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
pub mod replies {}
/// Contains [Event]s that request an action to be completed.
pub mod requests {}

/// An event that can be received from or sent to the X server.
pub trait Event {}
