// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod rw;

use async_std::net::TcpStream;
#[cfg(unix)]
use async_std::os::unix::net::UnixStream;

use crate::stream::Stream;
use async_std::{io, io::WriteExt};
use bytes::BytesMut;
use chumsky::prelude::*;
use std::{
	env,
	fmt,
	fmt::Formatter,
	net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use xrb::{
	connection::{
		ConnectionAuthenticationError,
		ConnectionFailure,
		ConnectionResponse,
		ConnectionSuccess,
		InitConnection,
	},
	message::Request,
};
use xrbk::{Readable, Writable};

enum BitmapFormat {
	U8,
	U16,
	U32,
}

pub struct Client {
	stream: Stream,
	/// A buffer to read bytes into.
	buffer: BytesMut,
	// TODO: store info provided by the X server
}

pub enum ConnectError {
	/// An error parsing the display name specified by the `DISPLAY` environment
	/// variable if [`Display::Default`] is specified.
	Parse(DisplayNameParseError),
	Io(io::Error),

	Failed(ConnectionFailure),
	Auth(ConnectionAuthenticationError),
}

impl Client {
	pub async fn send<Req: Request>(&mut self, request: Req) -> Result<(), io::Error> {
		if let Err(error) = request.write_to(&mut self.stream) {
			return Err(io::Error::new(io::ErrorKind::Other, error));
		}

		self.stream.flush().await?;

		// TODO: replies

		Ok(())
	}

	pub async fn connect(display: Display, auth: Option<AuthInfo>) -> Result<Self, ConnectError> {
		// If `Display::Default` is specified, parse the display name.
		let DisplayName {
			protocol,
			hostname,
			display,
			screen: _,
		} = match display {
			Display::Default => {
				let display_env = &env::var("DISPLAY")
					.expect("expected DISPLAY environment variable for Display::Default");

				match DisplayName::parse(display_env) {
					Ok(display_name) => display_name,
					Err(error) => return Err(ConnectError::Parse(error)),
				}
			},

			Display::Specific(name) => name,
		};

		// Open the appropriate data stream.
		let mut stream = Self::open_stream(&protocol, &hostname, display).await?;

		let (auth_name, auth_data) = match auth {
			Some(AuthInfo {
				protocol_name,
				protocol_data,
			}) => (&*protocol_name, &*protocol_data),

			None => ("", ""),
		};
		let message = InitConnection {
			auth_protocol_name: auth_name.into(),
			auth_protocol_data: auth_data.into(),
		};

		// Serialize the `message`.
		message.write_to(&mut stream).unwrap();

		// Send the `InitConnection` message.
		if let Err(error) = stream.flush().await {
			return Err(ConnectError::Io(error));
		}

		// Receive the connection response.
		let response = match ConnectionResponse::read_from(&mut stream) {
			Ok(response) => response,

			Err(error) => {
				return Err(ConnectError::Io(io::Error::new(
					io::ErrorKind::Other,
					error,
				)))
			},
		};

		match response {
			ConnectionResponse::Success(ConnectionSuccess { .. }) => Ok(Self {
				stream,
				buffer: BytesMut::with_capacity(4096),
			}),

			ConnectionResponse::Failed(failure) => Err(ConnectError::Failed(failure)),
			ConnectionResponse::Authenticate(auth_error) => Err(ConnectError::Auth(auth_error)),
		}
	}
}

impl Client {
	async fn open_stream(
		protocol: &Option<Protocol>, hostname: &Option<Hostname>, display: i16,
	) -> Result<Stream, ConnectError> {
		Ok(match (protocol, hostname) {
			// IPv4 with address
			(Some(Protocol::Inet), Some(Hostname::Other(hostname))) => Stream::TcpStream(
				match Self::open_tcp_stream(Some(IpType::V4), Some(&*hostname), display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				},
			),

			// IPv6 with address
			(None, Some(Hostname::Inet6(hostname)))
			| (Some(Protocol::Tcp), Some(Hostname::Inet6(hostname)))
			| (Some(Protocol::Inet6), Some(Hostname::Inet6(hostname)))
			| (Some(Protocol::Inet6), Some(Hostname::Other(hostname))) => Stream::TcpStream(
				match Self::open_tcp_stream(Some(IpType::V6), Some(&*hostname), display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				},
			),

			// TCP with address but unspecified IP version
			(None, Some(Hostname::Other(hostname)))
			| (Some(Protocol::Tcp), Some(Hostname::Other(hostname))) => Stream::TcpStream(
				match Self::open_tcp_stream(None, Some(&*hostname), display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				},
			),

			// IPv4 without address
			(Some(Protocol::Inet), None) => Stream::TcpStream(
				match Self::open_tcp_stream(Some(IpType::V4), None, display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				},
			),

			// IPv6 without address
			(Some(Protocol::Inet6), None) => Stream::TcpStream(
				match Self::open_tcp_stream(Some(IpType::V6), None, display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				},
			),

			// TCP without address and unspecified IP version
			(Some(Protocol::Tcp), None) => {
				Stream::TcpStream(match Self::open_tcp_stream(None, None, display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				})
			},

			// Unix domain sockets (IPC)
			#[cfg(unix)]
			(None, None) // unix domain sockets are default on #[cfg(unix)]
			| (None, Some(Hostname::Unix)) // hostname is "unix"
			| (Some(Protocol::Unix), None) // protocol is "unix"
			| (Some(Protocol::Unix), Some(Hostname::Unix)) => { // both are "unix"
				Stream::UnixStream(match Self::open_unix_stream(display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				})
			},

			// Default (if neither protocol nor hostname are specified) on #[cfg(not(unix))].
			#[cfg(not(unix))]
			(None, None) => {
				Stream::TcpStream(match Self::open_tcp_stream(None, None, display).await {
					Ok(stream) => stream,
					Err(error) => return Err(ConnectError::Io(error)),
				})
			},

			// DECnet was orphaned in the Linux kernel in 2010...
			(None, Some(Hostname::DecNet(_))) => {
				unimplemented!(
					"DECnet is not implemented; it was orphaned in the Linux kernel back in 2010"
				)
			},

			// TODO: improve errors
			_ => return Err(ConnectError::Parse(DisplayNameParseError::IllFormatted)),
		})
	}

	async fn open_tcp_stream(
		ip_type: Option<IpType>, hostname: Option<&str>, display: i16,
	) -> Result<TcpStream, io::Error> {
		const TCP_PORT: u16 = 6000;

		let port = ((TCP_PORT as i16) + display) as u16;

		match (ip_type, hostname) {
			// IP version interpreted
			(None, Some(address)) => TcpStream::connect((address.parse::<IpAddr>()?, port)),

			// IPv6 with address
			(Some(IpType::V6), Some(address)) => {
				TcpStream::connect((address.parse::<Ipv6Addr>()?, port))
			},
			// IPv6 localhost
			(Some(IpType::V6), None) => TcpStream::connect((Ipv6Addr::LOCALHOST, port)),

			// IPv4 with address
			(Some(IpType::V4), Some(address)) => {
				TcpStream::connect((address.parse::<Ipv4Addr>()?, port))
			},
			// IPv4 localhost (also the fallback)
			(Some(IpType::V4), None) | (None, None) => {
				TcpStream::connect((Ipv4Addr::LOCALHOST, port))
			},
		}
	}

	#[cfg(unix)]
	async fn open_unix_stream(display: i16) -> Result<UnixStream, io::Error> {
		// FIXME: see if we need to check /var/tsol/doors/.X11-unix/X on Solaris
		let socket = format!("/tmp/.X11-unix/X{}", display);

		UnixStream::connect(socket)
	}
}

pub enum Display {
	Default,
	Specific(DisplayName),
}

pub struct DisplayName {
	pub protocol: Option<Protocol>,
	pub hostname: Option<Hostname>,

	pub display: i16,
	pub screen: Option<i16>,
}

pub enum Hostname {
	DecNet(String),
	Inet6(String),
	#[cfg(unix)]
	Unix,

	Other(String),
}

impl DisplayName {
	pub const fn new(display: i16) -> Self {
		Self {
			protocol: None,
			hostname: None,

			display,
			screen: None,
		}
	}

	pub fn protocol(&mut self, protocol: Protocol) -> &mut Self {
		self.protocol = Some(protocol);

		self
	}

	pub fn hostname(&mut self, hostname: Hostname) -> &mut Self {
		self.hostname = Some(hostname);

		self
	}

	pub fn screen(&mut self, screen: i16) -> &mut Self {
		self.screen = Some(screen);

		self
	}
}

impl fmt::Display for DisplayName {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self.protocol {
			Some(Protocol::Tcp) => write!(f, "{}/", Protocol::Tcp)?,
			Some(Protocol::Inet) => write!(f, "{}/", Protocol::Inet)?,
			Some(Protocol::Inet6) => write!(f, "{}/", Protocol::Inet6)?,

			#[cfg(unix)]
			Some(Protocol::Unix) => write!(f, "{}/", Protocol::Unix)?,

			#[allow(deprecated)]
			Some(Protocol::DecNet) | None => {},
		}

		if let Some(hostname) = &self.hostname {
			write!(f, hostname)?;

			#[allow(deprecated)]
			if let Some(Protocol::DecNet) = &self.protocol {
				write!(f, "::")?;
			} else {
				write!(f, ":")?;
			}
		}

		write!(f, self.display)?;

		if let Some(screen) = &self.screen {
			write!(f, ".{}", screen)?;
		}

		Ok(())
	}
}

