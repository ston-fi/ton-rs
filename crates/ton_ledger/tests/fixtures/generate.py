"""Run with tonsdk==1.0.15 and the pinned app-ton/tests on PYTHONPATH.
Independent upstream Python codec, not the Rust implementation under test.
"""
from pathlib import Path
from tonsdk.utils import Address
from tonsdk.boc import Cell
from application_client import ton_transaction as t
from application_client import ton_sign_data as d

addr = Address('0:' + '11' * 32)
payloads = [
    t.CommentPayload('hello'),
    t.JettonTransferPayload(123456789, addr, addr, forward_amount=1),
    t.NFTTransferPayload(addr, addr, forward_amount=1),
    t.JettonBurnPayload(123, addr, custom_payload=b'\x12' * 20),
    t.AddWhitelistPayload(addr), t.SingleNominatorWithdrawPayload(123),
    t.ChangeValidatorPayload(addr), t.TonstakersDepositPayload(9),
    t.JettonDAOVotePayload(addr, 1700000000, True, False),
    t.ChangeDNSWalletPayload(addr, True, False),
    t.TokenBridgePaySwapPayload(b'\x42' * 32),
    t.TonWhalesPoolDepositPayload(12, 1),
    t.TonWhalesPoolWithdrawPayload(12, 1, 34),
    t.VestingSendMsgCommentPayload('hello', t.SendMode(3), addr, 10000000),
]
root = Path(__file__).parent
with (root / 'payloads.tsv').open('w') as f:
    for i, p in enumerate(payloads):
        c = p.to_message_body_cell()
        f.write(f'{i}\t{bytes(c.to_boc(False)).hex()}\t{p.to_request_bytes().hex()}\t{c.bytes_hash().hex()}\n')
with (root / 'transactions.tsv').open('w') as f:
    for v4 in [False, True]:
        for payload in [None, payloads[0]]:
            tx = t.Transaction(addr, t.SendMode(3), 7, 1700000000, False, 10000000,
                               payload=payload, subwallet_id=0xf1234567, include_wallet_op=v4)
            c = tx.transfer_cell()
            f.write(f'{int(v4)}\t{bytes(c.to_boc(False)).hex()}\t{tx.to_request_bytes().hex()}\t{c.bytes_hash().hex()}\n')
with (root / 'data.tsv').open('w') as f:
    for req in [d.PlaintextSignDataRequest('hello', 1700000000),
                d.AppDataSignDataRequest(Cell(), addr, 'app.example.org', Cell(), 1700000000)]:
        f.write(f'{req.to_request_bytes().hex()}\t{req.to_signed_data().hex()}\n')
