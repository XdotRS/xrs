// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use self::Error::Incomplete;
use crate::Client;
use bytes::{Buf, Bytes};
use std::io::Cursor;
use tokio::{
	io,
	io::{AsyncReadExt, AsyncWriteExt},
};

pub(crate) enum X11Frame {
	/// <table>
	///     <tbody>
	///         <tr>
	///             <td><b>Byte</b></td>
	///             <td><b>Meaning</b></td>
	///         </tr>
	///         <tr>
	///             <td>0</td>
	///             <td><code>major_opcode</code></td>
	///         </tr>
	///         <tr>
	///             <td>1</td>
	///             <td><code>metabyte</code></td>
	///         </tr>
	///         <tr>
	///             <td>2</td>
	///             <td rowspan="2"><code>length</code></td>
	///         </tr>
	///         <tr>
	///             <td>3</td>
	///         </tr>
	///         <tr>
	///             <td>4..(4 + (length * 4))</td>
	///             <td><code>chunk</code></td>
	///         </tr>
	///     </tbody>
	/// </table>
	///
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

	/// <table>
	///     <tbody>
	///         <tr>
	///             <td><b>Byte</b></td>
	///             <td><b>Meaning</b></td>
	///         </tr>
	///         <tr>
	///             <td>0</td>
	///             <td><code>1</code> - reply indicator</td>
	///         </tr>
	///         <tr>
	///             <td>1</td>
	///             <td><code>metabyte</code></td>
	///         </tr>
	///         <tr>
	///             <td>2</td>
	///             <td rowspan="2"><code>sequence</code></td>
	///         </tr>
	///         <tr>
	///             <td>3</td>
	///         </tr>
	///         <tr>
	///             <td>4</td>
	///             <td rowspan="4"><code>length</code></td>
	///         </tr>
	///         <tr>
	///             <td>5</td>
	///         </tr>
	///         <tr>
	///             <td>6</td>
	///         </tr>
	///         <tr>
	///             <td>7</td>
	///         </tr>
	///         <tr>
	///             <td>8..(32 + (length * 4))</td>
	///             <td><code>chunk</code></td>
	///         </tr>
	///     </tbody>
	/// </table>
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

	/// <table>
	///     <tbody>
	///         <tr>
	///             <td><b>Byte</b></td>
	///             <td><b>Meaning</b></td>
	///         </tr>
	///         <tr>
	///             <td>0</td>
	///             <td><code>code</code></td>
	///         </tr>
	///         <tr>
	///             <td>1..32</td>
	///             <td><code>chunk</code></td>
	///         </tr>
	///     </tbody>
	/// </table>
	Event {
		/// The code uniquely identifying the type of event.
		code: u8,
		/// Data contained in the event.
		chunk: [u8; 31],
	},

	/// <table>
	///     <tbody>
	///         <tr>
	///             <td><b>Byte</b></td>
	///             <td><b>Meaning</b></td>
	///         </tr>
	///         <tr>
	///             <td>0</td>
	///             <td><code>0</code> - error indicator</td>
	///         </tr>
	///         <tr>
	///             <td>1</td>
	///             <td><code>code</code></td>
	///         </tr>
	///         <tr>
	///             <td>2</td>
	///             <td rowspan="2"><code>sequence</code></td>
	///         </tr>
	///         <tr>
	///             <td>3</td>
	///         </tr>
	///         <tr>
	///             <td>4</td>
	///             <td rowspan="4"><code>metablock</code></td>
	///         </tr>
	///         <tr>
	///             <td>5</td>
	///         </tr>
	///         <tr>
	///             <td>6</td>
	///         </tr>
	///         <tr>
	///             <td>7</td>
	///         </tr>
	///         <tr>
	///             <td>8</td>
	///             <td rowspan="2"><code>minor_opcode</code></td>
	///         </tr>
	///         <tr>
	///             <td>9</td>
	///         </tr>
	///         <tr>
	///             <td>10</td>
	///             <td><code>major_opcode</code></td>
	///         </tr>
	///         <tr>
	///             <td>11..32</td>
	///             <td><code>chunk</code></td>
	///         </tr>
	///     </tbody>
	/// </table>
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

impl X11Frame {
	// https://tokio.rs/tokio/tutorial/framing
	pub(crate) fn check(buf: &mut Cursor<&[u8]>) -> Result<(), Error> {
		const BLOCK: usize = 4;
		const REPLY_BODY: usize = 32 - (2 * BLOCK); // 24

		match get_u8(buf)? {
			// Reply
			1 => {
				// Skip the rest of the first block; it can't be invalid at this
				// level of abstraction.
				skip(buf, BLOCK - 1)?;
				let len = get_u32(buf)? as usize;

				// Skip the reply's data, verifying that enough bytes are
				// present in the process.
				skip(buf, REPLY_BODY + (len * BLOCK))
			},

			// Error or event
			// Errors and events are always 32 bytes; none can be invalid at
			// this level of abstraction, so we skip all of them, verifying that
			// there are enough in the process.
			0 | _ => skip(buf, 32 - 1),
		}
	}

