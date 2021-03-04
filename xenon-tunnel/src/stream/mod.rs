/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

pub mod snowfall;

use futures::{ready, Sink, Stream};
use std::{
	io::{Cursor, Error as IoError},
	pin::Pin,
	task::{Context, Poll},
};
use tokio::{
	io::{AsyncRead, AsyncWrite, ReadBuf},
	net::TcpStream,
};
use tokio_util::{codec::Framed, io::StreamReader};

pub struct EncryptedTcpStream {
	inner: StreamReader<Framed<TcpStream, snowfall::SnowfallStream>, Cursor<Vec<u8>>>,
}

impl EncryptedTcpStream {
	pub fn new(snowfall: snow::TransportState, stream: TcpStream) -> Self {
		Self {
			inner: StreamReader::new(Framed::new(stream, snowfall::SnowfallStream::new(snowfall))),
		}
	}
}

impl Stream for EncryptedTcpStream {
	type Item = Result<Cursor<Vec<u8>>, IoError>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		Pin::new(self.inner.get_mut()).poll_next(cx)
	}
}

impl AsyncWrite for EncryptedTcpStream {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize, IoError>> {
		let mut inner = Pin::new(self.inner.get_mut());
		ready!(inner.as_mut().poll_ready(cx))?;
		inner.start_send(buf)?;
		Poll::Ready(Ok(buf.len()))
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
		Pin::new(self.inner.get_mut()).poll_flush(cx)
	}

	fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
		Pin::new(self.inner.get_mut()).poll_close(cx)
	}
}

impl AsyncRead for EncryptedTcpStream {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<Result<(), IoError>> {
		Pin::new(&mut self.inner).poll_read(cx, buf)
	}
}
