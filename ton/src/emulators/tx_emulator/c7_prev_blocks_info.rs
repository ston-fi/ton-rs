use crate::bail_ton;
use crate::block_tlb::{BlockIdExt, ShardIdent, TVMStackValue, TVMTuple};
use crate::errors::TonResult;
use crate::ton_core::errors::TonCoreError;
use fastnum::I512;
use std::ops::Deref;
use std::sync::Arc;
use ton_core::bail_ton_core_data;
use ton_core::cell::{CellBuilder, CellParser, TonHash};
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;

// 13th element of c7 register
// https://docs.ton.org/v3/documentation/tvm/changelog/tvm-upgrade-2023-07#opcodes-to-work-with-new-c7-values
#[derive(Debug, PartialEq)]
pub struct C7PrevBlocksInfo {
    pub last_mc_blocks: Arc<Vec<BlockIdExt>>,
    pub prev_key_block: BlockIdExt,
    pub last_mc_blocks_div_100: Arc<Vec<BlockIdExt>>,
}

// [ wc:Integer shard:Integer seqno:Integer root_hash:Integer file_hash:Integer ] = BlockId;
// [ last_mc_blocks:BlockId[] prev_key_block:BlockId last_mc_blocks_divisible_by_100:BlockId ] = PrevBlocksInfo;
impl TLB for C7PrevBlocksInfo {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        let main_tuple = TVMTuple::read(parser)?;
        if main_tuple.len() != 3 {
            bail_ton_core_data!("Expected PrevBlocksInfo tuple of length 3, found {}", main_tuple.len());
        }

        let last_mc_blocks_tuple = main_tuple.get_tuple(0)?;
        let mut last_mc_blocks = Vec::with_capacity(last_mc_blocks_tuple.len());
        for value in last_mc_blocks_tuple.deref() {
            last_mc_blocks.push(block_id_from_stack_value(value)?);
        }

        let prev_key_block = block_id_from_stack_value(main_tuple.get(1).unwrap())?;

        let last_mc_blocks_div_100_tuple = main_tuple.get_tuple(2)?;
        let mut last_mc_blocks_div_100 = Vec::with_capacity(last_mc_blocks_div_100_tuple.len());
        for value in last_mc_blocks_div_100_tuple.deref() {
            last_mc_blocks_div_100.push(block_id_from_stack_value(value)?);
        }

        Ok(C7PrevBlocksInfo {
            last_mc_blocks: Arc::new(last_mc_blocks),
            prev_key_block,
            last_mc_blocks_div_100: Arc::new(last_mc_blocks_div_100),
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        let mut last_mc_blocks_tuple = TVMTuple::default();
        for block in self.last_mc_blocks.deref() {
            last_mc_blocks_tuple.push_tuple(block_id_to_tuple(block));
        }

        let prev_key_block_tuple = block_id_to_tuple(&self.prev_key_block);

        let mut last_mc_blocks_div_100_tuple = TVMTuple::default();
        for block in self.last_mc_blocks_div_100.deref() {
            last_mc_blocks_div_100_tuple.push_tuple(block_id_to_tuple(block));
        }

        let mut main_tuple = TVMTuple::default();
        main_tuple.push_tuple(last_mc_blocks_tuple);
        main_tuple.push_tuple(prev_key_block_tuple);
        main_tuple.push_tuple(last_mc_blocks_div_100_tuple);
        main_tuple.write(builder)
    }
}

fn block_id_to_tuple(block: &BlockIdExt) -> TVMTuple {
    let mut tuple = TVMTuple::default();
    tuple.push_int(I512::from_i32(block.shard_ident.workchain));
    tuple.push_int(I512::from_u64(block.shard_ident.shard).unwrap());
    tuple.push_int(I512::from_u32(block.seqno));
    tuple.push_int(block.root_hash.to_i512());
    tuple.push_int(block.file_hash.to_i512());
    tuple
}

fn block_id_from_stack_value(value: &TVMStackValue) -> TonResult<BlockIdExt> {
    let TVMStackValue::Tuple(tuple) = value else {
        bail_ton!("Expected Tuple for BlockIdExt, found {value:?}");
    };
    let Ok(workchain) = tuple.get_int(0)?.to_i32() else {
        bail_ton!("Can't convert {} to i32", tuple.get_int(0)?);
    };
    let shard = tuple.get_int(1)?.to_u64().unwrap();
    let seqno = tuple.get_int(2)?.to_u32().unwrap();
    let root_hash = TonHash::from_i512(tuple.get_int(3)?)?;
    let file_hash = TonHash::from_i512(tuple.get_int(4)?)?;

    Ok(BlockIdExt {
        shard_ident: ShardIdent { workchain, shard },
        seqno,
        root_hash,
        file_hash,
    })
}

#[cfg(test)]
mod tests {
    use crate::emulators::tx_emulator::C7PrevBlocksInfo;
    use std::ops::Deref;
    use ton_core::traits::tlb::TLB;