pub enum DisplayNameParseError {
	IllFormatted,
	UnrecognizedProtocol,
}

impl DisplayName {
	pub fn parse(mut name: &str) -> Result<Self, IllFormatted> {
		let protocol = if let Some((protocol, _name)) = name.split_once('/') {
			name = _name;

			Some(protocol)
		} else {
			None
		};

		let hostname = if let Some((hostname, _name)) = name.rsplit_once(':') {
			name = _name;

			let first = |name| name.get(0);
			let last = |name| name.get(name.len() - 1);

			Some(if let Some(':') = last(hostname) {
				Hostname::DecNet(hostname[..hostname.len() - 1].to_owned())
			} else if let Some('[') = first(hostname) && let Some(']') = last(hostname) {
				Hostname::Inet6(hostname[1..hostname.len() - 1].to_owned())
			} else {
				match hostname {
					#[cfg(unix)]
					"unix" => Hostname::Unix,

					other => Hostname::Other(other.to_owned()),
				}
			})
		} else {
			None
		};

		let screen = if let Some((_name, screen)) = name.rsplit_once('.') {
			name = _name;

			Some(screen.parse::<i16>()?)
		} else {
			None
		};

		Ok(Self {
			protocol: if let Some(protocol) = protocol {
				Some(match protocol {
					"tcp" => Protocol::Tcp,
					"inet" => Protocol::Inet,
					"inet6" => Protocol::Inet6,

					#[cfg(unix)]
					"unix" => Protocol::Unix,

					_ => return Err(DisplayNameParseError::UnrecognizedProtocol),
				})
			} else {
				None
			},

			hostname,

			display: name.parse::<i16>()?,
			screen,
		})
	}
}

pub enum Protocol {
	/// A connection is established over DECnet.
	#[deprecated]
	DecNet,

	/// A connection is established over TCP.
	Tcp,
	/// A connection is established over TCP using IPv4.
	Inet,
	/// A connection is established over TCP using IPv6.
	Inet6,

	/// A connection is established over unix domain sockets (a form of
	/// inter-process communication, a.k.a. IPC).
	#[cfg(unix)]
	Unix,
}

impl fmt::Display for Protocol {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			#[allow(deprecated)]
			Protocol::DecNet => Err(fmt::Error),

			Protocol::Tcp => write!(f, "tcp"),
			Protocol::Inet => write!(f, "inet"),
			Protocol::Inet6 => write!(f, "inet6"),

			#[cfg(unix)]
			Protocol::Unix => write!(f, "unix"),
		}
	}
}

enum IpType {
	V4,
	V6,
}

pub struct AuthInfo {
	pub protocol_name: String,
	pub protocol_data: String,
}
