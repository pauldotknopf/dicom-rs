use crate::dimse::Command;
use crate::error::{Error, Result};
use crate::fsm::error::FsmError;
use crate::fsm::Event;
use crate::fsm::{Actions, AssociationExaminationResponse, AssociationRole, State, StateMachine};
use crate::pdu::reader::{read_pdu, DEFAULT_MAX_PDU};
use crate::pdu::writer::write_pdu;
use crate::pdu::{AssociateACContainer, AssociateRJContainer, AssociateRQContainer, PDataContainer, PDataValueType, PresentationContextProposed, PresentationContextResult, PresentationContextResultReason, PDU, AbortRQContainer, AbortRQSource, AbortRQServiceProviderReason, PDataValue};
use crate::NetStream;
use dicom_core::Length;
use dicom_encoding::text::SpecificCharacterSet;
use dicom_object::mem::InMemDicomObject;
use dicom_object::StandardDataDictionary;
use dicom_parser::DataSetReader;
use dicom_transfer_syntax_registry::get_registry;
use std::io::Write;
use std::net::Shutdown;
use std::sync::Arc;
use crate::pdu::AbortRQSource::ServiceUser;
use std::panic::resume_unwind;
use byteordered::byteorder::ReadBytesExt;
use dicom_encoding::Encode;

/// The entire data-set that may have expanded multiple PDATA/PDVs.
pub struct PDataDataSet {
    pub presentation_context: AcceptedContext,
    pub value_type: PDataValueType,
    pub data_set: InMemDicomObject<StandardDataDictionary>,
}

pub struct AssociationCommand {
    pub presentation_context: AcceptedContext,
    pub command: Command
}

/// Supported abstract syntax, along with the supported transfer syntaxes.
/// The association will be negotiated with these.
pub struct SupportedContext {
    /// The abstract syntax
    pub abstract_syntax: String,
    /// The list of transfer syntaxes for this abstract syntax.
    pub transfer_syntaxes: Vec<String>,
    /// Does the SCU or SCP have priority over which transfer syntax is chosed?
    pub scp_priority: bool,
}

#[derive(Clone)]
pub struct AcceptedContext {
    pub id: u8,
    pub abstract_syntax: String,
    pub transfer_syntax: String,
}

pub struct AssociationOptions {
    pub max_pdu_size: u32,
    pub supported_contexts: Vec<SupportedContext>,
}

impl Default for AssociationOptions {
    fn default() -> AssociationOptions {
        AssociationOptions {
            max_pdu_size: DEFAULT_MAX_PDU,
            supported_contexts: vec![],
        }
    }
}

impl PresentationContextProposed {
    fn accept(&self, supported_contexts: &[SupportedContext]) -> PresentationContextResult {
        for supported_context in supported_contexts {
            if supported_context.abstract_syntax == self.abstract_syntax {
                if supported_context.scp_priority {
                    for scp_transfer_syntax in &supported_context.transfer_syntaxes {
                        for scu_transfer_syntax in &self.transfer_syntaxes {
                            if scu_transfer_syntax == scp_transfer_syntax {
                                return PresentationContextResult {
                                    id: self.id,
                                    reason: PresentationContextResultReason::Acceptance,
                                    transfer_syntax: scu_transfer_syntax.clone(),
                                };
                            }
                        }
                    }
                } else {
                    for scu_transfer_syntax in &self.transfer_syntaxes {
                        for scp_transfer_syntax in &supported_context.transfer_syntaxes {
                            if scu_transfer_syntax == scp_transfer_syntax {
                                return PresentationContextResult {
                                    id: self.id,
                                    reason: PresentationContextResultReason::Acceptance,
                                    transfer_syntax: scu_transfer_syntax.clone(),
                                };
                            }
                        }
                    }
                }
                return PresentationContextResult {
                    id: self.id,
                    reason: PresentationContextResultReason::TransferSyntaxesNotSupported,
                    transfer_syntax: "".to_string(),
                };
            }
        }
        PresentationContextResult {
            id: self.id,
            reason: PresentationContextResultReason::AbstractSyntaxNotSupported,
            transfer_syntax: "".to_string(),
        }
    }
}

