mod benchmark_utils;

use crate::benchmark_utils::check_cpu_id;
use ton::emulators::tx_emulator::{TXEmulArgs, TXEmulOrdArgs};

use crate::benchmark_utils::cpu_load_function;
use clap::Parser;
use core_affinity::set_for_current;
use criterion::Criterion;
use futures_util::future::join_all;
use std::hint::black_box;
use std::time::Duration;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
        LazyLock, Mutex, OnceLock,
    },
    thread,
};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio_test::{assert_err, assert_ok};
use ton::block_tlb::{Msg, ShardAccount, Tx};
use ton::emulators::emul_bc_config::EmulBCConfig;
use ton::emulators::tx_emulator::TXEmulator;
use ton_core::errors::{TonCoreError, TonCoreResult};
use ton_core::traits::tlb::TLB;
const BENCH_WARMUP_TIME_SECS: u64 = 2;
const BENCH_MEASUREMENT_TIME_SECS: u64 = 5;

const DEFAULT_SLEEP_TIME_MICROS: u64 = 1000;
const AFFINITY_CORE_ID: usize = 1;

const BENCH_ITERATIONS_COUNT: u32 = 100;

const CRITERION_SAMPLES_COUNT: u32 = 10;

const LOCAL_CYCLES_IN_MICROS: u64 = 2600;
static PIN_TO_CORE: OnceLock<bool> = OnceLock::new();
static WORKER_THREADS_COUNT: OnceLock<u32> = OnceLock::new();

use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Pin threads to specific cores
    #[arg(long = "pin-to-core", alias = "pin_to_core", default_value_t = true, action = clap::ArgAction::Set)]
    pin_to_core: bool,
    #[arg(long = "threads", alias = "threads", default_value_t = 1, action = clap::ArgAction::Set)]
    threads: u32,
}

fn parse_custom_args() -> Args {
    let mut filtered: Vec<String> = vec![std::env::args().next().unwrap_or_else(|| "bench".into())];
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        let normalized = arg.replace('_', "-");
        if normalized.starts_with("--pin-to-core")
            || normalized.starts_with("--threads-count")
            || normalized.starts_with("--iterations-count")
        {
            filtered.push(arg.clone());
            if !arg.contains('=') {
                if let Some(value) = iter.next() {
                    filtered.push(value);
                }
            }
        }
    }
    Args::parse_from(filtered)
}

fn configure_criterion() -> Criterion {
    // Parse both Criterion's and your own arguments
    let args = parse_custom_args();
    println!(
        "Benchmark config: PIN_TO_CORE = {}, THREADS_COUNT = {}, ITERATIONS_COUNT = {}",
        args.pin_to_core, args.threads, BENCH_ITERATIONS_COUNT
    );

    // You can stash these globals for later if needed
    PIN_TO_CORE.set(args.pin_to_core).unwrap();
    WORKER_THREADS_COUNT.set(args.threads).unwrap();
    // ITERATIONS_COUNT.set(args.iterations_count).unwrap();

    Criterion::default()
        .with_output_color(true)
        .without_plots()
        .sample_size(CRITERION_SAMPLES_COUNT as usize)
        .warm_up_time(Duration::from_secs(BENCH_WARMUP_TIME_SECS))
        .measurement_time(Duration::from_secs(BENCH_MEASUREMENT_TIME_SECS))
}

