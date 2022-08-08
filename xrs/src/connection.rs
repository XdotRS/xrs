// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;

use async_std::io::{self, Read, Write};

use async_std::net::TcpStream;
use async_std::os::unix::net::UnixStream;

use regex::Regex;

use crate::Event;

/// Marker trait allowing the generic usage of either [TcpStream](async_std::net::TcpStream) or
/// [UnixStream](async_std::os::unix::net::UnixStream).
///
/// # Example usage
/// ```rust
/// async fn connect(address: &str, use_tcp: bool) -> Result<Box<dyn Socket>, io::Error> {
///     if use_tcp {
///         Ok(Box::new(TcpStream::connect(address).await?))
///     } else {
///         Ok(Box::new(UnixStream::connect(address).await?))
///     }
/// }
/// ```
trait Socket: Read + Write {}

impl Socket for TcpStream {}
impl Socket for UnixStream {}

pub struct Connection {
    _socket: Box<dyn Socket>,
}

impl Connection {
    pub async fn send(&self, _event: &dyn Event) {
        todo!()
    }

    /// Parses a [`DisplayServer`] enum, returns the connected socket and the screen number.
    ///
    /// If a hostname was provided, this function connects to a TCP socket with the given hostname
    /// and the display name as the port (_hostname_`:`_display name_). Otherwise, this function
    /// connects to a Unix socket, a.k.a. an IPC socket, with the given display name
    /// (`/tmp/X11-unix/X`_display name_).
    ///
    /// The returned `String` is the preferred screen number which the connection to the X server
    /// should include. If no screen number was given in the [DisplayServer], it defaults to `0`.
    async fn init(display_server: Server) -> Result<(Box<dyn Socket>, String), io::Error> {
        let address = match display_server {
            Server::Default => {
                // If the default X server is chosen, return the contents of the `DISPLAY`
                // environment variable.
                env::var("DISPLAY").expect("Couldn't fetch `DISPLAY` environment variable")
            }
            Server::Address(address) => address,
        };

        // This is some pretty incomprehensible regex. If you want to know the specifics, you can
        // put the regex (`(?:\w+\/)?(?:(\w+):)?(\w+)(?:\.(\w+))?`) into https://regexr.com/ or
        // a similar tool that can explain it, but its purpose here is to parse the display server
        // string.
        //
        // The X server string can be in any of the following valid formats:
        // - `protocol/hostname:display_name.screen_number`
        // - `hostname:display_name.screen_number`
        // - `hostname:display_name`
        // - `display_name`
        // - `display_name.screen_number`
        //
        // The regex will throw away the protocol part, as we can infer the protocol based on the
        // hostname if provided or that it will be a unix domain socket if not. It will then
        // capture three groups: the `hostname`, if present, the `display_name`, and the
        // `screen_number`, if present. These capture groups are numbered `0`, `1`, and `2`.
        let captures = Regex::new(r"(?:\w+\/)?(?:(\w+):)?(\w+)(?:\.(\w+))?")
            .unwrap() // This will panic if it is invalid regex; luckily, it is not.
            .captures(&address) // Apply the regex to our text.
            .expect("No display specified"); // Panic if no `display_name` was given.

        // We want to test for the `hostname`'s presence, so we leave it as an [Option].
        let hostname = captures.get(0);
        // The `display_name` is required, so we `unwrap()` it and get the match as a [`&str`].
        let display_name = captures.get(1).unwrap().as_str();
        // If the `screen_number` is [`None`], we map it to the default of `0`. Otherwise, we
        // convert it to a [`String`].
        let screen_number = String::from(captures.get(2).map_or("0", |num| num.as_str()));

        if hostname.is_some() {
            let hostname = hostname.unwrap().as_str();

            // A remote hostname was provided; we connect to a TCP socket.
            Ok((
                Box::new(TcpStream::connect(format!("{}:{}", hostname, display_name)).await?),
                screen_number,
            ))
        } else {
            // No remote hostname was provided; we connect to a Unix domain socket.
            Ok((
                Box::new(UnixStream::connect(format!("/tmp/X11-unix/X{}", display_name)).await?),
                screen_number,
            ))
        }
    }
}

/// Represents either the default X server, or the X server of the given address.
pub enum Server {
    /// Represents the default X server address, as provided by the `DISPLAY` environment variable.
    ///
    /// This will typically default to `:1`, which is a local X server address (connecting over
    /// Unix domain sockets) with a socket name of `/tmp/X11-unix/X1`, though it can be changed.
    Default,
    /// Represents an X server of the given address.
    ///
    /// The X server address is given in the following format:
    /// ```text
    /// [[protocol/]hostname:]display[.screen]
    /// ```
    /// where `[]` denotes a field (or group of fields) as being optional.
    ///
    /// `protocol/` represents the protocol used to connect to the X server - XRS ignores this.
    ///
    /// `hostname:` represents the hostname of a remote machine on which the X server is running.
    /// XRS will connect over TCP to the given host if the hostname is provided.
    ///
    /// `display` represents the X display XRS will connect to. The default display is typically
    /// `1`.
    ///
    /// `screen` represents the screen XRS will connect to; if not present, XRS will default to
    /// `0`.
    ///
    /// # Examples
    /// - inet/remoteserver:1.0
    /// - inet/remoteserver:1
    /// - inet6/remoteserver:1.0
    /// - inet6/remoteserver:1
    /// - remoteserver:1.0
    /// - remoteserver:1
    /// - :1.0
    /// - :1
    /// - 1.0
    /// - 1
    Address(String),
}

/// Initiates a [Connection] to the X server.
///
/// The given `display_server` can be either the [Default](DisplayServer::Default) display server,
/// as provided by the `DISPLAY` environment variable on POSIX-compliant systems, or a display
/// server [Of](DisplayServer::Of) the given [`&str`] name. It specifies which display server
/// (a.k.a. X server) the connection will be made to.
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
pub async fn connect(server: Server) -> Result<Connection, io::Error> {
    let (socket, _screen_number) = Connection::init(server).await?;
    let conn = Connection { _socket: socket };

    conn.send(&crate::req::InitConnection {}).await;

    Ok(conn)
}
