use crate::fsm::{Event, State};
use quick_error::quick_error;

/// Type alias for a result from this crate.
pub type FsmResult<T> = ::std::result::Result<T, FsmError>;

quick_error! {
    #[derive(Debug)]
    pub enum FsmError {
        StateChangeError {

        }
        NoAction {

        }
        InvalidEventForState {
            state: State,
            event: Event
        } {
            display(self_) -> ("invalid event: {:?} for state: {:?}", event, event)
        }
        UnexpectedState(state: State) {
            from()
            display("unexpected state: {:?}", state)
        }
    }
}