static BC_CONFIG: LazyLock<EmulBCConfig> = LazyLock::new(|| {
    EmulBCConfig::from_boc_hex(include_str!("../resources/tests/bc_config_key_block_42123611.hex")).unwrap()
});
#[allow(dead_code)]
fn test_emulator_iteration(emulator: &mut TXEmulator) -> anyhow::Result<()> {
    sys_tonlib_set_verbosity_level(0);

    let shard_account = ShardAccount::from_boc_hex("b5ee9c720102170100036600015094fb2314023373e7b36b05b69e31508eba9ba24a60e994060fee1ca55302f8c2000030a4972bcd4301026fc0092eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e422c858e433ce5b6500000c2925caf351106c29d2a534002030114ff00f4a413f4bcf2c80b0400510000001129a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4400201200506020148070804f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff090a0b0c02e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d0d0e0201200f10006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a7020070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb00000af400c9ed54007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e202012011120059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f318402015813140011b8c97ed44d0d70b1f8003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002012015160019adce76a26840206b90eb85ffc00019af1df6a26840106b90eb858fc0")?;
    let ext_in_msg = Msg::from_boc_hex("b5ee9c72010204010001560001e1880125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d4d18bb3ce5c84000000088001c01016862004975c883aea91de93142ae4dc222d803c74e5f130f37ef0d42fb353897fd0f982068e77800000000000000000000000000010201b20f8a7ea500000000000000005012a05f20080129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b0024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f90814dc9381030099259385618012934339d11465553b2f3e428ae79b0b1e2fd250b80784d4996dd44741736528ca0259f3a0f90024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f910")?;

    let mut ord_args = TXEmulOrdArgs {
        in_msg_boc: ext_in_msg.to_boc()?,
        emul_args: TXEmulArgs {
            shard_account_boc: shard_account.to_boc()?,
            bc_config: BC_CONFIG.clone(),
            rand_seed: TonHash::from_str("14857b338a5bf80a4c87e726846672173bb780f694c96c15084a3cbcc719ebf0")?,
            utime: 1738323935,
            lt: 53483578000001,
            ignore_chksig: false,
            prev_blocks_boc: None,
            libs_boc: None,
        },
    };
    assert_err!(emulator.emulate_ord(&ord_args));
    ord_args.emul_args.ignore_chksig = true;
    let response = assert_ok!(emulator.emulate_ord(&ord_args));
    assert!(response.success);

    let expected_tx = Tx::from_boc_hex("b5ee9c7241020c010002f50003b5792eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e4000030a49dab028194fb2314023373e7b36b05b69e31508eba9ba24a60e994060fee1ca55302f8c2000030a4972bcd43679cb7df00034657bf0280102030201e00405008272fb026ad92478055ab0086833e193b9e2ad35aa0073769228fcdc27ed38ef72a4c533ffcf55fd97275de407b0068404ed61966be66ec1e82d6c49d100f01e6064020f0c51c618a18604400a0b01e1880125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d4d18bb3ce5c84000000088001c060101df07016862004975c883aea91de93142ae4dc222d803c74e5f130f37ef0d42fb353897fd0f982068e77800000000000000000000000000010801b1680125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc90024bae441d7548ef498a15726e1116c01e3a72f89879bf786a17d9a9c4bfe87cc103473bc000614884c000061493b560504cf396fbec00801b20f8a7ea500000000000000005012a05f20080129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b0024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f90814dc9381090099259385618012934339d11465553b2f3e428ae79b0b1e2fd250b80784d4996dd44741736528ca0259f3a0f90024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f910009d419d8313880000000000000000110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020006fc987b3184c14882800000000000200000000000224cb2890dee94c80761e06b8c446b1a9835aff2fc055cee75373ceeceffa6b4240d03f644db9e7b3")?;
    let expected_shard_account = ShardAccount::from_boc_hex("b5ee9c7241021701000366000150775a15d6954e05b73e0c25729e776e6be6328ed14ebaf7262014603827198d24000030a49dab028101026fc0092eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e422c858e433ce5bef80000c29276ac0a0d036dd880934002030114ff00f4a413f4bcf2c80b0400510000001229a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4400201200506020148070804f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff1314151602e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d090a0201200b0c007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e20201200d0e0059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f31840201580f100011b8c97ed44d0d70b1f8003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002012011120019adce76a26840206b90eb85ffc00019af1df6a26840106b90eb858fc0006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a7020070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb00000af400c9ed5494cb980d")?;
    assert_eq!(response.shard_account_parsed()?, expected_shard_account);
    assert_eq!(response.tx_parsed()?, expected_tx);
    Ok(())
}

