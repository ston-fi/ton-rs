use anyhow::Result;
use ton_core::traits::tlb::TLB;

use crate::contracts::tep::metadata::metadata_content::MetadataContent;
use log::LevelFilter;
use log4rs::Config;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Root};
use std::sync::Once;

static LOG: Once = Once::new();
pub fn init_logging() {
    LOG.call_once(|| {
        let stderr = ConsoleAppender::builder()
            .target(Target::Stderr)
            .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S%.6f)} {T:>15.15} {h({l:>5.5})} {t}:{L} - {m}{n}",
            )))
            .build();

        let config = Config::builder()
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .build(Root::builder().appender("stderr").build(LevelFilter::Info))
            .unwrap();

        log4rs::init_config(config).unwrap();
    })
}

#[test]
fn byte_unaligned_content() -> Result<()> {
    init_logging();
    let content = MetadataContent::from_boc_hex(
        "b5ee9c7201010a0100d400010300c00102012002060143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e6400301050000c00402012005060143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e640070143bff7407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f400901020008008c68747470733a2f2f6a736f6e626c6f622e636f6d2f6170692f6a736f6e426c6f622f30313964666366342d333764322d376139332d393333392d31303131303138356365333000040039",
    )?;

    dbg!(content);
    Ok(())
}
