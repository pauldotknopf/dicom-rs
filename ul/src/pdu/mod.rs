use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PresentationContextProposed {
    pub id: u8,
    pub abstract_syntax: String,
    pub transfer_syntaxes: Vec<String>,
}

#[derive(Debug)]
pub struct PresentationContextResult {
    pub id: u8,
    pub reason: PresentationContextResultReason,
    pub transfer_syntax: String,
}

#[derive(Debug, PartialEq)]
pub enum PresentationContextResultReason {
    Acceptance = 0,
    UserRejection = 1,
    NoReason = 2,
    AbstractSyntaxNotSupported = 3,
    TransferSyntaxesNotSupported = 4,
}

#[derive(Debug)]
pub enum AssociationRJResult {
    Permanent,
    Transient,
}

#[derive(Debug)]
pub enum AssociationRJSource {
    ServiceUser(AssociationRJServiceUserReason),
    ServiceProviderASCE(AssociationRJServiceProviderASCEReason),
    ServiceProviderPresentation(AssociationRJServiceProviderPresentationReason),
}

#[derive(Debug)]
pub enum AssociationRJServiceUserReason {
    NoReasonGiven,
    ApplicationContextNameNotSupported,
    CallingAETitleNotRecognized,
    CalledAETitleNotRecognized,
    Reserved(u8),
}

#[derive(Debug)]
pub enum AssociationRJServiceProviderASCEReason {
    NoReasonGiven,
    ProtocolVersionNotSupported,
}

#[derive(Debug)]
pub enum AssociationRJServiceProviderPresentationReason {
    TemporaryCongestion,
    LocalLimitExceeded,
    Reserved(u8),
}

#[derive(Debug)]
pub struct PDataValue {
    pub presentation_context_id: u8,
    pub value_type: PDataValueType,
    pub is_last: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PDataValueType {
    Command,
    Data,
}

#[derive(Debug)]
pub enum AbortRQSource {
    ServiceUser,
    ServiceProvider(AbortRQServiceProviderReason),
    Reserved,
}

#[derive(Debug)]
pub enum AbortRQServiceProviderReason {
    ReasonNotSpecifiedUnrecognizedPDU,
    UnexpectedPDU,
    Reserved,
    UnrecognizedPDUParameter,
    UnexpectedPDUParameter,
    InvalidPDUParameter,
}

#[derive(Debug)]
pub enum PDUVariableItem {
    Unknown(u8),
    ApplicationContext(String),
    PresentationContextProposed(PresentationContextProposed),
    PresentationContextResult(PresentationContextResult),
    UserVariables(Vec<UserVariableItem>),
}

#[derive(Debug, Clone)]
pub enum UserVariableItem {
    Unknown(u8),
    MaxLength(u32),
    ImplementationClassUID(String),
    ImplementationVersionName(String),
}

#[derive(Debug)]
pub struct UnknownContainer {
    pdu_type: u8,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct AssociateRQContainer {
    pub protocol_version: u16,
    pub calling_ae_title: String,
    pub called_ae_title: String,
    pub application_context_name: String,
    pub presentation_contexts: Vec<PresentationContextProposed>,
    pub user_variables: Vec<UserVariableItem>,
}

impl Clone for AssociateRQContainer {
    fn clone(&self) -> Self {
        AssociateRQContainer {
            protocol_version: self.protocol_version,
            calling_ae_title: self.calling_ae_title.clone(),
            called_ae_title: self.called_ae_title.clone(),
            application_context_name: self.application_context_name.clone(),
            presentation_contexts: self.presentation_contexts.clone(),
            user_variables: self.user_variables.clone(),
        }
    }
}

#[derive(Debug)]
pub struct AssociateACContainer {
    pub protocol_version: u16,
    pub application_context_name: String,
    pub presentation_contexts: Vec<PresentationContextResult>,
    pub user_variables: Vec<UserVariableItem>,
}

#[derive(Debug)]
pub struct AssociateRJContainer {
    result: AssociationRJResult,
    source: AssociationRJSource,
}

#[derive(Debug)]
pub struct PDataContainer {
    pub data: Vec<PDataValue>,
}

#[derive(Debug)]
pub struct AbortRQContainer {
    pub source: AbortRQSource,
}

#[derive(Debug)]
pub enum PDU {
    Unknown(Arc<UnknownContainer>),
    AssociationRQ(Arc<AssociateRQContainer>),
    AssociationAC(Arc<AssociateACContainer>),
    AssociationRJ(Arc<AssociateRJContainer>),
    PData(Arc<PDataContainer>),
    ReleaseRQ,
    ReleaseRP,
    AbortRQ(Arc<AbortRQContainer>),
}

pub mod reader;
pub mod writer;

#[cfg(test)]
mod test;
