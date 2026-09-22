use crate::{
    client::{Client, verify},
    derivation_path::DerivationPath,
    error::{TonLedgerError, TransportError},
    proof::ProofRequest,
    protocol,
    ton_ledger_wallet::TonLedgerWallet,
    traits::Transport,
};
use async_trait::async_trait;
use ed25519_dalek::{Signer, SigningKey};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use ton::{
    block_tlb::{CommonMsgInfoInt, Msg},
    ton_core::{
        cell::TonCell,
        traits::tlb::TLB,
        types::{
            TonAddress,
            tlb_core::{EitherRefLayout, TLBCoins, TLBEitherRef},
        },
    },
    ton_wallet::{KeyPair, TonWallet, WalletVersion},
};
struct Script {
    steps: VecDeque<(Vec<u8>, Vec<u8>)>,
    seen: Arc<Mutex<usize>>,
}
#[async_trait]
impl Transport for Script {
    async fn exchange(&mut self, c: &[u8], _: Duration) -> Result<Vec<u8>, TransportError> {
        let (command, response) = self.steps.pop_front().ok_or(TransportError::Frame("unexpected command"))?;
        assert_eq!(c, command);
        *self.seen.lock().map_err(|_| TransportError::Disconnected)? += 1;
        Ok(response)
    }
}
fn ok(mut data: Vec<u8>) -> Vec<u8> {
    data.extend([0x90, 0]);
    data
}
fn key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}
fn setup() -> anyhow::Result<VecDeque<(Vec<u8>, Vec<u8>)>> {
    let path = hex::decode("068000002c8000025f80000000800000008000000080000000")?;
    Ok(VecDeque::from([
        (vec![0xe0, 4, 0, 0, 0], ok(b"TON".to_vec())),
        (vec![0xe0, 3, 0, 0, 0], ok(vec![2, 9, 1])),
        (protocol::command(5, 0, 0, &path)?, ok(key().verifying_key().to_bytes().to_vec())),
    ]))
}
fn signed(hash: &[u8], preimage: &[u8]) -> Vec<u8> {
    let mut out = vec![64];
    out.extend(key().sign(preimage).to_bytes());
    out.push(32);
    out.extend(hash);
    out
}
#[test]
fn test_signature_validation() -> anyhow::Result<()> {
    let hash = [9; 32];
    let response = signed(&hash, &hash);
    let public = key().verifying_key().to_bytes();
    verify(&response, &public, &hash, &hash)?;
    for n in 0..response.len() {
        assert!(verify(&response[..n], &public, &hash, &hash).is_err());
    }
    let mut extra = response.clone();
    extra.push(0);
    assert!(verify(&extra, &public, &hash, &hash).is_err());
    for offset in [0, 1, 65, 66] {
        let mut bad = response.clone();
        bad[offset] ^= 1;
        assert!(verify(&bad, &public, &hash, &hash).is_err());
    }
    assert!(verify(&response, &SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes(), &hash, &hash).is_err());
    Ok(())
}
#[test]
fn test_codec_boundaries() -> anyhow::Result<()> {
    for n in [254, 255] {
        assert_eq!(protocol::command(6, 0, 0, &vec![0; n])?.len(), n + 5);
    }
    assert!(protocol::command(6, 0, 0, &[0; 256]).is_err());
    let mut b = vec![];
    protocol::coins(&mut b, 0)?;
    assert_eq!(b, [0]);
    b.clear();
    protocol::coins(&mut b, (1u128 << 120) - 1)?;
    assert_eq!(b.len(), 16);
    assert!(protocol::coins(&mut b, 1u128 << 120).is_err());
    b.clear();
    protocol::uint48(&mut b, 1_700_000_000)?;
    assert_eq!(hex::encode(&b), "00006553f100");
    assert!(protocol::uint48(&mut b, 1 << 48).is_err());
    assert!(matches!(protocol::response(vec![0x69, 0x85]), Err(TonLedgerError::UserDenied)));
    assert!(matches!(protocol::response(vec![0xbd, 0]), Err(TonLedgerError::BlindSigningDisabled)));
    assert!(matches!(protocol::response(vec![0xab, 0xcd]), Err(TonLedgerError::Status(0xabcd))));
    Ok(())
}
#[tokio::test]
async fn test_chunk_transcripts() -> anyhow::Result<()> {
    for size in [254, 255, 256, 509, 510] {
        let payload = vec![0x42; size];
        let path = vec![3, 0, 0, 0, 44, 0, 0, 2, 95, 0, 0, 0, 0];
        let mut steps = VecDeque::new();
        steps.push_back((protocol::command(6, 0, 3, &path)?, ok(vec![])));
        let count = payload.chunks(255).len();
        for (i, c) in payload.chunks(255).enumerate() {
            steps.push_back((
                protocol::command(6, 0, if i + 1 == count { 0 } else { 2 }, c)?,
                ok(if i + 1 == count { vec![7] } else { vec![] }),
            ));
        }
        let seen = Arc::new(Mutex::new(0));
        let mut client = Client::new(
            Box::new(Script {
                steps,
                seen: seen.clone(),
            }),
            Duration::from_secs(1),
            Duration::from_secs(1),
        );
        assert_eq!(client.chunked(6, &path, &payload).await?, [7]);
        assert_eq!(*seen.lock().map_err(|_| anyhow::anyhow!("lock"))?, count + 1);
        assert!(client.chunked(6, &path, &[0; 511]).await.is_err());
    }
    Ok(())
}
struct Pending;
#[async_trait]
impl Transport for Pending {
    async fn exchange(&mut self, _: &[u8], _: Duration) -> Result<Vec<u8>, TransportError> {
        std::future::pending().await
    }
}
#[tokio::test]
async fn test_cancellation_and_timeout_poison_session() -> anyhow::Result<()> {
    let mut client = Client::new(Box::new(Pending), Duration::from_millis(20), Duration::from_millis(20));
    assert!(tokio::time::timeout(Duration::from_millis(1), client.chunked(6, &[1], &[2])).await.is_err());
    assert!(matches!(client.key(&[1]).await, Err(TonLedgerError::DirtySession)));
    let mut client = Client::new(Box::new(Pending), Duration::from_millis(1), Duration::from_millis(1));
    assert!(matches!(client.key(&[1]).await, Err(TonLedgerError::Transport(TransportError::Timeout))));
    assert!(matches!(client.key(&[1]).await, Err(TonLedgerError::DirtySession)));
    Ok(())
}
#[tokio::test]
async fn test_wallet_bytes_and_preflight() -> anyhow::Result<()> {
    for version in [WalletVersion::V3R2, WalletVersion::V4R2] {
        let k = key();
        let software = TonWallet::new(
            version,
            KeyPair {
                public_key: k.verifying_key().to_bytes(),
                secret_key: k.to_keypair_bytes(),
            },
        )?;
        let seen = Arc::new(Mutex::new(0));
        let mut steps = setup()?;
        let msg = Msg {
            info: CommonMsgInfoInt {
                bounce: false,
                ..CommonMsgInfoInt::new(software.address.to_msg_address_int().into(), TLBCoins::new(10_000_000))
            }
            .into(),
            init: None,
            body: TLBEitherRef::new_with_layout(TonCell::empty().clone(), EitherRefLayout::ToCell),
        }
        .to_cell()?;
        let body = software.create_ext_in_body(1_700_000_000, 7, vec![msg.clone()])?;
        steps.extend(setup()?);
        let path = DerivationPath::default().encode(0)?;
        // Independent fixed transaction request (zero address varies with wallet version).
        let mut payload = hex::decode(if version == WalletVersion::V4R2 {
            "0129a9a31701000000076553f1000398968000"
        } else {
            "0129a9a31700000000076553f1000398968000"
        })?;
        payload.extend(software.address.hash.as_slice());
        payload.extend([0, 3, 0, 0, 0]);
        steps.push_back((protocol::command(6, 0, 3, &path)?, ok(vec![])));
        steps.push_back((
            protocol::command(6, 0, 0, &payload)?,
            ok(signed(body.cell_hash()?.as_slice(), body.cell_hash()?.as_slice())),
        ));
        let mut wallet = TonLedgerWallet::builder(version)
            .with_transport(Script {
                steps,
                seen: seen.clone(),
            })
            .build()
            .await?;
        assert_eq!(wallet.address(), &software.address);
        assert!(wallet.create_ext_in_body(1, 1, vec![]).is_err());
        assert_eq!(*seen.lock().map_err(|_| anyhow::anyhow!("lock"))?, 3);
        let unsigned = wallet.create_ext_in_body(1_700_000_000, 7, vec![msg])?;
        let signed = wallet.sign_ext_in_body(&unsigned).await?;
        assert_eq!(signed.to_boc()?, software.sign_ext_in_body(&body)?.to_boc()?);
        for init in [false, true] {
            assert_eq!(
                wallet.create_ext_in_msg_from_body(signed.clone(), init)?.to_boc()?,
                software.create_ext_in_msg_from_body(signed.clone(), init)?.to_boc()?
            );
        }
    }
    Ok(())
}
#[test]
fn test_derivation_and_legacy_data() -> anyhow::Result<()> {
    assert_eq!(
        hex::encode(
            DerivationPath::Ton {
                account: 8,
                testnet: true
            }
            .encode(-1)?
        ),
        "068000002c8000025f80000001800000ff8000000880000000"
    );
    for p in [vec![44, 607], vec![44, 607, 0x80000000], vec![0; 11]] {
        assert!(DerivationPath::Custom(p).encode(0).is_err());
    }
    let crate::data::EncodedData {
        apdu: bytes,
        preimage,
        schema,
        hash,
    } = crate::data::encode(&crate::data::LedgerDataRequest::Plaintext("hello".into()), 1_700_000_000)?;
    assert_eq!(schema, 0x754bf91b);
    assert_eq!(hex::encode(bytes), "754bf91b000000006553f10068656c6c6f");
    assert_eq!(&preimage[12..], &hash);
    assert_eq!(preimage.len(), 44);
    assert!(crate::data::encode(&crate::data::LedgerDataRequest::Plaintext("é".into()), 0).is_err());
    let r = ProofRequest::new("example.org".into(), 1_700_000_000, b"challenge".to_vec());
    // Python hashlib vector, independent of Rust digest implementation.
    assert_eq!(
        hex::encode(crate::proof::digest(&TonAddress::ZERO, &r)),
        "5bf275dadfbf9dac875ccdc2ea769cce50a031c2493a1d8807660999c25ca80e"
    );
    Ok(())
}

