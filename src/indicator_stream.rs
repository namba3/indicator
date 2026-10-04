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
    ) -> Poll<Option<Self::Item>> {
        // SAFETY: `input_stream` is `Unpin`, and this implementation does not
        // move the pinned `IndicatorStream` value or project a pin to `inner`.
        let this = unsafe { self.get_unchecked_mut() };

        match Pin::new(&mut this.input_stream).poll_next(cx) {
            Poll::Ready(Some(input)) => Poll::Ready(this.inner.next(input).into()),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sma;
    use crate::test_helper::*;
    use futures_executor::LocalPool;
    use futures_util::task::SpawnExt;
    use futures_util::{StreamExt, stream};

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
