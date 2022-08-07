// This source code form is subject to the terms of the mozilla public
// license, v. 2.0. if a copy of the mpl was not distributed with this
// file, you can obtain one at https://mozilla.org/mpl/2.0/.

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
