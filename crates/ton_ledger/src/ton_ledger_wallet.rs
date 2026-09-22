//! A non-cloneable wallet with immutable identity and an exclusive device session.
pub mod builder;
use crate::{
    app::{AddressOptions, AppInfo, AppSettings},
    client::{Client, verify},
    data::{LedgerDataRequest, SignedData},
    derivation_path::DerivationPath,
    error::{TonLedgerError, TonLedgerResult},
    proof::{AddressProof, ProofRequest},
    signing::SigningPolicy,
};
use builder::Builder;
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
    pub fn builder(version: WalletVersion) -> Builder {
        Builder::new(version)
    }
    pub fn version(&self) -> WalletVersion {
        self.version
    }
    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }
    pub fn address(&self) -> &TonAddress {
        &self.address
    }
    pub fn wallet_id(&self) -> i32 {
        self.wallet_id
    }
    pub fn derivation_path(&self) -> &DerivationPath {
        &self.derivation_path
    }
    /// Constructs mode-3 bodies containing exactly one representable internal message.
    /// Empty bodies must be inline; nonempty bodies and state init must be references.
    pub fn create_ext_in_body(&self, expire_at: u32, seqno: u32, int_msgs: Vec<TonCell>) -> TonLedgerResult<TonCell> {
        if int_msgs.len() != 1 {
            return Err(TonLedgerError::Invalid("Ledger requires exactly one internal message"));
        }
        let body = WalletVersion::build_ext_in_body(self.version, expire_at, seqno, self.wallet_id, int_msgs)?;
        crate::payload::transaction(self.version, self.wallet_id, &body, self.policy)?;
        Ok(body)
    }
    /// Signs only if firmware reconstruction, returned hash and Ed25519 all match.
    pub async fn sign_ext_in_body(&mut self, body: &TonCell) -> TonLedgerResult<TonCell> {
        let payload = crate::payload::transaction(self.version, self.wallet_id, body, self.policy)?;
        let hash = body.cell_hash()?;
        self.check_identity().await?;
        let response = self.client.chunked(6, &self.path, &payload).await?;
        let signature = self.verify(&response, hash.as_slice(), hash.as_slice())?;
        let mut builder = TonCell::builder();
        builder.write_bits(signature, 512)?;
        builder.write_cell(body)?;
        Ok(builder.build()?)
    }
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
        crate::protocol::command(8, 1, flags, &data)?;
        let hash = crate::proof::digest(&self.address, request);
        self.check_identity().await?;
        let response = self.client.request(8, 1, flags, &data, true).await?;
        let signature = self.verify(&response, &hash, &hash)?;
        Ok(AddressProof { signature, hash })
    }
    /// Signs the legacy schema/timestamp/cell-hash preimage, not TON Connect signData.
    pub async fn sign_data(&mut self, request: &LedgerDataRequest, timestamp: u64) -> TonLedgerResult<SignedData> {
        let crate::data::EncodedData {
            apdu: data,
            preimage,
            schema,
            hash: cell_hash,
        } = crate::data::encode(request, timestamp)?;
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
    pub async fn app_info(&mut self) -> TonLedgerResult<AppInfo> {
        self.client.app_info().await
    }
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
