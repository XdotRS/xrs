// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use regex::{Match, Regex};
use std::fmt::Formatter;
use std::{env, fmt};
use xrb::message::Request;
use xrb::{
	connection::ImageEndianness,
	visual::{Format, Screen},
	Keycode,
};

enum BitmapFormat {
	U8,
	U16,
	U32,
}

pub struct Client {
	server_major_version: u16,
	server_minor_version: u16,

	vendor: String,
	release_number: u32,

	resource_id_base: u32,
	resource_id_mask: u32,

	image_byte_order: ImageEndianness,
	bitmap_scanline_unit: BitmapFormat,
	bitmap_scanline_padding: BitmapFormat,

	pixmap_formats: Vec<Format>,
	screens: Vec<Screen>,

	motion_buffer_size: u32,

	maximum_request_length: u16,

	min_keycode: Keycode,
	max_keycode: Keycode,
}

impl Client {
	pub async fn connect(
		display: Display,
		auth: Option<AuthInfo>,
	) -> Result<Self, ConnectionError> {
		let display_name = match display {
			Display::Default => DisplayName::parse(
				&env::var("DISPLAY")
					.expect("expected DISPLAY environment variable for Display::Default"),
			),

			Display::Specific(name) => name,
		};

		let protocol_type = match &display_name.protocol {
			Some(protocol) => protocol.protocol_type(),

			None if display_name.hostname.is_some() => ProtocolType::Tcp,
			None => ProtocolType::Unix,
		};

		todo!()
	}
}

impl Drop for Client {
	// disconnect
	fn drop(&mut self) {
		todo!()
	}
}

pub enum Display {
	Default,
	Specific(DisplayName),
}

pub struct DisplayName {
	pub protocol: Option<Protocol>,
	pub hostname: Option<String>,

	pub display_num: isize,
	pub screen_num: Option<isize>,
}

impl DisplayName {
	pub const fn new(display_num: isize) -> Self {
		Self {
			protocol: None,
			hostname: None,

			display_num,
			screen_num: None,
		}
	}

	pub fn protocol(&mut self, protocol: Protocol) -> &mut Self {
		self.protocol = Some(protocol);

		self
	}

	pub fn hostname(&mut self, hostname: String) -> &mut Self {
		self.hostname = Some(hostname);

		self
	}

	pub fn screen_num(&mut self, screen_num: isize) -> &mut Self {
		self.screen_num = Some(screen_num);

		self
	}
}

impl fmt::Display for DisplayName {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self.protocol {
			Some(Protocol::Tcp) => write!(f, "{}/", "tcp")?,
			Some(Protocol::Inet) => write!(f, "{}/", "inet")?,
			Some(Protocol::Inet6) => write!(f, "{}/", "inet6")?,

			Some(Protocol::Unix) => write!(f, "{}/", "unix")?,

			_ => {}
		}

		if let Some(hostname) = &self.hostname {
			write!(f, hostname)?;

			if let Some(Protocol::DecNet) = &self.protocol {
				write!(f, "::")?;
			} else {
				write!(f, ":")?;
			}
		}

		write!(f, self.display_num)?;

		if let Some(screen_num) = &self.screen_num {
			write!(f, ".{}", screen_num)?;
		}

		Ok(())
	}
}

pub enum DisplayNameParseError {
	IllFormatted,
	UnrecognizedProtocol,
}

impl DisplayName {
	pub fn parse(string: &str) -> Result<Self, IllFormatted> {
		// A regular expression to parse the display name format:
		// ( protocol `/` )? ( hostname `:` `:`? )? display_num ( `.` screen_num )?
		//
		// # Examples
		// - inet6/example.com:1.0
		// - inet6/1.0
		// - 1.0
		// - example.com:1
		let regex = Regex::new(
			r"(?x)
				^
				(?:(?<protocol>\w+)/)?
				(?:(?<hostname>.+)(?<host_separator>::?))?
				(?<display_num>-?\d+)
				(?:\.(?<screen_num>-?\d+))?
				$
			",
		)
		.unwrap();

		let captures = match regex.captures(string) {
			Some(captures) => captures,
			None => return Err(DisplayNameParseError::IllFormatted),
		};

		// Parse the protocol.
		let protocol = if let Some(r#match) = captures.name("protocol") {
			Some(match r#match.as_str() {
				"::" => Protocol::DecNet,

				_ => match captures.name("protocol").unwrap().as_str() {
					"tcp" => Protocol::Tcp,
					"inet" => Protocol::Inet,
					"inet6" => Protocol::Inet6,

					"unix" => Protocol::Unix,

					_ => return Err(DisplayNameParseError::UnrecognizedProtocol),
				},
			})
		} else {
			None
		};

		let hostname = captures
			.name("hostname")
			.map(|r#match| r#match.as_str().to_owned());

		// Parse the display number.
		let display_num = match captures
			.name("display_num")
			.unwrap()
			.as_str()
			.parse::<isize>()
		{
			Ok(display_num) => display_num,

			Err(_) => return Err(DisplayNameParseError::IllFormatted),
		};
		// Parse the screen number.
		let screen_num = if let Some(r#match) = captures.name("screen_num") {
			Some(match r#match.as_str().parse::<isize>() {
				Ok(screen_num) => screen_num,

				Err(_) => return Err(DisplayNameParseError::IllFormatted),
			})
		} else {
			None
		};

		Ok(Self {
			protocol,
			hostname,
			display_num,
			screen_num,
		})
	}
}

#[non_exhaustive]
pub enum Protocol {
	/// A connection is established over DECnet.
	DecNet,

	/// A connection is established over TCP.
	Tcp,
	/// A connection is established over TCP using IPv4.
	Inet,
	/// A connection is established over TCP using IPv6.
	Inet6,

	/// A connection is established over unix domain sockets (a form of inter-process
	/// communication, a.k.a. IPC).
	Unix,
}

impl Protocol {
	fn protocol_type(&self) -> ProtocolType {
		match self {
			Self::DecNet => ProtocolType::DecNet,

			Self::Tcp => ProtocolType::Tcp,
			Self::Inet => ProtocolType::Tcp,
			Self::Inet6 => ProtocolType::Tcp,

			Self::Unix => ProtocolType::Unix,
		}
	}
}

enum ProtocolType {
	DecNet,
	Tcp,
	Unix,
}

pub struct AuthInfo {
	pub protocol_name: String,
	pub protocol_data: String,
}

pub enum ConnectionError {
	ConnectionFailure,
	AuthenticationError,
}

pub struct Timeout;

impl Client {
	pub async fn send<R: Request>(&self, _request: R) -> Result<R::Reply, Timeout> {
		todo!()
	}
}
