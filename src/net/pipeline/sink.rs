use std::rc::Rc;
use std::sync::{RwLock};
use std::pin::Pin;
use bytes::{BytesMut, Buf, BufMut};
use futures::{Sink, ready};
use futures::task::{Context, Poll};
use tokio::io::AsyncWrite;
use crate::net::pipeline::HandlerPipeline;
use crate::net::ConnectionType;
use crate::net::pipeline::framing::FrameCodec;

/// A Sink that writes incoming packets to the wire,
/// applying all processors defined in the pipeline.
pub struct PipelineSink<C: ConnectionType> {
    pipeline: Rc<RwLock<HandlerPipeline<C>>>,
    buffer: Option<BytesMut>,
}

impl<C: ConnectionType> PipelineSink<C> {
    pub fn new(pipeline: Rc<RwLock<HandlerPipeline<C>>>) -> PipelineSink<C> {
        PipelineSink {
            pipeline,
            buffer: None,
        }
    }

    fn encode(&mut self, packet: C::Out) -> Result<BytesMut, ()> {
        let mut buffer = BytesMut::new();

        let pipeline = self.pipeline.write().unwrap();
        pipeline.codec.encode(&packet, &mut buffer)?;

        if let Some(compressor) = &pipeline.compressor {
            buffer = compressor.encode(buffer)?;
        }

        buffer = FrameCodec::encode(buffer)?;

        Ok(buffer)
    }

    fn flush_pending_buffer(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ()>> {
        if let Some(buffer) = &mut self.buffer {
            while buffer.has_remaining() {
                let mut pipeline = self.pipeline.write().unwrap();
                let written_bytes = ready!(pipeline.w.as_mut().poll_write(cx, buffer)).map_err(|_| ())?;
                buffer.advance(written_bytes);
            }
            self.buffer = None;
        }
        Poll::Ready(Ok(()))
    }
}

impl<C: ConnectionType> Sink<C::Out> for PipelineSink<C> {
    type Error = ();

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.flush_pending_buffer(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: C::Out) -> Result<(), Self::Error> {
        let payload = self.encode(item)?;

        // Temporary workaround, until actix [SinkWrite] calls `poll_ready` properly.
        if let Some(buffer) = &mut self.buffer {
            buffer.reserve(payload.len());
            buffer.put_slice(&payload);
            Ok(())
        } else {
            self.buffer = Some(payload);
            Ok(())
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.flush_pending_buffer(cx))?;

        let mut pipeline = self.pipeline.write().unwrap();
        pipeline.w.as_mut().poll_flush(cx).map_err(|_| ())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.flush_pending_buffer(cx))?;

        let mut pipeline = self.pipeline.write().unwrap();
        pipeline.w.as_mut().poll_shutdown(cx).map_err(|_| ())
    }
}
