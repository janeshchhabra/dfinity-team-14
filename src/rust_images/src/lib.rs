mod rc_bytes;

use crate::rc_bytes::RcBytes;
use ic_cdk::api::trap;
use ic_cdk::export::candid::{CandidType, Deserialize, Nat};
use ic_cdk_macros::*;
use serde_bytes::ByteBuf;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static STATE: State = State::default();
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct State {
    files:  RefCell<HashMap<Key, HashMap<ChunkId, RcBytes>>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CreateChunkArg {
    key: Key,
    index: ChunkId,
    content: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CreateChunkResponse {
    chunk_id: ChunkId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GetChunkArg {
    key: Key,
    index: ChunkId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GetChunkResponse {
    content: RcBytes,
}

type ChunkId = Nat;
type Key = String;


#[query]
fn get_chunk(arg: GetChunkArg) -> GetChunkResponse {
    STATE.with(|s| {
        let files = s.files.borrow();
        let chunks = files.get(&arg.key).unwrap_or_else(|| trap("Index doesn't exist"));
        let chunk = chunks.get(&arg.index).unwrap_or_else(|| trap("Index doesn't exist"));
        GetChunkResponse { content: chunk.clone() }
    })
}

#[update]
fn create_chunk(arg: CreateChunkArg) -> CreateChunkResponse {
    STATE.with(|s| {
        let mut asset_under_construction = s.files.borrow_mut();
        if !asset_under_construction.contains_key(&arg.key) {
            asset_under_construction.insert(arg.key.clone(), HashMap::new());
        }
        let key = asset_under_construction.get_mut(&arg.key).unwrap_or_else(|| trap("Couldn't create chunk"));
        key.insert( arg.index.clone(), RcBytes::from(arg.content));
        CreateChunkResponse { chunk_id: arg.index.clone() }
    })
}