#[test]
fn test_upstream_data_vectors() -> anyhow::Result<()> {
    use crate::data::{LedgerDataRequest, encode};
    let addr = TonAddress::new(0, ton::ton_core::cell::TonHash::from_slice(&[0x11; 32])?);
    let requests = [
        LedgerDataRequest::Plaintext("hello".into()),
        LedgerDataRequest::AppData {
            address: Some(addr),
            domain: Some("app.example.org".into()),
            data: TonCell::empty().clone(),
            extension: Some(TonCell::empty().clone()),
        },
    ];
    for (request, line) in requests.iter().zip(include_str!("../tests/fixtures/data.tsv").lines()) {
        let fields: Vec<_> = line.split('\t').collect();
        let crate::data::EncodedData { apdu, preimage, .. } = encode(request, 1_700_000_000)?;
        assert_eq!(hex::encode(apdu), fields[0]);
        assert_eq!(hex::encode(preimage), fields[1]);
    }
    Ok(())
}
#[tokio::test]
async fn test_proof_budget_precedes_io() -> anyhow::Result<()> {
    let seen = Arc::new(Mutex::new(0));
    let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2)
        .with_transport(Script {
            steps: setup()?,
            seen: seen.clone(),
        })
        .build()
        .await?;
    let request = ProofRequest::new("d".repeat(128), 1, vec![1; 89]);
    assert!(wallet.get_address_proof(&request, crate::app::AddressOptions::default()).await.is_err());
    assert_eq!(*seen.lock().map_err(|_| anyhow::anyhow!("lock"))?, 3);
    Ok(())
}

