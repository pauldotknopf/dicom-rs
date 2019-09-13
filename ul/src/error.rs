use quick_error::quick_error;
/// Type alias for a result from this crate.
pub type Result<T> = ::std::result::Result<T, crate::error::Error>;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: std::io::Error) {
            from()
        }
        FromUtf8(err: std::string::FromUtf8Error) {
            from()
        }
        DicomObjectError(err: dicom_object::Error) {
            from()
            display("dicom object error: {}", err)
        }
        DicomCoreError(err: dicom_core::error::Error) {
            from()
            display("dicom core error: {}", err)
        }
        /// A failed attempt to cast a value to an inappropriate format.
        CastValue(err: dicom_core::error::CastValueError) {
            from()
            display("value cast error: {}", err)
        }
        Generic(err: String) {
            from()
            from(err: &str) -> (err.to_string())
            display("{}", err)
        }
        NoPDUAvailable {

        }
        InvalidMaxPDU {

        }
        PDUTooLarge {

        }
        FsmError(err: crate::fsm::error::FsmError) {
            from()
            display("fsm error: {}", err)
        }
        InvalidPDU {

        }
        InconsistentState {

        }
        PeerAbortedAssociation {

        }
        InvalidPData {

        }
        InvalidPresentationContextId {

        }
        UnexpectedPdvType {

        }
        BadCommandType {

        }
        InvalidCommandData {

        }
    }
}
