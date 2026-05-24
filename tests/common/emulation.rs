use futures::{channel::mpsc, executor::block_on, StreamExt};
use iced::{Program, Size, Task};
use iced_test::{
    emulator::{Event, Mode},
    runtime::{task::into_stream, Action},
    Emulator,
};
use nameless_music_player_lib::app::{App, Message};

/// Emulator meant to simulate user interactions, do not use in test with `tokio::test` macro.
pub struct EmulatorSession<P: Program + 'static> {
    program: P,
    emulator: Emulator<P>,
    receiver: mpsc::Receiver<Event<P>>,
}

impl<P: Program + 'static> EmulatorSession<P> {
    /// Boots up the emulator session, use the app function in `src/app.rs` to create the program.
    pub fn boot(program: P, viewport: impl Into<Size>) -> Self {
        let (sender, receiver) = mpsc::channel(64);
        let emulator = Emulator::new(sender, &program, Mode::Zen, viewport.into());

        let mut session = Self {
            program,
            emulator,
            receiver,
        };

        session.run_pending_task_to_completion();

        session
    }

    // TODO: Fix this so it dispatches user interactions instead.
    /// Dispatches a App Message and drives any subsequent tasks to completion
    pub fn dispatch(&mut self, message: P::Message) {
        self.emulator.update(&self.program, message);

        self.run_pending_task_to_completion();
    }

    /// This consumes the emulator, no commands can be issued afterwards.
    pub fn into_state(self) -> P::State {
        self.emulator.into_state().0
    }

    fn run_pending_task_to_completion(&mut self) {
        loop {
            let event =
                block_on(self.receiver.recv()).expect("Emulator runtime stopped unexpectedly");

            match event {
                Event::Action(action) => self.emulator.perform(&self.program, action),
                Event::Ready => break,
                Event::Failed(instruction) => {
                    panic!("Emulator instruction failed: {:?}", instruction)
                }
            }
        }
    }
}

pub async fn perform_task(app: &mut App, task: Task<Message>) {
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
