use std::rc::Rc;
use std::sync::RwLock;
use std::pin::Pin;
use bytes::BytesMut;
use core::task::Poll;
use futures::task::Context;
use futures::{Stream};
use tokio::io::{ReadHalf, AsyncRead};
use tokio::net::TcpStream;
use crate::net::ConnectionType;
use crate::net::pipeline::HandlerPipeline;
use crate::net::pipeline::framing::FrameCodec;

const MIN_BUFFER_SIZE: usize = 256;

/// A Stream of decoded incoming packets.
///
/// It utilizes the processors configured
/// in the pipeline to decode the stream.
pub struct PipelineStream<C: ConnectionType> {
    r: Pin<Box<ReadHalf<TcpStream>>>,
    pipeline: Rc<RwLock<HandlerPipeline<C>>>,

    /// A buffer that temporary stores not yet completely
    /// received packet frames until they are complete.
    ///
    /// To workaround issues of the ownership system,
    /// the buffer gets moved out of this structs and
    /// is then returned.
    /// Constructs such as RefCell or Mutex cannot solve
    /// this issue as they keep a non-mutable reference
    /// to self which prevents mutable uses of self.
    read_buf: Option<BytesMut>,
}

impl<C: ConnectionType> PipelineStream<C> {
    pub fn new(r: ReadHalf<TcpStream>, pipeline: Rc<RwLock<HandlerPipeline<C>>>) -> PipelineStream<C> {
        PipelineStream {
            r: Box::pin(r),
            read_buf: Some(BytesMut::with_capacity(MIN_BUFFER_SIZE)),
            pipeline,
        }
    }

    fn borrow_buf(&mut self) -> &mut BytesMut {
        self.read_buf.as_mut().unwrap()
    }

    /// Take the ownership of the read_buf.
    ///
    /// Note that it must always get returned afterward.
    fn take_buf(&mut self) -> BytesMut {
        self.read_buf.take().unwrap_or_else(||
            BytesMut::with_capacity(MIN_BUFFER_SIZE))
    }

    fn return_buf(&mut self, buffer: BytesMut) {
        self.read_buf = Some(buffer);
    }

    fn try_read(&mut self, pipeline: &HandlerPipeline<C>) -> Poll<Option<Result<C::In, ()>>> {
        let mut buffer = self.borrow_buf();
        if let Some(mut frame) = FrameCodec::try_decode(&mut buffer)? {
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
        let pipeline = pipeline.write().unwrap();

        // Try to read remaining frames on the buffer
        if let Poll::Ready(Some(read_data)) = self.try_read(&pipeline) {
            return Poll::Ready(Some(read_data))
        }

        // Else try to load more data into the buffer
        let mut buffer = self.take_buf();
        buffer.reserve(MIN_BUFFER_SIZE);
        let poll_result = self.r.as_mut().poll_read_buf(cx, &mut buffer);
        self.return_buf(buffer);

        match poll_result {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(_)) => Poll::Ready(Some(Err(()))),
            Poll::Ready(Ok(0)) => Poll::Ready(None), // End of stream reached
            Poll::Ready(Ok(_)) => self.try_read(&pipeline),
        }
    }
}
