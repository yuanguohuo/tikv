// Copyright 2017 TiKV Project Authors. Licensed under Apache-2.0.

pub use self::imp::wait_for_signal;

#[cfg(unix)]
mod imp {
    use engine_rocks::RocksEngine;
    use engine_traits::{KvEngines, MiscExt};
    use libc::c_int;
    use nix::sys::signal::{SIGHUP, SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
    use signal::trap::Trap;
    use tikv_util::metrics;

    #[allow(dead_code)]
    pub fn wait_for_signal(engines: Option<KvEngines<RocksEngine, RocksEngine>>) {
        let trap = Trap::trap(&[SIGTERM, SIGINT, SIGHUP, SIGUSR1, SIGUSR2]);
        for sig in trap {
            match sig {
                SIGTERM | SIGINT | SIGHUP => {
                    info!("receive signal {}, stopping server...", sig as c_int);
                    break;
                }
                SIGUSR1 => {
                    // Use SIGUSR1 to log metrics.
                    info!("{}", metrics::dump());
                    if let Some(ref engines) = engines {
                        info!("{:?}", engines.kv.dump_stats());
                        info!("{:?}", engines.raft.dump_stats());
                    }
                }
                // TODO: handle more signal
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use engine_rocks::RocksEngine;
    use engine_traits::KvEngines;

    pub fn wait_for_signal(_: Option<KvEngines<RocksEngine, RocksEngine>>) {}
}