#[tokio::test]
async fn test_builder_order_and_signed_identity() -> anyhow::Result<()> {
    for version in [WalletVersion::V3R2, WalletVersion::V4R2] {
        let key = key();
        let id = 0xf1234567u32 as i32;
        let path = DerivationPath::Ton {
            account: 19,
            testnet: true,
        };
        let mut steps = setup()?;
        steps[2].0 = protocol::command(5, 0, 0, &path.encode(-1)?)?;
        let seen = Arc::new(Mutex::new(0));
        let wallet = TonLedgerWallet::builder(version)
            .with_derivation_path(path.clone())
            .with_workchain(-1)
            .with_wallet_id(id)
            .with_transport(Script { steps, seen })
            .build()
            .await?;
        let software = TonWallet::new_with_params(
            version,
            KeyPair {
                public_key: key.verifying_key().to_bytes(),
                secret_key: key.to_keypair_bytes(),
            },
            -1,
            id,
        )?;
        assert_eq!(wallet.address(), &software.address);
        assert_eq!(wallet.derivation_path(), &path);
    }
    // After construction a different key is rejected before starting a signing sequence.
    let mut steps = setup()?;
    let mut changed = setup()?;
    changed[2].1 = ok(SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes().to_vec());
    steps.extend(changed);
    let seen = Arc::new(Mutex::new(0));
    let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2)
        .with_transport(Script {
            steps,
            seen: seen.clone(),
        })
        .build()
        .await?;
    let req = crate::data::LedgerDataRequest::Plaintext("hello".into());
    assert!(matches!(wallet.sign_data(&req, 0).await, Err(TonLedgerError::IdentityChanged)));
    assert!(matches!(wallet.settings().await, Err(TonLedgerError::DirtySession)));
    assert_eq!(*seen.lock().map_err(|_| anyhow::anyhow!("lock"))?, 6);
    Ok(())
}

