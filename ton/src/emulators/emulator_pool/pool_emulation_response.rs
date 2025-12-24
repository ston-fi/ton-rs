use crate::emulators::tvm_emulator::{TVMGetMethodResponse, TVMSendMsgResponse};
use crate::emulators::tx_emulator::TXEmulationResponse;
use crate::errors::TonError;

#[derive(Debug)]
pub enum PoolEmulationResponse {
    EmulGetMethod(TVMGetMethodResponse),
    EmulSendExtMsg(TVMSendMsgResponse),
    EmulSendIntMsg(TVMSendMsgResponse),
    EmulOrdTx(TXEmulationResponse),
    EmulTickTockTx(TXEmulationResponse),
}

impl TryFrom<PoolEmulationResponse> for TVMGetMethodResponse {
    type Error = TonError;
    fn try_from(value: PoolEmulationResponse) -> Result<Self, Self::Error> {
        match value {
            PoolEmulationResponse::EmulGetMethod(resp) => Ok(resp),
            _ => Err(TonError::EmulatorUnexpectedResponse(format!("Expected TVMGetMethodResponse, got  {value:?}"))),
        }
    }
}

impl TryFrom<PoolEmulationResponse> for TVMSendMsgResponse {
    type Error = TonError;
    fn try_from(value: PoolEmulationResponse) -> Result<Self, Self::Error> {
        match value {
            PoolEmulationResponse::EmulSendExtMsg(resp) | PoolEmulationResponse::EmulSendIntMsg(resp) => Ok(resp),
            _ => Err(TonError::EmulatorUnexpectedResponse(format!("Expected TVMSendMsgResponse, got  {value:?}"))),
        }
    }
}

impl TryFrom<PoolEmulationResponse> for TXEmulationResponse {
    type Error = TonError;
    fn try_from(value: PoolEmulationResponse) -> Result<Self, Self::Error> {
        match value {
            PoolEmulationResponse::EmulOrdTx(resp) | PoolEmulationResponse::EmulTickTockTx(resp) => Ok(resp),
            _ => Err(TonError::EmulatorUnexpectedResponse(format!("Expected TXEmulationResponse, got  {value:?}"))),
        }
    }
}