fn get_test_args() -> anyhow::Result<TXEmulOrdArgs> {
    let shard_account = ShardAccount::from_boc_hex("b5ee9c720102170100036600015094fb2314023373e7b36b05b69e31508eba9ba24a60e994060fee1ca55302f8c2000030a4972bcd4301026fc0092eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e422c858e433ce5b6500000c2925caf351106c29d2a534002030114ff00f4a413f4bcf2c80b0400510000001129a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4400201200506020148070804f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff090a0b0c02e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d0d0e0201200f10006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a7020070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb00000af400c9ed54007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e202012011120059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f318402015813140011b8c97ed44d0d70b1f8003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002012015160019adce76a26840206b90eb85ffc00019af1df6a26840106b90eb858fc0")?;
    let ext_in_msg = Msg::from_boc_hex("b5ee9c72010204010001560001e1880125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d4d18bb3ce5c84000000088001c01016862004975c883aea91de93142ae4dc222d803c74e5f130f37ef0d42fb353897fd0f982068e77800000000000000000000000000010201b20f8a7ea500000000000000005012a05f20080129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b0024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f90814dc9381030099259385618012934339d11465553b2f3e428ae79b0b1e2fd250b80784d4996dd44741736528ca0259f3a0f90024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f910")?;

    Ok(TXEmulOrdArgs {
        in_msg_boc: ext_in_msg.to_boc()?,
        emul_args: TXEmulArgs {
            shard_account_boc: shard_account.to_boc()?,
            bc_config: BC_CONFIG.clone(),
            rand_seed: TonHash::from_str("14857b338a5bf80a4c87e726846672173bb780f694c96c15084a3cbcc719ebf0")?,
            utime: 1738323935,
            lt: 53483578000001,
            ignore_chksig: true,
            prev_blocks_boc: None,
            libs_boc: None,
        },
    })
}
fn pin_to_core() -> bool { *PIN_TO_CORE.get_or_init(|| true) }

fn threads_count() -> u32 { *WORKER_THREADS_COUNT.get_or_init(|| 1) } // *THREADS_COUNT.get_or_init(|| 5)

fn total_requests() -> u32 { BENCH_ITERATIONS_COUNT }

fn check_thread_params(msg: &str) {
    if pin_to_core() {
        check_cpu_id(AFFINITY_CORE_ID as i32).expect(msg);
    }
}

fn apply_thread_params(msg: &str) {
    if pin_to_core() {
        set_for_current(core_affinity::CoreId { id: AFFINITY_CORE_ID });
        check_thread_params(msg);
    }
}
fn apply_main_thread_params() {
    if pin_to_core() {
        set_for_current(core_affinity::CoreId { id: 0 });
    }
}

trait PerfPool {
    async fn init(&self, cores_count: u32) -> TonCoreResult<()>;
    async fn do_task(&self, task: Task) -> TonCoreResult<u64>;
}

#[derive(Clone)]
enum Task {
    // TokioSleep {
    //     run_time: u64,
    //     // tx: Option<oneshot::Sender<Result<u64, TonError>>>,
    // },
    CpuFullLoad {
        run_time: u64,
        // tx: Option<oneshot::Sender<Result<u64, TonError>>>,
    },
    StdSleep {
        run_time: u64,
        // tx: Option<oneshot::Sender<Result<u64, TonError>>>,
    },
    TxEmulOrd(TXEmulOrdArgs),
}
fn fibonachi(n: u64) -> u64 {
    if n == 0 {
        return 0;
    } else if n == 1 {
        return 1;
    }
    fibonachi(n - 1) + fibonachi(n - 2)
}

struct CpuLoadObject {
    duration: Duration,
    tx_emulator: TXEmulator,
}

impl CpuLoadObject {
    fn new(d: u64) -> TonCoreResult<Self> {
        Ok(Self {
            duration: Duration::from_micros(d),
            tx_emulator: TXEmulator::new(0, false)?,
        })
    }

    fn do_task(&mut self, task: &Task) -> TonCoreResult<u64> {
        let run_time_cycles: u64 = rdtsc();
        check_thread_params("CpuLoadObject::do_task");
        let sleep_microseconds = match task {
            Task::CpuFullLoad { run_time, .. } => {
                cpu_load_function(*run_time);
                *run_time
            }
            Task::StdSleep { run_time, .. } => {
                std::thread::sleep(self.duration);
                *run_time
            }
            Task::TxEmulOrd(emul_args) => {
                let _res = self.tx_emulator.emulate_ord(&emul_args.clone())?;
                DEFAULT_SLEEP_TIME_MICROS
            }
        };
        if Duration::from_micros(sleep_microseconds) != self.duration {
            panic!(
                "{}",
                format!(
                    "Expected sleep duration {:?}, got {:?}",
                    self.duration,
                    Duration::from_micros(sleep_microseconds)
                )
            );
        }

        let total_time = rdtsc() - run_time_cycles;

        Ok(total_time)
    }
}

enum Job {
    Execute(Task, oneshot::Sender<TonCoreResult<u64>>),
    PrintStats,
    Stop,
}

