// This source code form is subject to the terms of the mozilla public
// license, v. 2.0. if a copy of the mpl was not distributed with this
// file, you can obtain one at https://mozilla.org/mpl/2.0/.

use crate::Event;

pub struct Connection {}

impl Connection {
    pub fn send(event: impl Event) {}
}

/// Represents either the [Default](DisplayServer::Default) display server, or the display server
/// [Of](DisplayServer::Of) the given name.
pub enum DisplayServer<'a> {
    /// Represents the default display server name, as provided by the `DISPLAY` environment
    /// variable.
    Default,
    /// Represents a display server of the given name.
    Of(&'a str),
}

/// Initiates a [Connection] to the X server.
///
/// The given `display_server` can be either the [Default](DisplayServer::Default) display server,
/// as provided by the `DISPLAY` environment variable on POSIX-compliant systems, or a display
/// server [Of](DisplayServer::Of) the given `&str` name. It specifies which display server (a.k.a.
/// X server) the connection will be made to.
///
/// # Examples
/// ```rust
/// // Connect to the X server on the default display server.
/// let conn = xrs::connect(xrs::DisplayServer::Default);
/// ```
/// ```rust
/// // Connect to the display server named `:0`: specifically refers to a local display server.
/// let local_conn = xrs::connect(xrs::DisplayServer::Of(":0"));
/// ```
pub fn connect(display_server: DisplayServer) -> Connection {
    Connection {}
}
