use crate::fsm::State;
use crate::fsm::{AssociationRole, StateMachine};

#[test]
fn can_new_state_machine() {
    let state_machine = StateMachine::new(AssociationRole::Acceptor);
    assert_eq!(state_machine.state, State::State01);
}
/*mod can_change_from_state_1 {
    use crate::fsm::{StateMachine, Event, State, AssociationRole};

    fn get_state_machine() -> StateMachine {
        StateMachine::new(AssociationRole::Acceptor)
    }

    #[test]
    fn with_associate_request() {
        let mut state_machine = get_state_machine();
        let mut actions = MockActions::new();
        actions.expect_ae_1_transport_connect()
            .returning(|| Ok(()));

        state_machine.process_event(Event::AAssociateReqLocalUser, &mut actions).expect("invalid event");

        assert_eq!(state_machine.state, State::State04);
    }

    #[test]
    fn with_trans_con_indication() {
        let mut state_machine = get_state_machine();
        let mut actions = MockActions::new();
        actions.expect_ae_5_transport_connect_response()
            .returning(|| Ok(()));

        state_machine.process_event(Event::TransConnIndication, &mut actions).expect("invalid event");

        assert_eq!(state_machine.state, State::State02);
    }
}*/