struct Inner {
    senders: Vec<Sender<Job>>,
    c_sended: AtomicUsize,
    handles: Mutex<Vec<thread::JoinHandle<()>>>,
}

fn worker_loop(rx: Receiver<Job>, id: u32) -> TonCoreResult<u64> {
    check_thread_params("worker_loop");
    let mut cnt_sleep: u64 = 0;
    let mut cnt_cpu: u64 = 0;
    let mut cnt_ord: u64 = 0;
    let mut obj = CpuLoadObject::new(DEFAULT_SLEEP_TIME_MICROS)?;
    while let Ok(job) = rx.recv() {
        match job {
            Job::Execute(task, tx) => {
                let res = obj.do_task(&task);
                match task {
                    Task::CpuFullLoad { .. } => {
                        cnt_cpu += 1;
                    }
                    Task::StdSleep { .. } => {
                        cnt_sleep += 1;
                    }
                    Task::TxEmulOrd(_) => {
                        cnt_ord += 1;
                    }
                }

                let _ = tx.send(res);
            }
            Job::PrintStats => {
                println!("Worker PrintStats {id} processed Cpu {cnt_cpu}, sleeps:{cnt_sleep}  ");
            }
            Job::Stop => {
                break;
            }
        }
    }

    Ok(cnt_sleep + cnt_cpu)
}

impl Inner {
    fn new(threads: usize) -> Self {
        let mut senders = Vec::with_capacity(threads);
        let mut handles = Vec::with_capacity(threads);

        for i in 0..threads {
            let (tx, rx): (Sender<Job>, Receiver<Job>) = mpsc::channel();
            senders.push(tx);

            let handle = thread::Builder::new()
                .name(format!("cpu-pool-{i}"))
                .spawn(move || {
                    apply_thread_params("Inner worker_loop");
                    worker_loop(rx, i as u32).expect("worker loop failed");
                })
                .expect("spawn worker");
            handles.push(handle);
        }
        Self {
            senders,
            c_sended: AtomicUsize::new(0),
            handles: Mutex::new(handles),
        }
    }
    async fn execute_task(&self, task: Task) -> TonCoreResult<u64> {
        let (tx, rx) = oneshot::channel();
        let idx = self.c_sended.fetch_add(1, Ordering::Relaxed) % self.senders.len();
        self.senders[idx]
            .send(Job::Execute(task, tx))
            .map_err(|e| TonCoreError::Custom(format!("send task error: {e}")))?;
        let res = rx.await.map_err(|e| TonCoreError::Custom(format!("receive task error: {e}")))?;
        // self.c_completed.fetch_add(1, Ordering::SeqCst);
        res
    }

    fn print_it(&self) -> TonCoreResult<()> {
        for sender in &self.senders {
            sender.send(Job::PrintStats).map_err(|e| TonCoreError::Custom(format!("send stoperror: {e}")))?;
        }
        println!("sended: {}", self.c_sended.load(Ordering::Relaxed));
        Ok(())
    }

    fn stop(&self) {
        for sender in &self.senders {
            let _ = sender.send(Job::Stop);
        }
        if let Ok(mut handles) = self.handles.lock() {
            for handle in handles.drain(..) {
                let _ = handle.join();
            }
        }
    }
}

pub struct CpuThreadPoolPerf {
    inner: OnceLock<Inner>,
}
impl CpuThreadPoolPerf {
    pub const fn new() -> Self { Self { inner: OnceLock::new() } }

    // Create threads here. Async for API symmetry; it doesn't await anything internally.
}
impl PerfPool for CpuThreadPoolPerf {
    async fn init(&self, cores_count: u32) -> TonCoreResult<()> {
        let _ = self.inner.get_or_init(|| Inner::new(cores_count as usize));
        if self.inner.get().ok_or(TonCoreError::Custom(" ".to_string()))?.senders.len() != cores_count as usize {
            panic!("Pool::init called multiple times with different cores_count");
        }
        Ok(())
    }

    async fn do_task(&self, task: Task) -> TonCoreResult<u64> {
        let inner = self.inner.get().expect("Pool::init must be called first");
        inner.execute_task(task).await
    }
}

impl CpuThreadPoolPerf {
    pub fn print_it(&self) {
        if let Some(inner) = self.inner.get() {
            let _ = inner.print_it();
        }
    }

