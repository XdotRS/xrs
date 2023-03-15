// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::Client;
use bytes::Bytes;

pub(crate) enum X11Frame {
	RequestHeader {
		major_opcode: u8,
		metabyte: u8,
		length: u16,
	},
	ReplyHeader {
		metabyte: u8,
		sequence: u16,
		length: u32,
	},
	EventHeader {
		code: u8,
		metabyte: u8,
		sequence: u16,
	},
	ErrorHeader {
		code: u8,
		sequence: u16,
		metablock: [u8; 4],
		minor_opcode: u16,
		major_opcode: u8,
	},

	ReplyEventChunk {
		chunk: [u8; 24],
	},
	ErrorChunk {
		chunk: [u8; 21],
	},
	Chunk {
		chunk: Bytes,
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

			if self.stream.read_buf(&mut self.buffer).await? {
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
