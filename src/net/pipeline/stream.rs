use std::rc::Rc;
use std::sync::RwLock;
use std::pin::Pin;
use bytes::BytesMut;
use core::task::Poll;
use futures::task::Context;
use futures::{Stream};
use tokio::io::AsyncRead;
use crate::net::ConnectionType;
use crate::net::pipeline::HandlerPipeline;
use crate::net::pipeline::framing::FrameCodec;

const MIN_BUFFER_SIZE: usize = 256;

/// A Stream of decoded incoming packets.
///
/// It utilizes the processors configured
/// in the pipeline to decode the stream.
pub struct PipelineStream<C: ConnectionType> {
    pipeline: Rc<RwLock<HandlerPipeline<C>>>,

    /// A buffer that temporary stores not yet completely
    /// received packet frames until they are complete.
    read_buf: BytesMut,
}

impl<C: ConnectionType> PipelineStream<C> {
    pub fn new(pipeline: Rc<RwLock<HandlerPipeline<C>>>) -> PipelineStream<C> {
        PipelineStream {
            read_buf: BytesMut::with_capacity(MIN_BUFFER_SIZE),
            pipeline,
        }
    }

    fn try_read(&mut self, pipeline: &HandlerPipeline<C>) -> Poll<Option<Result<C::In, ()>>> {
        if let Some(mut frame) = FrameCodec::try_decode(&mut self.read_buf)? {
            if let Some(compressor) = &pipeline.compressor {
                frame = compressor.decode(frame)?;
            }

            Poll::Ready(Some(Ok(pipeline.codec.decode(&mut frame)?)))
        } else {
            Poll::Pending
        }
    }
}

impl<C: ConnectionType> Stream for PipelineStream<C> {
    type Item = Result<C::In, ()>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pipeline = self.pipeline.clone();
        let mut pipeline = pipeline.write().unwrap();

        // Try to read remaining frames on the buffer
        if let Poll::Ready(Some(read_data)) = self.try_read(&pipeline) {
            return Poll::Ready(Some(read_data))
        }

        // Else try to load more data into the buffer
        self.read_buf.reserve(MIN_BUFFER_SIZE);
        match pipeline.r.as_mut().poll_read_buf(cx, &mut self.read_buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(_)) => Poll::Ready(Some(Err(()))),
            Poll::Ready(Ok(0)) => Poll::Ready(None), // End of stream reached
            Poll::Ready(Ok(_)) => self.try_read(&pipeline),
        }
    }
}