    pub fn stop(&self) {
        if let Some(inner) = self.inner.get() {
            inner.stop();
        }
    }
}

impl Drop for CpuThreadPoolPerf {
    fn drop(&mut self) { self.stop(); }
}

static POOL: LazyLock<CpuThreadPoolPerf> = LazyLock::new(CpuThreadPoolPerf::new);

async fn run_pool_test<P: PerfPool>(pool: &P, task: &Task) -> TonCoreResult<u64> {
    let start_time = rdtsc();
    let mut answer_keeper = Vec::new();
    answer_keeper.reserve(total_requests() as usize);
    for _ in 0..total_requests() {
        answer_keeper.push(pool.do_task(task.clone()));
    }
    let request_time = rdtsc();
    let mut rv = 0;
    let answer = join_all(answer_keeper).await;
    for res in answer {
        let val = res?;
        black_box(val);
        rv += val;
    }
    let end_time = rdtsc();
    let rt = request_time - start_time;
    let wt = end_time - request_time;
    if rt > wt {
        panic!("run_pool_test: invalid timing rt={} > wt={}", rt, wt);
    }
    Ok(rv)
}
async fn pool_spawn_sleep() -> TonCoreResult<()> {
    POOL.init(threads_count()).await?;

    apply_main_thread_params();
    let task = Task::StdSleep {
        run_time: DEFAULT_SLEEP_TIME_MICROS,
    };
    run_pool_test(LazyLock::<CpuThreadPoolPerf>::force(&POOL), &task).await?;
    Ok(())
}
async fn pool_spawn_cpu_load() -> TonCoreResult<()> {
    POOL.init(threads_count()).await?;
    // apply_main_thread_params();

    let task = Task::CpuFullLoad {
        run_time: DEFAULT_SLEEP_TIME_MICROS,
    };
    run_pool_test(LazyLock::<CpuThreadPoolPerf>::force(&POOL), &task).await?;

    Ok(())
}

fn simple_thread_function(task: &Task, count: u32) -> TonCoreResult<u64> {
    check_thread_params("simple_thread_function");
    let mut rv: u64 = 0;
    let mut obj = CpuLoadObject::new(DEFAULT_SLEEP_TIME_MICROS)?;
    for _ in 0..count {
        let val = obj.do_task(task)?;
        black_box(val);
        rv += val;
    }
    Ok(rv)
}

async fn simple_async_thread_function(task: &Task, count: u32) -> TonCoreResult<u64> {
    check_thread_params("simple_thread_function");
    let mut rv: u64 = 0;
    let mut obj = CpuLoadObject::new(DEFAULT_SLEEP_TIME_MICROS)?;
    for _ in 0..count {
        let val = obj.do_task(task)?;
        black_box(val);
        rv += val;
    }
    Ok(rv)
}

use ton::errors::TonResult;
use ton::sys_utils::sys_tonlib_set_verbosity_level;
use ton_core::cell::TonHash;

async fn bl_threads_sleep() -> TonCoreResult<()> {
    apply_main_thread_params();
    let mut threads = Vec::new();
    let task = Task::StdSleep {
        run_time: DEFAULT_SLEEP_TIME_MICROS,
    };

    for _i in 0..threads_count() {
        let task_clone = task.clone();
        let handler = thread::spawn(move || {
            apply_thread_params("bl_threads_sleep");
            simple_thread_function(&task_clone, total_requests() / threads_count()).expect("worker loop failed")
        });
        threads.push(handler);
    }
    for t in threads {
        t.join().expect("worker thread failed");
    }
    Ok(())
}

async fn bl_threads_cpu_load() -> TonCoreResult<()> {
    apply_main_thread_params();
    let mut threads = Vec::new();
    let task = Task::CpuFullLoad {
        run_time: DEFAULT_SLEEP_TIME_MICROS,
    };

    for _i in 0..threads_count() {
        let task_clone = task.clone();
        let handler = thread::spawn(move || {
            apply_thread_params("bl_threads_sleep");
            simple_thread_function(&task_clone, total_requests() / threads_count()).expect("worker loop failed")
        });
        threads.push(handler);
    }
    for t in threads {
        t.join().expect("worker thread failed");
    }
    Ok(())
}

