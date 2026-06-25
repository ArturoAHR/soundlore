use futures::StreamExt;
use iced::Task;
use iced_test::runtime::{Action, task::into_stream};
use soundlore_lib::{
    app::{App, Event},
    message::Message,
};

pub async fn perform_task(app: &mut App, task: Task<Message<Event>>) {
    let mut pending_tasks = vec![task];

    while let Some(task) = pending_tasks.pop() {
        let Some(mut stream) = into_stream(task) else {
            continue;
        };

        while let Some(action) = stream.next().await {
            if let Action::Output(message) = action {
                pending_tasks.push(app.update(message));
            }
        }
    }
}
