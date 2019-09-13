use crate::error::Result;
use crate::fsm::error::FsmError;
use crate::pdu::{AssociateACContainer, AssociateRJContainer, AssociateRQContainer, PDataContainer, AbortRQSource};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    /// State01 Idle
    State01,
    // State02 Transport connection open (Awaiting A-ASSOCIATE-RQ PDU)
    State02,
    /// State03 - Awaiting local A-ASSOCIATE response primitive (from local user)
    State03,
    /// Awaiting transport connection opening to complete (from local transport service)
    State04,
    /// Awaiting A-ASSOCIATE-AC or A-ASSOCIATE-RJ PDU
    State05,
    /// Association established and ready for data transfer
    State06,
    /// Awaiting A-RELEASE-RP PDU
    State07,
    /// Awaiting local A-RELEASE response primitive (from local user)
    State08,
    /// Release collision requestor side; awaiting A-RELEASE response (from local user)
    State09,
    /// Release collision acceptor side; awaiting A-RELEASE-RP PDU
    State10,
    /// Release collision requestor side; awaiting A-RELEASE-RP PDU
    State11,
    /// Release collision acceptor side; awaiting A-RELEASE response primitive (from local user)
    State12,
    /// Awaiting Transport Connection Close Indication (Association no longer exists
    State13,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AssociationExaminationResponse {
    Accept,
    Reject,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AssociationRole {
    Requester,
    Acceptor,
}

pub trait Actions {
    fn ae_1_transport_connect(&mut self) -> Result<()>;
    fn ae_2_send_association_rq_pdu(&mut self) -> Result<()>;
    fn ae_3_association_confirmation_ac(&mut self) -> Result<()>;
    fn ae_4_associate_confirmation_rj(&mut self) -> Result<()>;
    fn ae_5_transport_connect_response(&mut self) -> Result<()>;
    fn ae_6_examine_associate_rq(
        &mut self,
        association_rq: Arc<AssociateRQContainer>,
    ) -> Result<AssociationExaminationResponse>;
    fn ae_7_send_association_ac(&mut self, association_ac: Arc<AssociateACContainer>)
        -> Result<()>;
    fn ae_8_send_association_rj(&mut self, association_rj: Arc<AssociateRJContainer>)
        -> Result<()>;
    fn dt_1_send_pdata(&mut self, pdata: Arc<PDataContainer>) -> Result<()>;
    fn dt_2_indicate_pdata(&mut self, pdata: Arc<PDataContainer>) -> Result<()>;
    fn aa_1_send_association_abort(&mut self) -> Result<()>;
    fn aa_2_close_transport(&mut self) -> Result<()>;
    fn aa_3_indicate_peer_aborted(&mut self) -> Result<()>;
    fn aa_4_indicate_ap_abort(&mut self) -> Result<()>;
    fn aa_5_stop_artim_timer(&mut self) -> Result<()>;
    fn aa_6_ignore_pdu(&mut self) -> Result<()>;
    fn aa_7_send_abort(&mut self) -> Result<()>;
    fn aa_8_unrecognized_pdu_send_abort(&mut self) -> Result<()>;
    fn ar_1_send_release_rq(&mut self) -> Result<()>;
    fn ar_2_indicate_release(&mut self) -> Result<()>;
    fn ar_3_confirm_release(&mut self) -> Result<()>;
    fn ar_4_send_release_rp(&mut self) -> Result<()>;
    fn ar_5_stop_artim_timer(&mut self) -> Result<()>;
    fn ar_6_indicate_pdata(&mut self) -> Result<()>;
    fn ar_7_send_pdata(&mut self) -> Result<()>;
    fn ar_8_indicate_association_release(&mut self) -> Result<()>;
    fn ar_9_send_association_release_rp(&mut self) -> Result<()>;
    fn ar_10_confirm_release(&mut self) -> Result<()>;
}

#[derive(Debug)]
pub enum Event {
    /// A-ASSOCIATE Request (local user)
    AAssociateReqLocalUser,
    /// Transport Conn. Confirm (local transport service)
    TransConnConfirmLocalUser,
    /// A-ASSOCIATE-AC PDU (received on transport connection)
    AAssociateAcPduRcv,
    /// A-ASSOCIATE-RJ PDU (received on transport connection)
    AAssociateRjPduRcv,
    /// Transport Connection Indication (local transport service)
    TransConnIndication,
    /// A-ASSOCIATE-RQ PDU (received on transport connection)
    AAssociateRqPduRcv(Arc<AssociateRQContainer>),
    /// A-ASSOCIATE response primitive (accept)
    AAssociateResponseAccept(Arc<AssociateACContainer>),
    /// A-ASSOCIATE response primitive (reject)
    AAssociateResponseReject(Arc<AssociateRJContainer>),
    /// P-DATA request primitive
    PDataReq(Arc<PDataContainer>),
    /// P-DATA-TF PDU
    PDataTfPduRcv(Arc<PDataContainer>),
    /// A-RELEASE Request primitive
    AReleaseReq,
    /// A-RELEASE-RQ PDU (received on open transport connection)
    AReleaseRQPduRcv,
    /// A-RELEASE-RP PDU (received on transport connection)
    AReleaseRpPduRcv,
    /// A-RELEASE Response primitive
    AReleaseResp,
    /// A-ABORT Request primitive
    AAbortReq,
    /// A-ABORT PDU (received on open transport connection)
    AAbortPduRcv,
    /// Transport connection closed indication (local transport service)
    TransConnClosed,
    /// ARTIM timer expired (Association reject/release timer)
    ArtimTimerExpired,
    /// Unrecognized or invalid PDU received
    InvalidPdu,
}

#[derive(Debug, Copy, Clone)]
pub struct StateMachine {
    state: State,
    role: AssociationRole,
}

impl StateMachine {
    pub fn new(role: AssociationRole) -> StateMachine {
        StateMachine {
            state: State::State01,
            role,
        }
    }
    pub fn get_state(&self) -> State {
        self.state
    }
    pub fn process_event<T: Actions>(&mut self, event: Event, actions: &mut T) -> Result<()> {
        match &event {
            Event::AAssociateReqLocalUser => match self.state {
                State::State01 => {
                    self.state = State::State04;
                    actions.ae_1_transport_connect()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::TransConnConfirmLocalUser => match self.state {
                State::State04 => {
                    self.state = State::State05;
                    actions.ae_2_send_association_rq_pdu()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAssociateAcPduRcv => match self.state {
                State::State02 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State05 => {
                    self.state = State::State06;
                    actions.ae_3_association_confirmation_ac()?;
                    Ok(())
                }
                State::State13 => {
                    actions.aa_6_ignore_pdu()?;
                    Ok(())
                }
                State::State03
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAssociateRjPduRcv => match self.state {
                State::State02 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State05 => {
                    self.state = State::State01;
                    actions.ae_4_associate_confirmation_rj()?;
                    Ok(())
                }
                State::State13 => {
                    actions.aa_6_ignore_pdu()?;
                    Ok(())
                }
                State::State03
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::TransConnIndication => match self.state {
                State::State01 => {
                    self.state = State::State02;
                    actions.ae_5_transport_connect_response()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAssociateRqPduRcv(data) => match self.state {
                State::State02 => {
                    match actions.ae_6_examine_associate_rq(data.clone())? {
                        AssociationExaminationResponse::Accept => {
                            self.state = State::State03;
                        }
                        AssociationExaminationResponse::Reject => {
                            self.state = State::State13;
                        }
                    }
                    Ok(())
                }
                State::State13 => {
                    self.state = State::State13;
                    actions.aa_7_send_abort()?;
                    Ok(())
                }
                State::State03
                | State::State05
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAssociateResponseAccept(data) => match self.state {
                State::State03 => {
                    self.state = State::State06;
                    actions.ae_7_send_association_ac(data.clone())?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAssociateResponseReject(data) => match self.state {
                State::State03 => {
                    self.state = State::State13;
                    actions.ae_8_send_association_rj(data.clone())?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::PDataReq(data) => match self.state {
                State::State06 => {
                    self.state = State::State06;
                    actions.dt_1_send_pdata(data.clone())?;
                    Ok(())
                }
                State::State08 => {
                    self.state = State::State08;
                    actions.ar_7_send_pdata()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::PDataTfPduRcv(data) => match self.state {
                State::State02 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State03
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                State::State05 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                State::State06 => {
                    self.state = State::State06;
                    actions.dt_2_indicate_pdata(data.clone())?;
                    Ok(())
                }
                State::State07 => {
                    self.state = State::State07;
                    actions.ar_6_indicate_pdata()?;
                    Ok(())
                }
                State::State13 => {
                    self.state = State::State13;
                    actions.aa_6_ignore_pdu()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AReleaseReq => match self.state {
                State::State06 => {
                    self.state = State::State07;
                    actions.ar_1_send_release_rq()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AReleaseRQPduRcv => match self.state {
                State::State02 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State03
                | State::State05
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                State::State06 => {
                    self.state = State::State08;
                    actions.ar_2_indicate_release()?;
                    Ok(())
                }
                State::State07 => {
                    match self.role {
                        AssociationRole::Requester => {
                            self.state = State::State09;
                        }
                        AssociationRole::Acceptor => {
                            self.state = State::State10;
                        }
                    }
                    actions.ar_8_indicate_association_release()?;
                    Ok(())
                }
                State::State13 => {
                    self.state = State::State13;
                    actions.aa_6_ignore_pdu()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AReleaseRpPduRcv => match self.state {
                State::State02 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State03
                | State::State05
                | State::State06
                | State::State08
                | State::State09
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                State::State07 => {
                    self.state = State::State01;
                    actions.ar_3_confirm_release()?;
                    Ok(())
                }
                State::State10 => {
                    self.state = State::State12;
                    actions.ar_10_confirm_release()?;
                    Ok(())
                }
                State::State11 => {
                    self.state = State::State01;
                    actions.ar_3_confirm_release()?;
                    Ok(())
                }
                State::State13 => {
                    self.state = State::State13;
                    actions.aa_6_ignore_pdu()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AReleaseResp => match self.state {
                State::State08 | State::State12 => {
                    self.state = State::State13;
                    actions.ar_4_send_release_rp()?;
                    Ok(())
                }
                State::State09 => {
                    self.state = State::State11;
                    actions.ar_9_send_association_release_rp()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAbortReq => match self.state {
                State::State03
                | State::State05
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State04 => {
                    self.state = State::State01;
                    actions.aa_2_close_transport()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::AAbortPduRcv => match self.state {
                State::State02 | State::State13 => {
                    self.state = State::State01;
                    actions.aa_2_close_transport()?;
                    Ok(())
                }
                State::State03
                | State::State05
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State01;
                    actions.aa_3_indicate_peer_aborted()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::TransConnClosed => match self.state {
                State::State02 => {
                    self.state = State::State01;
                    actions.aa_5_stop_artim_timer()?;
                    Ok(())
                }
                State::State03
                | State::State04
                | State::State05
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State01;
                    actions.aa_4_indicate_ap_abort()?;
                    Ok(())
                }
                State::State13 => {
                    self.state = State::State01;
                    actions.ar_5_stop_artim_timer()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::ArtimTimerExpired => match self.state {
                State::State02 | State::State13 => {
                    self.state = State::State01;
                    actions.aa_2_close_transport()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
            Event::InvalidPdu => match self.state {
                State::State02 => {
                    self.state = State::State13;
                    actions.aa_1_send_association_abort()?;
                    Ok(())
                }
                State::State03
                | State::State05
                | State::State06
                | State::State07
                | State::State08
                | State::State09
                | State::State10
                | State::State11
                | State::State12 => {
                    self.state = State::State13;
                    actions.aa_8_unrecognized_pdu_send_abort()?;
                    Ok(())
                }
                State::State13 => {
                    self.state = State::State13;
                    actions.aa_7_send_abort()?;
                    Ok(())
                }
                state => Err(FsmError::InvalidEventForState { state, event })?,
            },
        }
    }
}

pub mod error;

#[cfg(test)]
mod test;