	// https://tokio.rs/tokio/tutorial/framing
	pub(crate) fn parse(buf: &mut Cursor<&[u8]>) -> Result<Self, Error> {
		const BLOCK: usize = 4;

		const MESSAGE_BASE: usize = 32;
		const ERROR_BODY: usize = MESSAGE_BASE - 7;
		const REPLY_BODY: usize = MESSAGE_BASE - (2 * BLOCK);

		match get_u8(buf)? {
			// Error
			0 => {
				let error_code = get_u8(buf)?;
				let sequence = get_u16(buf)?;
				let metablock = [get_u8(buf)?, get_u8(buf)?, get_u8(buf)?, get_u8(buf)?];

				let minor_opcode = get_u16(buf)?;
				let major_opcode = get_u8(buf)?;

				let chunk = buf.chunk()[..ERROR_BODY].try_into().unwrap();
				skip(buf, ERROR_BODY)?;

				Ok(Self::Error {
					code: error_code,
					sequence,
					metablock,

					minor_opcode,
					major_opcode,

					chunk,
				})
			},

			// Reply
			1 => {
				let metabyte = get_u8(buf)?;
				let sequence = get_u16(buf)?;
				let length = get_u32(buf)?;

				let bytes = REPLY_BODY + ((length as usize) * BLOCK);
				let chunk = Bytes::copy_from_slice(&buf.chunk()[..bytes]);
				skip(buf, bytes)?;

				Ok(Self::Reply {
					metabyte,
					sequence,
					length,

					chunk,
				})
			},

			// Event
			event_code => {
				let bytes = MESSAGE_BASE - 1;
				let chunk = buf.chunk()[..bytes].try_into().unwrap();
				skip(buf, bytes)?;

				Ok(Self::Event {
					code: event_code,
					chunk,
				})
			},
		}
	}
}

impl Client {
	// https://tokio.rs/tokio/tutorial/framing
	fn parse_frame(&mut self) -> Result<Option<X11Frame>, Error> {
		let mut buf = Cursor::new(&self.buffer[..]);

		match X11Frame::check(&mut buf) {
			Ok(_) => {
				let length = buf.position() as usize;

				buf.set_position(0);

				let frame = X11Frame::parse(&mut buf)?;

				self.buffer.advance(length);

				Ok(Some(frame))
			},

			Err(Incomplete) => Ok(None),
			Err(other) => Err(other),
		}
	}

	// https://tokio.rs/tokio/tutorial/framing
	pub(crate) async fn read_frame(&mut self) -> Result<Option<X11Frame>, Error> {
		/// The end of the stream is reached when there are 0 bytes remaining.
		const END_OF_STREAM: usize = 0;

		loop {
			if let Some(frame) = self.parse_frame()? {
				return Ok(Some(frame));
			}

			if self.stream.read_buf(&mut self.buffer).await? == END_OF_STREAM {
				return if self.buffer.is_empty() {
					Ok(None)
				} else {
					Err("connection reset by peer".into())
				};
			}
		}
	}

	// https://tokio.rs/tokio/tutorial/framing
	pub(crate) async fn write_frame(&mut self, frame: &X11Frame) -> io::Result<()> {
		match frame {
			X11Frame::Request {
				major_opcode,
				metabyte,
				length,
				chunk,
			} => {
				self.stream.write_u8(*major_opcode).await?;
				self.stream.write_u8(*metabyte).await?;
				self.stream.write_u16(*length).await?;
				self.stream.write(&*chunk).await?;
			},

			X11Frame::Reply {
				metabyte,
				sequence,
				length,
				chunk,
			} => {
				self.stream.write_u8(1).await?;
				self.stream.write_u8(*metabyte).await?;
				self.stream.write_u16(*sequence).await?;
				self.stream.write_u32(*length).await?;
				self.stream.write(&*chunk).await?;
			},

			X11Frame::Event { code, chunk } => {
				self.stream.write_u8(*code).await?;
				self.stream.write(&*chunk).await?;
			},

			X11Frame::Error {
				code,
				sequence,
				metablock,
				minor_opcode,
				major_opcode,
				chunk,
			} => {
				self.stream.write_u8(0).await?;
				self.stream.write_u8(*code).await?;
				self.stream.write_u16(*sequence).await?;
				self.stream.write(&*metablock).await?;
				self.stream.write_u16(*minor_opcode).await?;
				self.stream.write_u8(*major_opcode).await?;
				self.stream.write(&*chunk).await?;
			},
		}

		self.stream.flush().await?;

		Ok(())
	}
}

enum Error {
	Incomplete,
	Other,
}

fn peek_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, Error> {
	if !buf.has_remaining() {
		return Err(Incomplete);
	}

	Ok(buf.chunk()[0])
}

fn get_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, Error> {
	if !buf.has_remaining() {
		return Err(Incomplete);
	}

	Ok(buf.get_u8())
}

fn peek_u16(buf: &mut Cursor<&[u8]>) -> Result<u16, Error> {
	if buf.remaining() < 2 {
		return Err(Incomplete);
	}

	let chunk = buf.chunk();
	Ok(u16::from_be_bytes([chunk[0], chunk[1]]))
}

fn get_u16(buf: &mut Cursor<&[u8]>) -> Result<u16, Error> {
	if buf.remaining() < 2 {
		return Err(Incomplete);
	}

	Ok(buf.get_u16())
}

fn peek_u32(buf: &mut Cursor<&[u8]>) -> Result<u32, Error> {
	if buf.remaining() < 4 {
		return Err(Incomplete);
	}

	let chunk = buf.chunk();
	Ok(u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
}

fn get_u32(buf: &mut Cursor<&[u8]>) -> Result<u32, Error> {
	if buf.remaining() < 4 {
		return Err(Incomplete);
	}

	Ok(buf.get_u32())
}

fn skip(buf: &mut Cursor<&[u8]>, count: usize) -> Result<(), Error> {
	if buf.remaining() < count {
		return Err(Incomplete);
	}

	buf.advance(count);
	Ok(())
}
