use crate::emulators::tvm_emulator::{TVMRunGetMethodResponse, TVMSendMsgResponse};
use crate::emulators::tx_emulator::TXEmulationResponse;

pub enum PoolEmulationResponse {
    EmulGetMethod(TVMRunGetMethodResponse),
    EmulSendExtMsg(TVMSendMsgResponse),
    EmulSendIntMsg(TVMSendMsgResponse),
    EmulOrdTx(TXEmulationResponse),
    EmulTickTockTx(TXEmulationResponse),
}