async fn baseline_pseudo_pool_sleep() -> TonCoreResult<()> {
    apply_thread_params("baseline_pseudo_pool_yeld");
    let mut obj = CpuLoadObject::new(DEFAULT_SLEEP_TIME_MICROS)?;
    let task = Task::StdSleep {
        run_time: DEFAULT_SLEEP_TIME_MICROS,
    };
    for _ in 0..total_requests() {
        obj.do_task(&task).unwrap();
    }

    Ok(())
}
async fn baseline_pseudo_pool_cpu_load() -> TonCoreResult<()> {
    apply_thread_params("baseline_pseudo_pool_cpu_load");

    let mut obj = CpuLoadObject::new(DEFAULT_SLEEP_TIME_MICROS)?;
    let task = Task::CpuFullLoad {
        run_time: DEFAULT_SLEEP_TIME_MICROS,
    };
    for _ in 0..total_requests() {
        let _ = obj.do_task(&task);
    }

    Ok(())
}

async fn baseline_pseudo_pool_emulate() -> TonCoreResult<()> {
    apply_thread_params("baseline_pseudo_pool_emulate");
    let mut obj = CpuLoadObject::new(DEFAULT_SLEEP_TIME_MICROS)?;
    let arg = get_test_args().unwrap();
    let task = Task::TxEmulOrd(arg);
    for _ in 0..total_requests() {
        obj.do_task(&task);
    }

    Ok(())
}

async fn pool_spawn_emulate() -> TonCoreResult<()> {
    POOL.init(threads_count()).await?;
    // apply_main_thread_params();
    let arg = get_test_args().unwrap();

    let task = Task::TxEmulOrd(arg);
    run_pool_test(LazyLock::<CpuThreadPoolPerf>::force(&POOL), &task).await?;

    Ok(())
}

fn benchmark_functions(c: &mut Criterion) {
    // Create a runtime for async benchmarks
    let rt = Runtime::new().expect("Failed to create tokio runtime");

    // c.bench_function("baseline_sleep x+k,  where k - bench overhead", |b| b.iter(|| rt.block_on(sync_baseline_sleep_bench()).unwrap()));

    // c.bench_function("baseline_sleep 2x+k, where k - bench overhead", |b| {
    //     b.iter(|| rt.block_on(sync_baseline_sleep_bench_twise()).unwrap())
    // });

    // c.bench_function("threads_yelded_sleep_bench", |b| b.iter(|| rt.block_on(threads_yelded_sleep_bench()).unwrap()));
    // c.bench_function("threads_cpu_load_bench", |b| b.iter(|| rt.block_on(threads_cpu_load_bench()).unwrap()));
    // c.bench_function("threads_std_sleep", |b| b.iter(|| rt.block_on(threads_std_sleep()).unwrap()));
    // c.bench_function("spawn_bloking_yelded_sleep_bench", |b| b.iter(|| rt.block_on(spawn_bloking_yelded_sleep_bench()).unwrap()));
    // c.bench_function("baseline_pseudo_pool_sleep", |b| b.iter(|| rt.block_on(baseline_pseudo_pool_sleep()).unwrap()));
    // c.bench_function("baseline_pseudo_pool_emulate", |b| b.iter(|| rt.block_on(baseline_pseudo_pool_emulate()).unwrap()));
    // c.bench_function("baseline_pseudo_pool_cpu_load", |b| {
    //     b.iter(|| rt.block_on(baseline_pseudo_pool_cpu_load()).unwrap())
    // });
    // c.bench_function("bl_threads_cpu_load", |b| b.iter(|| rt.block_on(bl_threads_cpu_load()).unwrap()));
    // c.bench_function("bl_threads_sleep", |b| b.iter(|| rt.block_on(bl_threads_sleep()).unwrap()));
    // c.bench_function("pool_spawn_cpu_load", |b| b.iter(|| rt.block_on(pool_spawn_cpu_load()).unwrap()));
    c.bench_function("pool_spawn_emulate", |b| b.iter(|| rt.block_on(pool_spawn_emulate()).unwrap()));

    // c.bench_function("pool_spawn_yeld", |b| b.iter(|| rt.block_on(pool_spawn_yeld()).unwrap()));
}

fn main() {
    let mut criterion = configure_criterion();
    benchmark_functions(&mut criterion);
    POOL.print_it();
    criterion.final_summary();
    let aval = std::thread::available_parallelism();
    println!("Available parallelism: {:?}", aval);
}
