use crate::{Indicator, Next};
use core::pin::Pin;
use core::task::Poll;
use futures_core::stream::Stream;

pub struct IndicatorStream<Inner, InputStream>
where
    Inner: Indicator + Next<InputStream::Item>,
    InputStream: Stream,
{
    inner: Inner,
    input_stream: InputStream,
}

impl<Inner, InputStream> IndicatorStream<Inner, InputStream>
where
    Inner: Indicator + Next<InputStream::Item>,
    InputStream: Stream,
{
    pub(crate) fn new(inner: Inner, input_stream: InputStream) -> Self {
        Self {
            inner,
            input_stream,
        }
    }

    pub fn decompose(self) -> Inner {
        self.inner
    }

    unsafe fn pin_input_stream(&self) -> Pin<&mut InputStream> {
        Pin::new_unchecked(&mut *(&self.input_stream as *const _ as *mut _))
    }
    unsafe fn inner_mut(&self) -> &mut Inner {
        &mut *(&self.inner as *const _ as *mut _)
    }
}

impl<Inner, InputStream> Stream for IndicatorStream<Inner, InputStream>
where
    Inner: Indicator + Next<InputStream::Item>,
    InputStream: Stream + Unpin,
{
    type Item = Inner::Output;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let input_stream = unsafe { self.pin_input_stream() };

        match input_stream.poll_next(cx) {
            Poll::Ready(Some(input)) => {
                let inner = unsafe { self.inner_mut() };
                Poll::Ready(inner.next(input).into())
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use crate::Sma;
    use futures_executor::LocalPool;
    use futures_util::task::SpawnExt;
    use futures_util::{stream, StreamExt};

    #[test]
    fn test() -> core::result::Result<(), Box<dyn std::error::Error>> {
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        let input_stream = stream::iter(RANDOM_DATA.iter());

        let fut = async move {
            let mut sma = Sma::new(4).unwrap();
            let mut stream = IndicatorStream::new(sma.clone(), input_stream);

            for input in RANDOM_DATA.iter() {
                let correct = sma.next(input);
                assert_eq!(stream.next().await.unwrap(), correct);
            }

            assert_eq!(stream.next().await, None);
        };

        spawner.spawn(fut)?;

        pool.run();

        Ok(())
    }
}