impl AssociationOptions {
    pub fn add_supported_context(
        &mut self,
        abstract_syntax: String,
        transfer_syntaxes: Vec<String>,
        scp_priority: bool,
    ) -> Result<()> {
        // TODO: validate
        self.supported_contexts.push(SupportedContext {
            abstract_syntax,
            transfer_syntaxes,
            scp_priority,
        });
        Ok(())
    }
}

pub struct Association<'a> {
    stream: &'a mut NetStream,
    state_machine: StateMachine,
    association_rq: Option<Arc<AssociateRQContainer>>,
    accepted_contexts: Vec<AcceptedContext>,
    current_pdata: Option<Arc<PDataContainer>>,
    options: AssociationOptions,
}

impl<'a> Association<'a> {
    fn process_event(&mut self, event: Event) -> Result<()> {
        let mut state_machine = self.state_machine;
        state_machine.process_event(event, self)?;
        self.state_machine = state_machine;
        Ok(())
    }
}

impl<'a> Actions for Association<'a> {
    fn ae_1_transport_connect(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ae_2_send_association_rq_pdu(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ae_3_association_confirmation_ac(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ae_4_associate_confirmation_rj(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ae_5_transport_connect_response(&mut self) -> Result<()> {
        // Start the timer?
        Ok(())
    }

    fn ae_6_examine_associate_rq(
        &mut self,
        association_rq: Arc<AssociateRQContainer>,
    ) -> Result<AssociationExaminationResponse> {
        self.association_rq = Some(association_rq);
        Ok(AssociationExaminationResponse::Accept)
    }

    fn ae_7_send_association_ac(
        &mut self,
        association_ac: Arc<AssociateACContainer>,
    ) -> Result<()> {
        write_pdu(self.stream, &PDU::AssociationAC(association_ac))?;
        Ok(())
    }

    fn ae_8_send_association_rj(
        &mut self,
        association_rj: Arc<AssociateRJContainer>,
    ) -> Result<()> {
        write_pdu(self.stream, &PDU::AssociationRJ(association_rj))?;
        Ok(())
    }

    fn dt_1_send_pdata(&mut self, _pdata: Arc<PDataContainer>) -> Result<()> {
        unimplemented!()
    }

    fn dt_2_indicate_pdata(&mut self, pdata: Arc<PDataContainer>) -> Result<()> {
        self.current_pdata = Some(pdata);
        Ok(())
    }

    fn aa_1_send_association_abort(&mut self) -> Result<()> {
        write_pdu(self.stream, &PDU::AbortRQ(Arc::new(AbortRQContainer{
            source: AbortRQSource::ServiceUser
        })))
    }

    fn aa_2_close_transport(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn aa_3_indicate_peer_aborted(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn aa_4_indicate_ap_abort(&mut self) -> Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        Err(Error::PeerAbortedAssociation)
    }

    fn aa_5_stop_artim_timer(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn aa_6_ignore_pdu(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn aa_7_send_abort(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn aa_8_unrecognized_pdu_send_abort(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_1_send_release_rq(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_2_indicate_release(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_3_confirm_release(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_4_send_release_rp(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_5_stop_artim_timer(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_6_indicate_pdata(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_7_send_pdata(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_8_indicate_association_release(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_9_send_association_release_rp(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn ar_10_confirm_release(&mut self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> Association<'a> {
    pub fn receive_association(
        stream: &'a mut NetStream,
        options: AssociationOptions,
    ) -> Result<Association> {
        // TODO: validate PDU

        let mut association = Association {
            stream,
            state_machine: StateMachine::new(AssociationRole::Acceptor),
            association_rq: None,
            accepted_contexts: vec![],
            current_pdata: None,
            options,
        };

        association.process_event(Event::TransConnIndication)?;
        assert_eq!(association.state_machine.get_state(), State::State02);

        let pdu_result = read_pdu(association.stream, DEFAULT_MAX_PDU);

        if let Ok(pdu) = pdu_result {
            match pdu {
                PDU::AssociationAC { .. } => {
                    association.process_event(Event::AAssociateAcPduRcv)?;
                }
                PDU::AssociationRJ { .. } => {
                    association.process_event(Event::AAssociateRjPduRcv)?;
                }
                PDU::AssociationRQ(data) => {
                    association.process_event(Event::AAssociateRqPduRcv(data))?;
                }
                PDU::PData(data) => {
                    association.process_event(Event::PDataTfPduRcv(data))?;
                }
                PDU::ReleaseRQ { .. } => {
                    association.process_event(Event::AReleaseRQPduRcv)?;
                }
                PDU::ReleaseRP { .. } => {
                    association.process_event(Event::AReleaseRpPduRcv)?;
                }
                PDU::AbortRQ { .. } => {
                    association.process_event(Event::AAbortPduRcv)?;
                }
                PDU::Unknown { .. } => {
                    association.process_event(Event::InvalidPdu)?;
                }
            }
        } else if let Err(e) = pdu_result {
            // TODO: Check if network timed out, issue Event::ArtimTimerExpired.
            if let Error::NoPDUAvailable = e {
                association.process_event(Event::TransConnClosed)?;
            }
            Err(e)?
        }

        // State01: Idle. Occurs when the incoming connection was abruptly closed, or network timed out.
        // State03: Waiting for an association indication (accept/reject). This is the normal condition.
        // State13: Waiting for connection to close.
        if association.state_machine.get_state() != State::State03 {
            return Err(FsmError::UnexpectedState(
                association.state_machine.get_state(),
            ))?;
        }

        let association_rq = association
            .association_rq
            .as_ref()
            .ok_or_else(|| crate::error::Error::InconsistentState)?;

        let mut accepted = vec![];
        for proposed_context in &association_rq.presentation_contexts {
            // //association_rq.presentation_contexts.iter().map(|p| p.accept(&association.options.supported_contexts)).collect();
            let result = proposed_context.accept(&association.options.supported_contexts);
            if result.reason == PresentationContextResultReason::Acceptance {
                association.accepted_contexts.push(AcceptedContext {
                    id: proposed_context.id,
                    abstract_syntax: proposed_context.abstract_syntax.clone(),
                    transfer_syntax: result.transfer_syntax.clone(),
                })
            }
            accepted.push(result);
        }

        association.process_event(Event::AAssociateResponseAccept(Arc::new(
            AssociateACContainer {
                protocol_version: 1,
                application_context_name: "".to_string(),
                presentation_contexts: accepted,
                user_variables: vec![],
            },
        )))?;

        Ok(association)
    }

    fn read_pdata(&mut self) -> Result<Arc<PDataContainer>> {
        let pdu_result = read_pdu(self.stream, DEFAULT_MAX_PDU);
        if let Ok(pdu) = pdu_result {
            match pdu {
                PDU::AssociationAC { .. } => {
                    self.process_event(Event::AAssociateAcPduRcv)?;
                }
                PDU::AssociationRJ { .. } => {
                    self.process_event(Event::AAssociateRjPduRcv)?;
                }
                PDU::AssociationRQ(data) => {
                    self.process_event(Event::AAssociateRqPduRcv(data))?;
                }
                PDU::PData(data) => {
                    self.process_event(Event::PDataTfPduRcv(data))?;
                }
                PDU::ReleaseRQ { .. } => {
                    self.process_event(Event::AReleaseRQPduRcv)?;
                }
                PDU::ReleaseRP { .. } => {
                    self.process_event(Event::AReleaseRpPduRcv)?;
                }
                PDU::AbortRQ { .. } => {
                    self.process_event(Event::AAbortPduRcv)?;
                }
                PDU::Unknown { .. } => {
                    self.process_event(Event::InvalidPdu)?;
                }
            }
        } else if let Err(e) = pdu_result {
            // TODO: Check if network timed out, issue Event::ArtimTimerExpired.
            if let Error::NoPDUAvailable = e {
                self.process_event(Event::TransConnClosed)?;
            }
            Err(e)?
        }

        let pdata = self
            .current_pdata
            .as_ref()
            .map(|e| e.clone())
            .ok_or_else(|| crate::error::Error::InconsistentState)?;
        self.current_pdata = None;

        Ok(pdata.clone())
    }

    pub fn receive_data_set(
        &mut self,
        expected_type: Option<PDataValueType>,
    ) -> Result<PDataDataSet> {
        let mut bytes = vec![];
        let mut value_type: Option<PDataValueType> = None;
        let mut presentation_context_id: Option<u8> = None;

        let mut is_last = false;
        loop {
            let pdata = self.read_pdata()?;
            for pdv in &pdata.data {
                // We are iterating another item, after we have already been indicated that
                // we processed the "last" element?
                if is_last {
                    return Err(Error::InvalidPData);
                }

                if let Some(expected_type) = expected_type {
                    if pdv.value_type != expected_type {
                        return Err(Error::UnexpectedPdvType);
                    }
                }

                Write::write_all(&mut bytes, &pdv.data)?;

                // Make sure these values don't change for the duration.
                if let Some(d) = value_type {
                    if d != pdv.value_type {
                        return Err(Error::InvalidPData);
                    }
                } else {
                    value_type = Some(pdv.value_type);
                }
                if let Some(d) = presentation_context_id {
                    if d != pdv.presentation_context_id {
                        return Err(Error::InvalidPData);
                    }
                } else {
                    presentation_context_id = Some(pdv.presentation_context_id);
                }

                is_last = pdv.is_last;
                if is_last {
                    break;
                }
            }
            if is_last {
                break;
            }
        }

        let value_type = value_type.ok_or(Error::InvalidPData)?;
        let presentation_context_id = presentation_context_id.ok_or(Error::InvalidPData)?;

        // Find the transfer syntax in the registry.
        let presentation_context = self
            .accepted_contexts
            .iter()
            .find(|c| c.id == presentation_context_id)
            .ok_or(Error::InvalidPresentationContextId)?;

        let ts = get_registry()
            .get(&presentation_context.transfer_syntax)
            .ok_or(dicom_object::Error::UnsupportedTransferSyntax)?;
        let mut data_set_reader = DataSetReader::new_with_dictionary(
            bytes.as_slice(),
            StandardDataDictionary.clone(),
            ts,
            SpecificCharacterSet::Default,
        )?;

        Ok(PDataDataSet {
            presentation_context: presentation_context.clone(),
            value_type: value_type,
            data_set: InMemDicomObject::build_object(
                &mut data_set_reader,
                StandardDataDictionary,
                false,
                Length::new(bytes.len() as u32),
            )?,
        })
    }

    pub fn send_dimse_command(&mut self, command: AssociationCommand) -> Result<()> {

        //let data_set = command.command.to_command_data_set()?;

//
//
//        self.process_event(Event::PDataReq(Arc::new(PDataContainer{
//            data: vec![
//                PDataValue{
//                    presentation_context_id: command.presentation_context.id,
//                    value_type: PDataValueType::Command,
//                    is_last: true,
//                    data:
//
//
////                    pub presentation_context_id: u8,
////                    pub value_type: PDataValueType,
////                    pub is_last: bool,
////                    pub data: Vec<u8>,
//                }
//            ]
//        })));
        Ok(())
    }

    pub fn read_dimse_command(&mut self) -> Result<AssociationCommand> {
        let result = self.receive_data_set(Some(PDataValueType::Command))?;
        Ok(AssociationCommand {
            presentation_context: result.presentation_context,
            command: Command::from_command_data_set(&result.data_set)?
        })
    }

    pub fn abort_association(&mut self) -> Result<()> {
        self.process_event(Event::AAbortReq)
    }
}

#[cfg(test)]
mod test {
    use crate::asso::SupportedContext;
    use crate::pdu::{PresentationContextProposed, PresentationContextResultReason};

    #[test]
    fn can_negotiate_abstract_syntax() {
        let proposed_presentation_context = PresentationContextProposed {
            id: 1,
            abstract_syntax: "abstract 1".to_string(),
            transfer_syntaxes: vec!["transfer 1".to_string(), "transfer 2".to_string()],
        };
        let supported_presentation_contexts = vec![SupportedContext {
            abstract_syntax: "abstract 1".to_string(),
            transfer_syntaxes: vec!["transfer 2".to_string(), "transfer 1".to_string()],
            scp_priority: false,
        }];
        let accepted_presentation_context =
            proposed_presentation_context.accept(&supported_presentation_contexts);

        assert_eq!(accepted_presentation_context.id, 1);
        assert_eq!(
            accepted_presentation_context.reason,
            PresentationContextResultReason::Acceptance
        );
        assert_eq!(accepted_presentation_context.transfer_syntax, "transfer 1");

        let supported_presentation_contexts = vec![SupportedContext {
            abstract_syntax: "abstract 1".to_string(),
            transfer_syntaxes: vec!["transfer 2".to_string(), "transfer 1".to_string()],
            scp_priority: true,
        }];
        let accepted_presentation_context =
            proposed_presentation_context.accept(&supported_presentation_contexts);

        assert_eq!(accepted_presentation_context.id, 1);
        assert_eq!(
            accepted_presentation_context.reason,
            PresentationContextResultReason::Acceptance
        );
        assert_eq!(accepted_presentation_context.transfer_syntax, "transfer 2");
    }

    #[test]
    fn can_indicate_abstract_syntax_not_supported() {
        let proposed_presentation_context = PresentationContextProposed {
            id: 1,
            abstract_syntax: "abstract 1".to_string(),
            transfer_syntaxes: vec!["transfer 1".to_string(), "transfer 2".to_string()],
        };
        let supported_presentation_contexts = vec![SupportedContext {
            abstract_syntax: "abstract 2".to_string(),
            transfer_syntaxes: vec!["transfer 2".to_string(), "transfer 1".to_string()],
            scp_priority: false,
        }];
        let accepted_presentation_context =
            proposed_presentation_context.accept(&supported_presentation_contexts);

        assert_eq!(accepted_presentation_context.id, 1);
        assert_eq!(
            accepted_presentation_context.reason,
            PresentationContextResultReason::AbstractSyntaxNotSupported
        );
        assert_eq!(accepted_presentation_context.transfer_syntax, "");
    }

    #[test]
    fn can_indicate_transfer_syntax_not_supported() {
        let proposed_presentation_context = PresentationContextProposed {
            id: 1,
            abstract_syntax: "abstract 1".to_string(),
            transfer_syntaxes: vec!["transfer 1".to_string(), "transfer 2".to_string()],
        };
        let supported_presentation_contexts = vec![SupportedContext {
            abstract_syntax: "abstract 1".to_string(),
            transfer_syntaxes: vec!["transfer 3".to_string(), "transfer 4".to_string()],
            scp_priority: false,
        }];
        let accepted_presentation_context =
            proposed_presentation_context.accept(&supported_presentation_contexts);

        assert_eq!(accepted_presentation_context.id, 1);
        assert_eq!(
            accepted_presentation_context.reason,
            PresentationContextResultReason::TransferSyntaxesNotSupported
        );
        assert_eq!(accepted_presentation_context.transfer_syntax, "");
    }
}
