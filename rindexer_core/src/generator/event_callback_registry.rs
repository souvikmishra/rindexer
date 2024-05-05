use std::{any::Any, sync::Arc};

use futures::future::BoxFuture;

use ethers::addressbook::Address;
use ethers::prelude::{RetryClient, U256, U64};
use ethers::{
    providers::{Http, Provider},
    types::{Bytes, Log, H256},
};

type Decoder = Arc<dyn Fn(Vec<H256>, Bytes) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

#[derive(Clone)]
pub struct NetworkContract {
    pub network: String,

    pub address: String,

    pub provider: &'static Arc<Provider<RetryClient<Http>>>,

    pub decoder: Decoder,

    pub start_block: Option<u64>,

    pub end_block: Option<u64>,

    pub polling_every: Option<u64>,
}

impl NetworkContract {
    pub fn decode_log(&self, log: Log) -> Arc<dyn Any + Send + Sync> {
        (self.decoder)(log.topics, log.data)
    }
}

#[derive(Clone)]
pub struct ContractInformation {
    pub name: String,
    pub details: Vec<NetworkContract>,
    pub abi: String,
}

#[derive(Debug)]
pub struct TxInformation {
    pub network: String,

    pub address: Address,

    pub block_hash: Option<H256>,

    pub block_number: Option<U64>,

    pub transaction_hash: Option<H256>,

    pub transaction_index: Option<U64>,

    pub log_index: Option<U256>,

    pub transaction_log_index: Option<U256>,

    pub log_type: Option<String>,

    pub removed: Option<bool>,
}

pub struct EventResult {
    pub decoded_data: Arc<dyn Any + Send + Sync>,
    pub tx_information: TxInformation,
}

impl EventResult {
    pub fn new(network_contract: Arc<NetworkContract>, log: &Log) -> Self {
        Self {
            decoded_data: network_contract.decode_log(log.clone()),
            tx_information: TxInformation {
                network: network_contract.network.to_string(),
                address: log.address,
                block_hash: log.block_hash,
                block_number: log.block_number,
                transaction_hash: log.transaction_hash,
                transaction_index: log.transaction_index,
                log_index: log.log_index,
                transaction_log_index: log.transaction_log_index,
                log_type: log.log_type.clone(),
                removed: log.removed,
            },
        }
    }
}

pub struct EventInformation {
    pub topic_id: &'static str,
    pub contract: ContractInformation,
    pub callback: Arc<dyn Fn(Vec<EventResult>) -> BoxFuture<'static, ()> + Send + Sync>,
}

impl Clone for EventInformation {
    fn clone(&self) -> Self {
        EventInformation {
            topic_id: self.topic_id,
            contract: self.contract.clone(),
            callback: Arc::clone(&self.callback),
        }
    }
}

#[derive(Clone)]
pub struct EventCallbackRegistry {
    pub events: Vec<EventInformation>,
}

impl EventCallbackRegistry {
    pub fn new() -> Self {
        EventCallbackRegistry { events: Vec::new() }
    }

    pub fn find_event(&self, topic_id: &'static str) -> Option<&EventInformation> {
        self.events.iter().find(|e| e.topic_id == topic_id)
    }

    pub fn register_event(&mut self, event: EventInformation) {
        self.events.push(event);
    }

    pub async fn trigger_event(&self, topic_id: &'static str, data: Vec<EventResult>) {
        if let Some(callback) = self.find_event(topic_id).map(|e| &e.callback) {
            callback(data).await;
        } else {
            println!(
                "EventCallbackRegistry: No event found for topic_id: {}",
                topic_id
            );
        }
    }

    pub fn complete(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }
}
