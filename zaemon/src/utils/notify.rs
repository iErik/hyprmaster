use tokio::sync::mpsc::UnboundedSender;

use notify_debouncer_full::{
  DebouncedEvent,
  DebounceEventResult,
  DebounceEventHandler
};



pub struct DebouncedSender(
  pub UnboundedSender<DebouncedEvent>
);


impl DebounceEventHandler for DebouncedSender {
  fn handle_event(&mut self, ev: DebounceEventResult) {
    match ev {
      Ok(evs) => {
        for event in evs {
          _ = self.0.send(event)
            .expect("Sender not poisoned");
        }
      },
      Err(error) => {
        println!("AsyncSender Error: {:#?}", error);
      }
    }
  }
}

