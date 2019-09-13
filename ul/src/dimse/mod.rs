use crate::error::{Error, Result};
use dicom_core::DataDictionary;
use dicom_object::mem::InMemDicomObject;

//const C_STORE_RQ: u16 = 0x0001;
//const C_STORE_RSP: u16 = 0x8001;
//const C_GET_RQ: u16 = 0x0010;
//const C_GET_RSP: u16 = 0x8010;
//const C_FIND_RQ: u16 = 0x0020;
//const C_FIND_RSP: u16 = 0x8020;
//const C_MOVE_RQ: u16 = 0x0021;
//const C_MOVE_RSP: u16 = 0x8021;
const C_ECHO_RQ: u16 = 0x0030;
//const C_ECHO_RSP: u16 = 0x8030;
//const N_EVENT_REPORT_RQ: u16 = 0x0100;
//const N_EVENT_REPORT_RSP: u16 = 0x8100;
//const N_GET_RQ: u16 = 0x0110;
//const N_GET_RSP: u16 = 0x8110;
//const N_SET_RQ: u16 = 0x0120;
//const N_SET_RSP: u16 = 0x8120;
//const N_ACTION_RQ: u16 = 0x0130;
//const N_ACTION_RSP: u16 = 0x8130;
//const N_CREATE_RQ: u16 = 0x0140;
//const N_CREATE_RSP: u16 = 0x8140;
//const N_DELETE_RQ: u16 = 0x0150;
//const N_DELETE_RSP: u16 = 0x8150;
//const C_CANCEL_RQ: u16 = 0x0FFF;

#[derive(PartialEq)]
pub enum CommandDataSetType {
    Present,
    NoPresent
}

/// The status of the C-ECHO request.
pub enum EchoStatus {
    /// Success (0000H)
    Success,
    /// Refused: SOP Class Not Supported (0122H) - Indicates that a different SOP Class than the Verification SOP Class was specified, which was not supported.
    Refused,
    /// Duplicate Invocation (0210H) - Indicates that the Message ID (0000,0110) specified is allocated to another notification or operation.
    DuplicateInvocation,
    /// Mistyped argument (0212H) - Indicates that one of the parameters supplied has not been agreed for use on the Association between the DIMSE Service Users.
    MistypedArgument,
    /// Unrecognized Operation (0211H) - Indicates that a different SOP Class than the Verification SOP Class was specified, which does not recognize a C-ECHO operation.
    UnrecognizedOperation
}

impl EchoStatus {
    fn from(value: u16) -> Result<EchoStatus> {
        match value {
            0x0000 => {
                Ok(EchoStatus::Success)
            }
            0x0122 => {
                Ok(EchoStatus::Refused)
            }
            0x0210 => {
                Ok(EchoStatus::DuplicateInvocation)
            }
            0x0212 => {
                Ok(EchoStatus::MistypedArgument)
            }
            0x0211 => {
                Ok(EchoStatus::UnrecognizedOperation)
            }
            _ => {
                Err(Error::InvalidCommandData)
            }
        }
    }
}

pub struct EchoRqContainer {
    pub message_id: u16,
    pub affected_sop_class_uid: String
}

pub enum Command {
    EchoRq(EchoRqContainer),
}

impl Command {
    pub fn from_command_data_set<D>(obj: &InMemDicomObject<D>) -> Result<Command>
    where
        D: DataDictionary,
        D: Clone,
    {
        let command_field = obj.element_by_name("CommandField")?.value().as_u16()?.first().ok_or(Error::InvalidCommandData)?.to_owned();

        let command_data_set_type;
        match obj.element_by_name("CommandDataSetType")?.value().as_u16()?.first().ok_or(Error::InvalidCommandData)?.to_owned() {
            0x0101 => {
                command_data_set_type = CommandDataSetType::Present;
            }
            _ => {
                command_data_set_type = CommandDataSetType::NoPresent;
            }
        }

        match command_field {
            C_ECHO_RQ => {
                if command_data_set_type != CommandDataSetType::NoPresent {
                    return Err(Error::InvalidCommandData);
                }
                Ok(Command::EchoRq(EchoRqContainer {
                    message_id: obj.element_by_name("MessageID")?.value().as_u16()?.first().ok_or(Error::InvalidCommandData)?.to_owned(),
                    affected_sop_class_uid: obj.element_by_name("AffectedSOPClassUID")?.value().to_str()?.to_string()
                }))
            }
            _ => {
                Err(Error::BadCommandType)
            }
        }
    }

    pub fn to_command_data_set<D>(&self) -> Result<InMemDicomObject<D>>
        where
            D: DataDictionary,
            D: Clone,
    {
        Err(Error::InvalidCommandData)
    }
}
