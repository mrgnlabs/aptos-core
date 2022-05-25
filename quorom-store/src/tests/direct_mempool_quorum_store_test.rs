// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::direct_mempool_quorum_store::start_direct_mempool_quorum_store;
use aptos_mempool::{QuorumStoreRequest, QuorumStoreResponse};
use consensus_types::{
    common::{Payload, PayloadFilter},
    request_response::{ConsensusRequest, ConsensusResponse},
};
use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use std::time::Duration;
use tokio::{runtime::Handle, time::timeout};

#[tokio::test(flavor = "multi_thread")]
async fn test_block_request_no_txns() {
    let (quorum_store_to_mempool_sender, mut quorum_store_to_mempool_receiver) =
        mpsc::channel(1_024);
    let (mut consensus_to_quorum_store_sender, consensus_to_quorum_store_receiver) =
        mpsc::channel(1_024);
    let (consensus_callback, consensus_callback_rcv) = oneshot::channel();
    let _quorum_store = start_direct_mempool_quorum_store(
        &Handle::current(),
        consensus_to_quorum_store_receiver,
        quorum_store_to_mempool_sender,
        10_000,
    );

    consensus_to_quorum_store_sender
        .try_send(ConsensusRequest::GetBlockRequest(
            100,
            PayloadFilter::InMemory(vec![]),
            consensus_callback,
        ))
        .unwrap();

    let QuorumStoreRequest::GetBatchRequest(_max_batch_size, _exclude_txns, callback) = timeout(
        Duration::from_millis(1_000),
        quorum_store_to_mempool_receiver.select_next_some(),
    )
    .await
    .unwrap();

    callback
        .send(Ok(QuorumStoreResponse::GetBatchResponse(vec![])))
        .unwrap();

    let ConsensusResponse::GetBlockResponse(payload) =
        timeout(Duration::from_millis(1_000), consensus_callback_rcv)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    assert!(payload.is_empty());
    match payload {
        Payload::DirectMempool(txns) => assert!(txns.is_empty()),
        _ => panic!("Unexpected payload {:?}", payload),
    }
}
