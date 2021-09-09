use ic_cdk::export::candid::{CandidType, Deserialize, Nat, Principal, Encode, Decode};
use std::process::Command;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};
use ic_agent::Agent;
use tokio::task;
use std::thread;
use std::sync::Arc;

const CREATE_CHUNK: &str = "create_chunk";
const CAP: usize = 512;

#[derive(CandidType, Debug, Deserialize)]
struct CreateChunkRequest<'a> {
    key: String,
    #[serde(with = "serde_bytes")]
    content: &'a [u8],
}

#[derive(CandidType, Debug, Deserialize)]
struct CreateChunkResponse {
    chunk_id: Nat,
}

#[derive(CandidType, Debug, Deserialize)]
struct GetChunkRequest {
    key: String,
    index: Nat,
}


#[derive(CandidType, Debug, Deserialize)]
struct GetChunkResponse {
    content: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frontend_url = "http://127.0.0.1:8000";
    let canister_id = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")?;
    let agent = Agent::builder()
    .with_transport(
        ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(frontend_url)?,
    )
    .build()?;
    let arc_agent = Arc::new(agent);

    let args = GetChunkRequest { key: "test".to_string(), index: Nat::from(0) };
    let args = Encode!(&args).expect("ENCODING FAILED");
    let transport = ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(FRONTEND_URL).expect("Couldn't build transport.");
    let result = transport.query(Principal::from_str(CANISTER), args);
    println!("{:?}", result);


    let file = File::open("netboot.tar.gz")?;
    let mut reader = BufReader::with_capacity(CAP, file);
    let mut counter = 0;

    loop {
        println!("{:?}", counter);
        let arc_agent_clone = arc_agent.clone();
        let length = {
            let buffer = reader.fill_buf()?;
            // do stuff with buffer here
            let move_buffer = buffer.clone();
            
            create_chunk(&arc_agent_clone, &canister_id.clone(), "ubuntu".to_string(), Nat::from(counter), move_buffer);
            //println!("{:?}", response);
            buffer.len()
        };
        if length == 0 {
            break;
        }
        counter += 1;
        reader.consume(length);
    }
    Ok(())
}

fn create_chunk(
    agent: &Agent,
    canister_id: &Principal,
    key: String,
    index: Nat,
    content: &[u8],
) -> CreateChunkResponse {
    let args = CreateChunkRequest { key, content };
   let args = Encode!(&args)?;
    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();



    let response = agent
        .update(&canister_id, CREATE_CHUNK)
        .with_arg(&args)
        .call_and_wait(waiter)
        .await?;
    let result = Decode!(&response, CreateChunkResponse)?;
    println!("{:?}", result);
    Ok(result) 
}