    #[test]
    fn test_c7_prev_blocks_info_serde() -> anyhow::Result<()> {
        let b64 = "b5ee9c7201028f0100097b00020607000301020200030402060700100586020607001006070206070005080902000a8602000b0c02060700050d0e02000f1000440200670bc67f1efbb52e985c40fd8e97e44047363e6138181124bdc6ab023090891e0200118602001213020607000514150200161700440200f13ba02badcc8a91657e7c7d6f8757aaf3d2c8ee3cf19151f1e7642e5074c7e602008b1800440200780889b74ac5da9fe7228d57bcd618106207be748a70e13c9c6f954fe0ceb4cf0200198602001a1b02060700051c1d02001e1f00440200c47d3e64aded7eab7cbefb5245d4f53878a40f12a47431560b524f42998dce4002008b20004402006e83d710803e0a7551a054c9fc993d9721f3e25cf9eb12d16d8a10160f381a56004402000000000000000000000000000000000000000000000000000000000002b762c50200218602002223020607000524250200262700440200699546e1e2994eb105fe4b50de658c88983275260e7a49edae06c527e3cac02102008b280044020032fc568d73df0b846b465b1d7fcb07c4d84c1cf9127496d4d8971e9dd2557bd8004402000000000000000000000000000000000000000000000000000000000002b7a6510200298602002a2b02060700052c2d02002e2f0044020007b4d40543f2ae96129724729533ed4fd7995d0362c22d4d0e3d8de42a05471902008b3000440200dc11b8949fcadbd1a5e2d576494e05b0e94aba47749e030b1986cf196fb8229a004402000000000000000000000000000000000000000000000000000000000002b7a6500200318602003233020607000534350200363700440200cb01f8076f6bf39066478b2a55c5024b82f5a764122998aeda3ea545a978dd9d02008b3800440200980cb37deeddbfde49fb196ff83ec5db58fc8c4361dada12c934566d4c407549004402000000000000000000000000000000000000000000000000000000000002b7a64f0200398602003a3b02060700053c3d02003e3f00440200088aa98f4ad71b8adf7683049f0451d4fcd43c0a2b2e2bb7384ecda57fe94f4402008b4000440200ff4f3bd9df5642a2a568401eb885c2271aea5a67a9ed06440ea460d2016db047004402000000000000000000000000000000000000000000000000000000000002b7a64e02004186020042430206070005444502004647004402009f17810af2f12b65549492c92615bef8991905dfb06bd9a4c81fc10f4180010c02008b480044020027ab7aff5b80870c02b997b238d3e9c75b83468d98b48bc1deb539b6654a06f6004402000000000000000000000000000000000000000000000000000000000002b7a64d0200498602004a4b02060700054c4d02004e4f004402006ff30af1533225c2bcbe75bef21b06d8d247620e00a6c99ff73daae755cc8a1302008b5000440200ab24bcc0e830250524cde7835cff9690ffec8d34a40b2c72dcc07ae1f33a010b004402000000000000000000000000000000000000000000000000000000000002b7a64c0200518602005253020607000554550200565700440200629eb582deab8450cd6a22d080a8cf6dda4414754b2a8f218c21c3f542749e5602008b58004402009e88cc3c6d1b43cdad4d181a0fd2291595310aeed42560b8a930cf9aac9bc8cb004402000000000000000000000000000000000000000000000000000000000002b7a64b0200598602005a5b02060700055c5d02005e5f00440200638af919862ab9d7280abf3a0647439b504367cdfd4f282286f8863176e6548502008b6000440200cf737d64463f2303516a295792d8e90b8d3aa734c1c41c9a5397cf5806bc900d004402000000000000000000000000000000000000000000000000000000000002b7a64a02006186020062630206070005646502006667004402006146c8577dc015f728cd9bbdcb7dad5b449199d6d1d4f115757d29358ca64a0802008b68004402007523ea59057a2355d7203e204d15b9b8c5d9d86d98b7a8361d97edf34d0281e7004402000000000000000000000000000000000000000000000000000000000002b7a6490200698602006a6b02060700056c6d02006e6f00440200dd4280dce39bb5bc274b10e2a6777c2b80c0a422752d099882952eb7eb22128a02008b70004402004f5d0c6ae205a5495ef35cbbb303bcafcc6dc3eb5f9cd2300bbb712704b5d703004402000000000000000000000000000000000000000000000000000000000002b7a648020086860200717202060700057374020075760044020072bee1f8e7729087201ef61791cb5046c1f00b58917e10ce4e4315c43a0bbe8f02008b77004402001eced4630221627e76f6340cb7f6ced6376ddd7ab92b16b4a7d4a14e1aade4e5004402000000000000000000000000000000000000000000000000000000000002b7a6470206070005787902060700057a7b02007c7d0044020027d78cd69197b4ea6a7ee7ca2af9dc5e1f24d6c5193b15c11162cd2ed2c6589602008b7e004402001975bfb520280196a0081ee9d440c4a7f4bc528d8506839b90b3fee6f246cb57004402000000000000000000000000000000000000000000000000000000000002b7a64602007f8000440200f51826578084271dc6005059885205c1dfd13ab76cb3ae390bfb7d4d6f08d6b702008182004402005279e7230aa5efe11bfe09e2a190ea1a2853b20f7850a21a3061b8f43fb26e9602008b8300440200e0a634c7cf683656c99a66b64f0d2db7f6eb1226d26cebd1aa5424a5c34ee80e004402000000000000000000000000000000000000000000000000000000000002b7a64502008b840044020015e3345801ce5d3c0df5ee8531b8a09383a7609e1ce8ba147450f69da623c11702008b8500440200e5bbaac74f8d13785f5b723644a069a975e478c235e76674d6b53201c0498aac004402000000000000000000000000000000000000000000000000000000000002b7a644004402000000000000000000000000000000000000000000000000000000000002b7a642004402000000000000000000000000000000000000000000000000000000000002b7a643020607000587880200898a00440200e37dde246bf25625b7a53e2192d386b84ed28103d3c37cfe1f7e6970b2709e7302008b8c004402008ce90a264ce83346d4984b715bffb14ea1ad550e09718030c885481636c8eb4e02008d8e00440200000000000000000000000000000000000000000000000000000000000006f4dc00440201ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff004402000000000000000000000000000000000000000000000000008000000000000000";
        let parsed = C7PrevBlocksInfo::from_boc_hex(&b64)?;
        assert_eq!(parsed.last_mc_blocks.len(), 16);
        let mut cur_seqno = 45590082u32;
        for block in parsed.last_mc_blocks.deref() {
            assert_eq!(block.seqno, cur_seqno);
            cur_seqno += 1;
        }

        let serialized = parsed.to_boc_base64()?;
        let parsed_back = C7PrevBlocksInfo::from_boc_base64(&serialized)?;
        assert_eq!(parsed, parsed_back);
        Ok(())
    }
}
