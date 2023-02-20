// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use async_std::{
	io,
	io::{Read, Write},
	net::TcpStream,
	os::unix::net::UnixStream,
};
use std::{
	io::{IoSlice, IoSliceMut},
	pin::Pin,
	task::{Context, Poll},
};

pub enum Stream {
	TcpStream(TcpStream),
	#[cfg(unix)]
	UnixStream(UnixStream),
}

impl Read for Stream {
	fn poll_read(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8],
	) -> Poll<io::Result<usize>> {
		match self {
			Self::TcpStream(stream) => stream.poll_read(cx, buf),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_read(cx, buf),
		}
	}

	fn poll_read_vectored(
		self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [IoSliceMut<'_>],
	) -> Poll<io::Result<usize>> {
		match self {
			Self::TcpStream(stream) => stream.poll_read_vectored(cx, bufs),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_read_vectored(cx, bufs),
		}
	}
}

impl Read for &Stream {
	fn poll_read(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8],
	) -> Poll<io::Result<usize>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_read(Pin::new(&mut &*stream), cx, buf),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_read(Pin::new(&mut &*stream), cx, buf),
		}
	}

	fn poll_read_vectored(
		self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [IoSliceMut<'_>],
	) -> Poll<io::Result<usize>> {
		match self {
			Stream::TcpStream(stream) => {
				<&TcpStream>::poll_read_vectored(Pin::new(&mut &*stream), cx, bufs)
			},
			#[cfg(unix)]
			Stream::UnixStream(stream) => {
				<&UnixStream>::poll_read_vectored(Pin::new(&mut &*stream), cx, bufs)
			},
		}
	}
}

impl Write for Stream {
	fn poll_write(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8],
	) -> Poll<io::Result<usize>> {
		match self {
			Self::TcpStream(stream) => stream.poll_write(cx, buf),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_write(cx, buf),
		}
	}

	fn poll_write_vectored(
		self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>],
	) -> Poll<io::Result<usize>> {
		match self {
			Self::TcpStream(stream) => stream.poll_write_vectored(cx, bufs),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_write_vectored(cx, bufs),
		}
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Self::TcpStream(stream) => stream.poll_flush(cx),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_flush(cx),
		}
	}

	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Self::TcpStream(stream) => stream.poll_close(cx),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_close(cx),
		}
	}
}

impl Write for &Stream {
	fn poll_write(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8],
	) -> Poll<io::Result<usize>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_write(Pin::new(&mut &*stream), cx, buf),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_write(Pin::new(&mut &*stream), cx, buf),
		}
	}

	fn poll_write_vectored(
		self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>],
	) -> Poll<io::Result<usize>> {
		match self {
			Stream::TcpStream(stream) => {
				<&TcpStream>::poll_write_vectored(Pin::new(&mut &*stream), cx, bufs)
			},
			#[cfg(unix)]
			Stream::UnixStream(stream) => {
				<&UnixStream>::poll_write_vectored(Pin::new(&mut &*stream), cx, bufs)
			},
		}
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_flush(Pin::new(&mut &*stream), cx),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_flush(Pin::new(&mut &*stream), cx),
		}
	}

	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_close(Pin::new(&mut &*stream), cx),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_close(Pin::new(&mut &*stream), cx),
		}
	}
}
