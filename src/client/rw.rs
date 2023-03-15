// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::Client;
use bytes::Bytes;

pub(crate) enum X11Frame {
	/// Note: requests are never read by X.RS; requests are received by X
	/// servers, while X.RS is a client library. The only way to tell apart
	/// requests vs. replies & events is whether they are received by an X
	/// client or whether they are received by the X server.
	Request {
		/// The major opcode identifying the type of request (for core requests)
		/// or the extension that defines the request (for extension requests).
		major_opcode: u8,
		/// A single byte in the header which may be used for additional data.
		metabyte: u8,
		/// The length of the request in blocks (units of 4 bytes).
		///
		/// If the big-requests extension is enabled and this is `0`, then the
		/// next block is the length of the request instead, but as a `u32`
		/// value.
		length: u16,

		/// Additional data contained in the request, if any.
		chunk: Bytes,
	},

	Reply {
		/// A single byte in the header which may be used for additional data.
		metabyte: u8,
		/// The sequence number which identifies when the request that generated
		/// the reply was sent.
		sequence: u16,
		/// The length of any additional data after the first 8 blocks (32
		/// bytes) in the reply in blocks (units of 4 bytes).
		length: u32,

		/// Data contained in the reply.
		chunk: Bytes,
	},

	Event {
		/// The code uniquely identifying the type of event.
		code: u8,
		/// Data contained in the event.
		chunk: [u8; 31],
	},

	Error {
		/// The code uniquely identifying the type of error.
		code: u8,
		/// The sequence number which identifies when the last request sent that
		/// relates to the generation of this error was sent.
		sequence: u16,
		/// A single block (4 bytes) in the header which may be used for
		/// additional data.
		metablock: [u8; 4],

		/// The minor opcode identifying the type of the last request sent that
		/// relates to the generation of this error within its extension (`0`
		/// for core requests).
		minor_opcode: u16,
		/// The major opcode identifying the type of (for core requests) or the
		/// extension which defines (for extension requests) the last request
		/// sent that relates to the generation of this error.
		major_opcode: u8,

		/// Additional data contained in the error.
		chunk: [u8; 21],
	},
}

impl Client {
	pub(crate) async fn read_frame(&mut self) -> Result<Option<X11Frame>, !> {
		/// The end of the stream is reached when there are 0 bytes remaining.
		const END_OF_STREAM: usize = 0;

		loop {
			if let Some(frame) = self.parse_frame()? {
				return Ok(Some(frame));
			}

			if self.stream.read_buf(&mut self.buffer).await? == END_OF_STREAM {
				if self.buffer.is_empty() {
					return Ok(None);
				} else {
					return Err("connection reset by peer".into());
				}
			}
		}
	}

	pub(crate) async fn write_frame(&mut self, frame: &X11Frame) -> Result<(), !> {
		todo!()
	}
}
