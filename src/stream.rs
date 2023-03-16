// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
	io::IoSlice,
	pin::Pin,
	task::{Context, Poll},
};
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::{
	io,
	io::{AsyncRead, AsyncWrite, ReadBuf},
	net::TcpStream,
};

pub enum Stream {
	TcpStream(TcpStream),
	#[cfg(unix)]
	UnixStream(UnixStream),
}

impl AsyncRead for Stream {
	fn poll_read(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf,
	) -> Poll<io::Result<usize>> {
		match self {
			Self::TcpStream(stream) => stream.poll_read(cx, buf),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_read(cx, buf),
		}
	}
}

impl AsyncRead for &Stream {
	fn poll_read(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8],
	) -> Poll<io::Result<usize>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_read(Pin::new(&mut &*stream), cx, buf),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_read(Pin::new(&mut &*stream), cx, buf),
		}
	}
}

impl AsyncWrite for Stream {
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

	fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Self::TcpStream(stream) => stream.poll_close(cx),
			#[cfg(unix)]
			Self::UnixStream(stream) => stream.poll_close(cx),
		}
	}
}

impl AsyncWrite for &Stream {
	fn poll_write(
		self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8],
	) -> Poll<io::Result<usize>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_write(Pin::new(&mut &*stream), cx, buf),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_write(Pin::new(&mut &*stream), cx, buf),
		}
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_flush(Pin::new(&mut &*stream), cx),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_flush(Pin::new(&mut &*stream), cx),
		}
	}

	fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		match self {
			Stream::TcpStream(stream) => <&TcpStream>::poll_close(Pin::new(&mut &*stream), cx),
			#[cfg(unix)]
			Stream::UnixStream(stream) => <&UnixStream>::poll_close(Pin::new(&mut &*stream), cx),
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
}
