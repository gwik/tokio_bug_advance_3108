use futures::channel::mpsc;
use futures::channel::mpsc::SendError;
use futures::{Sink, SinkExt, StreamExt};
use tokio::time;

pub struct UnboundedSender {
    sender: mpsc::UnboundedSender<i32>,
}

impl UnboundedSender {
    pub fn new(mut showdown_sender: impl Sink<i32> + Send + Unpin + 'static) -> Self {
        let (tx, mut rx) = mpsc::unbounded();
        tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                if showdown_sender.send(message).await.is_err() {
                    return;
                }
                time::sleep(time::Duration::from_millis(700)).await;
            }
        });
        Self { sender: tx }
    }

    pub async fn send(&self, message: i32) -> Result<(), SendError> {
        (&self.sender).send(message).await
    }
}

#[cfg(test)]
mod tests {

    use super::UnboundedSender;
    use futures::{channel::mpsc, StreamExt};

    #[tokio::test]
    async fn sender_does_not_delay_on_first_message_1() -> Result<(), Box<dyn std::error::Error>> {
        use tokio::time::{self, Instant};
        time::pause();
        let (tx, mut rx) = mpsc::unbounded();
        let sender = UnboundedSender::new(tx);
        let now = Instant::now();
        sender.send(42).await?;
        assert_eq!(rx.next().await, Some(42));
        assert_eq!(now, Instant::now());
        Ok(())
    }

    #[tokio::test]
    async fn sender_does_not_delay_on_first_message_2() -> Result<(), Box<dyn std::error::Error>> {
        use tokio::task;
        use tokio::time::{self, Duration, Instant};
        time::pause();
        let (tx, mut rx) = mpsc::unbounded();
        let sender = UnboundedSender::new(tx);
        let now = Instant::now();
        // First element gets send instantly
        sender.send(42).await?;
        task::yield_now().await;
        assert_eq!(rx.try_next()?, Some(42));
        assert_eq!(now, Instant::now());
        // Second element will require waiting
        sender.send(24).await?;
        task::yield_now().await;
        assert!(rx.try_next().is_err());
        // Actually waiting
        time::advance(Duration::from_millis(701)).await;
        task::yield_now().await;
        assert_eq!(rx.try_next()?, Some(24));
        Ok(())
    }
}