struct PauseAfterPath {
    calls: usize,
}
#[async_trait]
impl Transport for PauseAfterPath {
    async fn exchange(&mut self, _: &[u8], _: Duration) -> Result<Vec<u8>, TransportError> {
        self.calls += 1;
        if self.calls == 1 { Ok(ok(vec![])) } else { std::future::pending().await }
    }
}
#[tokio::test]
async fn test_cancellation_between_chunks() -> anyhow::Result<()> {
    let mut client = Client::new(Box::new(PauseAfterPath { calls: 0 }), Duration::from_secs(1), Duration::from_secs(1));
    assert!(tokio::time::timeout(Duration::from_millis(1), client.chunked(6, &[1], &[3; 256])).await.is_err());
    assert!(matches!(client.chunked(6, &[1], &[3]).await, Err(TonLedgerError::DirtySession)));
    Ok(())
}

#[tokio::test]
async fn test_app_identity_settings_and_invalid_configuration() -> anyhow::Result<()> {
    let seen = Arc::new(Mutex::new(0));
    assert!(matches!(
        TonLedgerWallet::builder(WalletVersion::V5R1)
            .with_transport(Script {
                steps: VecDeque::new(),
                seen: seen.clone()
            })
            .build()
            .await,
        Err(TonLedgerError::UnsupportedWallet)
    ));
    assert_eq!(*seen.lock().map_err(|_| anyhow::anyhow!("lock"))?, 0);
    for version in [[2, 9, 0], [2, 10, 0]] {
        let mut steps = setup()?;
        steps[1].1 = ok(version.to_vec());
        assert!(matches!(
            TonLedgerWallet::builder(WalletVersion::V4R2)
                .with_transport(Script {
                    steps,
                    seen: seen.clone()
                })
                .build()
                .await,
            Err(TonLedgerError::UnvalidatedFirmware(_))
        ));
    }
    let mut steps = setup()?;
    steps[0].1 = ok(b"Not TON".to_vec());
    assert!(matches!(
        TonLedgerWallet::builder(WalletVersion::V4R2)
            .with_transport(Script {
                steps,
                seen: seen.clone()
            })
            .build()
            .await,
        Err(TonLedgerError::WrongApp)
    ));
    let mut steps = setup()?;
    steps.extend([
        (vec![0xe0, 10, 0, 0, 0], ok(vec![3])),
        (vec![0xe0, 10, 0, 0, 0], ok(vec![4])),
    ]);
    let mut wallet =
        TonLedgerWallet::builder(WalletVersion::V4R2).with_transport(Script { steps, seen }).build().await?;
    let s = wallet.settings().await?;
    assert!(s.blind_signing && s.expert_mode);
    assert!(matches!(wallet.settings().await, Err(TonLedgerError::Response(_))));
    Ok(())
}
