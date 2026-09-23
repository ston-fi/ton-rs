//! A non-cloneable wallet with immutable identity and an exclusive device session.
pub mod app;
pub mod builder;
pub mod config;
pub mod data;
pub mod proof;

use self::{
    app::{AppInfo, AppSettings},
    builder::Builder,
    config::{AddressOptions, DerivationPath, SigningPolicy},
    data::{LedgerDataRequest, SignedData},
    proof::{AddressProof, ProofRequest},
};
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    protocol::{
        self,
        client::{Client, verify},
    },
};
use ton::ton_core::traits::tlb::TLB;
use ton::{
    block_tlb::{CommonMsgInfoExtIn, Msg, StateInit},
    ton_core::{
        cell::{TonCell, TonHash},
        types::{
            TonAddress,
            tlb_core::{MsgAddressExt, TLBCoins, TLBEitherRef},
        },
    },
    ton_wallet::{WalletV3Data, WalletV4Data, WalletVersion},
};
/// An immutable wallet identity bound to one exclusive Ledger session.
///
/// Signing verifies the device key, reconstructed message hash and Ed25519
/// signature. No method broadcasts transactions. Dropping an in-flight operation
/// can leave the device prompt active and makes the session unusable; reconnect
/// before retrying. Inspect wallet history before retrying an uncertain broadcast.
pub struct TonLedgerWallet {
    client: Client,
    version: WalletVersion,
    public_key: [u8; 32],
    address: TonAddress,
    wallet_id: i32,
    derivation_path: DerivationPath,
    path: Vec<u8>,
    policy: SigningPolicy,
}
impl TonLedgerWallet {
    /// Configures a V3R2/V4R2 wallet; validation and device I/O happen in `build`.
    pub fn builder(version: WalletVersion) -> Builder {
        Builder::new(version)
    }
    /// Wallet code version selected at construction.
    pub fn version(&self) -> WalletVersion {
        self.version
    }
    /// Ed25519 public key read and bound during construction.
    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }
    /// Raw address derived locally from wallet code, public key and wallet ID.
    pub fn address(&self) -> &TonAddress {
        &self.address
    }
    /// Subwallet ID, preserving all 32 bits through the signed representation.
    pub fn wallet_id(&self) -> i32 {
        self.wallet_id
    }
    /// Key derivation configuration bound to this session.
    pub fn derivation_path(&self) -> &DerivationPath {
        &self.derivation_path
    }
    /// Constructs mode-3 bodies containing exactly one representable internal message.
    /// Empty bodies must be inline; nonempty bodies and state init must be references.
    /// `expire_at` is Unix seconds. Rejects unsupported layouts, wallet IDs,
    /// payload policies and message counts without contacting the device.
    pub fn create_ext_in_body(&self, expire_at: u32, seqno: u32, int_msgs: Vec<TonCell>) -> TonLedgerResult<TonCell> {
        if int_msgs.len() != 1 {
            return Err(TonLedgerError::Invalid("Ledger requires exactly one internal message"));
        }
        let body = WalletVersion::build_ext_in_body(self.version, expire_at, seqno, self.wallet_id, int_msgs)?;
        protocol::payload::transaction(self.version, self.wallet_id, &body, self.policy)?;
        Ok(body)
    }
    /// Signs only if firmware reconstruction, returned hash and Ed25519 all match.
    /// Unsupported inputs fail before I/O. Approval denial returns `UserDenied`;
    /// timeout, cancellation or verification failure requires a fresh session.
    pub async fn sign_ext_in_body(&mut self, body: &TonCell) -> TonLedgerResult<TonCell> {
        let payload = protocol::payload::transaction(self.version, self.wallet_id, body, self.policy)?;
        let hash = body.cell_hash()?;
        self.check_identity().await?;
        let response = self.client.chunked(6, &self.path, &payload).await?;
        let signature = self.verify(&response, hash.as_slice(), hash.as_slice())?;
        let mut builder = TonCell::builder();
        builder.write_bits(signature, 512)?;
        builder.write_cell(body)?;
        Ok(builder.build()?)
    }
    /// Wraps an already signed body for this wallet, optionally including deployment state.
    /// Does not contact the device or validate the supplied signature; use a body
    /// returned by [`Self::sign_ext_in_body`]. Cell encoding errors are propagated.
    pub fn create_ext_in_msg_from_body(&self, signed_body: TonCell, add_state_init: bool) -> TonLedgerResult<TonCell> {
        let info = CommonMsgInfoExtIn {
            src: MsgAddressExt::NONE,
            dst: self.address.to_msg_address_int(),
            import_fee: TLBCoins::ZERO,
        };
        let mut msg = Msg::new(info, signed_body);
        if add_state_init {
            msg.init = Some(TLBEitherRef::new(state_init(self.version, &self.public_key, self.wallet_id)?));
        }
        Ok(msg.to_cell()?)
    }
    /// Constructs, approves, verifies and wraps one internal message without broadcasting.
    /// `seqno` and Unix-seconds `expire_at` come from the caller. `add_state_init`
    /// includes wallet deployment state. Propagates construction and signing errors.
    pub async fn create_ext_in_msg(
        &mut self,
        int_msgs: Vec<TonCell>,
        seqno: u32,
        expire_at: u32,
        add_state_init: bool,
    ) -> TonLedgerResult<TonCell> {
        let body = self.create_ext_in_body(expire_at, seqno, int_msgs)?;
        let signed = self.sign_ext_in_body(&body).await?;
        self.create_ext_in_msg_from_body(signed, add_state_init)
    }
    /// Firmware returns the confirmed key; version, wallet ID and workchain are
    /// bound by request specifiers. Display flags do not alter the raw address.
    pub async fn confirm_address(&mut self, options: AddressOptions) -> TonLedgerResult<TonAddress> {
        let (flags, data) = self.address_request(options);
        let key = self.client.request(5, 1, flags, &data, true).await?;
        if key.as_slice() != self.public_key {
            self.client.invalidate();
            return Err(TonLedgerError::IdentityChanged);
        }
        Ok(self.address)
    }
    /// Requests approval and verifies a TON address proof for this wallet.
    /// Domain/payload length and APDU-budget errors fail before device I/O.
    /// Device, identity and signature errors are returned without a proof.
    pub async fn get_address_proof(
        &mut self,
        request: &ProofRequest,
        options: AddressOptions,
    ) -> TonLedgerResult<AddressProof> {
        if request.domain.len() > 128 || request.payload.len() > 128 {
            return Err(TonLedgerError::Invalid("proof domain/payload limit is 128 bytes"));
        }
        let (flags, mut data) = self.address_request(options);
        data.push(request.domain.len() as u8);
        data.extend(request.domain.as_bytes());
        data.extend(request.timestamp.to_be_bytes());
        data.extend(&request.payload);
        protocol::apdu::command(8, 1, flags, &data)?;
        let hash = protocol::proof::digest(&self.address, request);
        self.check_identity().await?;
        let response = self.client.request(8, 1, flags, &data, true).await?;
        let signature = self.verify(&response, &hash, &hash)?;
        Ok(AddressProof { signature, hash })
    }
    /// Signs the legacy schema/timestamp/cell-hash preimage, not TON Connect signData.
    /// `timestamp` is Unix seconds. Invalid input fails before I/O; approval and
    /// verification failures return errors rather than an unchecked signature.
    pub async fn sign_data(&mut self, request: &LedgerDataRequest, timestamp: u64) -> TonLedgerResult<SignedData> {
        let protocol::data::EncodedData {
            apdu: data,
            preimage,
            schema,
            hash: cell_hash,
        } = protocol::data::encode(request, timestamp)?;
        self.check_identity().await?;
        let response = self.client.chunked(9, &self.path, &data).await?;
        let signature = self.verify(&response, &cell_hash, &preimage)?;
        Ok(SignedData {
            signature,
            cell_hash,
            schema,
            timestamp,
        })
    }
    /// Queries the app and requires TON app major version 2.
    pub async fn app_info(&mut self) -> TonLedgerResult<AppInfo> {
        self.client.app_info().await
    }
    /// Reads blind-signing and expert-mode flags without changing device settings.
    pub async fn settings(&mut self) -> TonLedgerResult<AppSettings> {
        self.client.settings().await
    }
    fn address_request(&self, options: AddressOptions) -> (u8, Vec<u8>) {
        let flags = 4 | u8::from(options.testnet) | if self.address.workchain == -1 { 2 } else { 0 };
        let mut data = self.path.clone();
        data.push(u8::from(self.version == WalletVersion::V3R2));
        data.extend(self.wallet_id.to_be_bytes());
        (flags, data)
    }
    async fn check_identity(&mut self) -> TonLedgerResult<()> {
        self.client.app_info().await?;
        if self.client.key(&self.path).await? != self.public_key {
            self.client.invalidate();
            return Err(TonLedgerError::IdentityChanged);
        }
        Ok(())
    }
    fn verify(&mut self, response: &[u8], hash: &[u8], preimage: &[u8]) -> TonLedgerResult<[u8; 64]> {
        let result = verify(response, &self.public_key, hash, preimage);
        if result.is_err() {
            self.client.invalidate();
        }
        result
    }
}

fn state_init(version: WalletVersion, public_key: &[u8; 32], wallet_id: i32) -> TonLedgerResult<StateInit> {
    let public_key = TonHash::from_slice(public_key)?;
    let data = match version {
        WalletVersion::V3R2 => WalletV3Data::new(wallet_id, public_key).to_cell()?,
        WalletVersion::V4R2 => WalletV4Data::new(wallet_id, public_key).to_cell()?,
        _ => return Err(TonLedgerError::UnsupportedWallet),
    };
    Ok(StateInit::new(WalletVersion::get_code(version)?.clone(), data))
}
