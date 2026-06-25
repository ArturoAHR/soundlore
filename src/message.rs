use iced::Task;

use crate::config::MESSAGE_EVENT_TTL;

#[derive(Debug, Clone)]
pub struct Message<Payload: Send + Sync + 'static> {
    pub payload: Payload,
    ttl: i32,
}

impl<Payload: Send + Sync + 'static> Message<Payload> {
    pub fn new(payload: Payload) -> Self {
        Self {
            ttl: MESSAGE_EVENT_TTL,
            payload,
        }
    }

    fn new_with_ttl(payload: Payload, ttl: i32) -> Self {
        Self { ttl, payload }
    }

    pub fn from_payload<'a, A>(f: impl Fn(A) -> Payload + 'a) -> impl Fn(A) -> Self + 'a {
        move |value: A| Self::new(f(value))
    }

    pub fn wrap_payload<'a, A>(f: impl Fn(A) -> Payload + 'a) -> impl Fn(Message<A>) -> Self + 'a
    where
        A: Send + Sync,
    {
        move |message: Message<A>| Message::new_with_ttl(f(message.payload), message.ttl)
    }

    pub fn new_from<T>(&self, payload: T) -> Message<T>
    where
        T: Send + Sync + 'static,
    {
        if self.ttl == 0 {
            panic!("Message TTL exceeded")
        }

        Message {
            ttl: self.ttl - 1,
            payload,
        }
    }

    pub fn task_from<T, A>(
        &self,
        future: impl Future<Output = A> + Send + 'static,
        f: impl FnOnce(A) -> T + Send + 'static,
    ) -> Task<Message<T>>
    where
        T: Send + Sync + 'static,
        A: Send + Sync + 'static,
    {
        let ttl = self.ttl;

        Task::perform(future, move |value: A| {
            Message::new_with_ttl(f(value), ttl - 1)
        })
    }
}
