// Copyright 2018 TiKV Project Authors. Licensed under Apache-2.0.

use std::sync::{Arc, Mutex};

use engine_rocks::RocksSnapshot;
use kvproto::raft_cmdpb::RaftCmdRequest;
use kvproto::raft_serverpb::RaftMessage;
use raftstore::errors::{Error as RaftStoreError, Result as RaftStoreResult};
use raftstore::router::{handle_send_error, RaftStoreRouter};
use raftstore::store::msg::{Callback, CasualMessage, PeerMsg, SignificantMsg};
use tikv_util::collections::HashMap;
use tikv_util::mpsc::{loose_bounded, LooseBoundedSender, Receiver};
use txn_types::TxnExtra;

#[derive(Clone)]
pub struct MockRaftStoreRouter {
    senders: Arc<Mutex<HashMap<u64, LooseBoundedSender<PeerMsg<RocksSnapshot>>>>>,
}

impl MockRaftStoreRouter {
    pub fn new() -> MockRaftStoreRouter {
        MockRaftStoreRouter {
            senders: Arc::default(),
        }
    }
    pub fn add_region(&self, region_id: u64, cap: usize) -> Receiver<PeerMsg<RocksSnapshot>> {
        let (tx, rx) = loose_bounded(cap);
        self.senders.lock().unwrap().insert(region_id, tx);
        rx
    }
}

impl RaftStoreRouter<RocksSnapshot> for MockRaftStoreRouter {
    fn significant_send(&self, region_id: u64, msg: SignificantMsg) -> RaftStoreResult<()> {
        let mut senders = self.senders.lock().unwrap();
        if let Some(tx) = senders.get_mut(&region_id) {
            tx.force_send(PeerMsg::SignificantMsg(msg)).unwrap();
            Ok(())
        } else {
            error!("failed to send significant msg"; "msg" => ?msg);
            Err(RaftStoreError::RegionNotFound(region_id))
        }
    }

    fn casual_send(
        &self,
        region_id: u64,
        msg: CasualMessage<RocksSnapshot>,
    ) -> RaftStoreResult<()> {
        let mut senders = self.senders.lock().unwrap();
        if let Some(tx) = senders.get_mut(&region_id) {
            tx.try_send(PeerMsg::CasualMessage(msg))
                .map_err(|e| handle_send_error(region_id, e))
        } else {
            Err(RaftStoreError::RegionNotFound(region_id))
        }
    }

    fn send_raft_msg(&self, _: RaftMessage) -> RaftStoreResult<()> {
        unimplemented!()
    }
    fn send_command(&self, _: RaftCmdRequest, _: Callback<RocksSnapshot>) -> RaftStoreResult<()> {
        unimplemented!()
    }
    fn send_command_txn_extra(
        &self,
        _: RaftCmdRequest,
        _: TxnExtra,
        _: Callback<RocksSnapshot>,
    ) -> RaftStoreResult<()> {
        unimplemented!()
    }
    fn broadcast_unreachable(&self, _: u64) {
        unimplemented!()
    }
}
