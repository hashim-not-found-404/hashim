use crate::client::Cache;
use crate::new_types::NonceUuid;
use crate::new_types::UserUuid;
use crate::new_types::UuidType;
use crate::request_response::FromClient;
use crate::request_response::FromServer;
use crate::types::ADDRESS;
use crate::types::HashimError;
use anyhow::Result;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::encode_decode::Coding;
use infrastructure::encode_decode::Ed;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::RwLock;
use utility::cache::CacheCaster;
use utility::cache::CacheStruct;
use utility::cache::CacheUtility;
use utility::cache::MessageFromServer;
use utility::cache::MessageToCache;
use utility::cache::OpError;
use utility::cache::OpErrorTrait;
use utility::cache::OpInput;
use utility::cache::OpInputTrait;
use utility::cache::OpOk;
use utility::cache::OpOkTrait;
use utility::dtos::Txn;
use utility::dtos::TxnNumber;
use utility::dtos::TypeOperationsError;
use utility::dtos::TypeOperationsInput;
use utility::dtos::TypeOperationsOk;
use utility::network::Network;
use utility::network::network_actor;
use utility::process_manager::process_manager_actor;
use utility::types::ReadAndSet;
use utility::ui_effect::Caster;
use utility::ui_effect::Commander;
use utility::ui_effect::Model;
use utility_ui::domain::HashimSignal;

pub fn new<Ch, Cu, Mdl, Cas, CasCh>(model: Arc<Mdl>) -> Commander
where
    Ch: Cache + 'static,
    Cu: CacheUtility<Cache = Ch> + 'static,
    Mdl: Model,
    Cas: Caster,
    CasCh: CacheCaster<Cache = Ch>,
{
    let (sender_to_network, receiver_to_network) = Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = Mpsc::channel();
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::<_>(
        receiver_to_network,
        sender_to_error.clone(),
        MyNetwork {
            sender_to_cache: sender_to_cache.clone(),
            is_online:       is_online.clone(),
        },
        format!("ws://{}/ws", ADDRESS),
    );

    let cache = CacheStruct::new::<Ch, Cu, CasCh>(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager_actor();

    Commander::new::<Mdl, Cas>(sender_to_process_manager, model, cache)
}

struct MyNetwork {
    sender_to_cache: MpscSender<MessageToCache>,
    is_online:       Arc<RwLock<bool>>,
}

impl Network for MyNetwork {
    async fn network_state(&mut self, is_online: bool) {
        self.is_online.put(is_online);

        if is_online {
            self.sender_to_cache.send(MessageToCache::WeAreBackOnline).await.unwrap();
        }
    }

    async fn network_sender(&mut self, data: Vec<u8>) {
        self.sender_to_cache.send(MessageToCache::DataFromServer(data)).await.unwrap();
    }
}

pub trait Casting: 'static {
    fn cast_input<Ch: Cache>(v: TypeOperationsInput) -> Box<dyn OpInputTrait>;
    fn cast_ok<Ch: Cache>(v: TypeOperationsOk) -> Box<dyn OpOkTrait>;
    fn cast_error(v: TypeOperationsError) -> Box<dyn OpErrorTrait>;
}

struct MyCache<Ch: Cache, Cas: Casting> {
    cache: Ch,
    _ph:   PhantomData<Cas>,
}

impl<Ch, Cas> CacheUtility for MyCache<Ch, Cas>
where
    Ch: Cache,
    Cas: Casting,
{
    type Cache = Ch;

    async fn new() -> Self {
        Self {
            cache: Ch::new().await,
            _ph:   PhantomData,
        }
    }

    fn get_inner_cache(&mut self) -> &mut Self::Cache {
        &mut self.cache
    }

    async fn get_all_pending_txn(&mut self) -> Vec<Txn<OpInput>> {
        let all_txns = self.cache.get_all_pending_txn().await;
        let mut vec_to_return = Vec::new();

        for Txn {
            txn_number,
            operation,
        } in all_txns
        {
            let operation: TypeOperationsInput = Ed::decode(&operation).unwrap();
            let operation = Cas::cast_input::<Self::Cache>(operation);
            let operation = Arc::from(operation);
            let operation = OpInput(operation);

            let txn = Txn {
                txn_number,
                operation,
            };

            vec_to_return.push(txn);
        }

        vec_to_return
    }

    async fn clear_pending_txn_state(&mut self) {
        self.cache.clear_pending_txn_state().await
    }

    async fn start_pending_txn_state(&mut self) {
        self.cache.start_pending_txn_state().await
    }

    async fn delete_input_txn(&mut self, txn_number: TxnNumber) {
        self.cache.delete_input_txn(txn_number).await
    }

    async fn mark_input_txn_as_faild(&mut self, txn_number: TxnNumber) {
        self.cache.mark_input_txn_as_faild(txn_number).await
    }

    async fn write_input_to_cache(&mut self, txn_number: TxnNumber, input: OpInput) {
        let operation = input.0.clone();
        let operation = dyn_clone::clone_box(&*operation);
        let operation: TypeOperationsInput = operation;
        let operation = Ed::encode(&operation);

        self.cache
            .write_txn_input(&Txn {
                txn_number,
                operation,
            })
            .await
    }

    async fn write_error_to_cache(&mut self, txn_number: TxnNumber, error: OpError) {
        let operation = error.0;
        let operation = dyn_clone::clone_box(&*operation);
        let operation: TypeOperationsError = operation;
        let operation = Ed::encode(&operation);

        self.cache
            .write_txn_result(&Txn {
                txn_number,
                operation,
            })
            .await
    }

    async fn encode_the_inputs(&mut self, inputs: Vec<Txn<OpInput>>) -> Vec<u8> {
        let mut jwts = Vec::new();

        for i in &inputs {
            if let Some(user_uuid) = i.operation.0.user_uuid() {
                let user_uuid1 = UserUuid::from(UuidType::from(user_uuid));

                if let Some(jwt) = self.cache.get_jwt(&user_uuid1).await {
                    jwts.push(jwt)
                }
            }
        }

        let mut operations1 = Vec::with_capacity(inputs.len());

        for i in inputs {
            let operation = i.operation.0;
            let operation = dyn_clone::clone_box(&*operation);
            let operation: TypeOperationsInput = operation;

            let value = Txn {
                txn_number: i.txn_number,
                operation,
            };

            operations1.push(value);
        }

        let data = FromClient {
            jwts,
            nonce: NonceUuid::from(UuidType::from(Id::generate())),
            operations: operations1,
        };

        Ed::encode(&data)
    }

    fn decode_the_response(resp: Vec<u8>) -> Result<MessageFromServer> {
        let resp: FromServer = Ed::decode(&resp).unwrap();

        let resp = match resp {
            FromServer::Error(i) => MessageFromServer::Error(i.into()),
            FromServer::PushData(i) => {
                let mut operations = Vec::new();

                for i in i.operations {
                    let a = i.operation;
                    let r = match a {
                        Ok(ok) => {
                            let ok = Cas::cast_ok::<Ch>(ok);
                            Ok(OpOk(ok))
                        }
                        Err(err) => {
                            let err = Cas::cast_error(err);
                            Err(OpError(err))
                        }
                    };

                    operations.push(Txn {
                        txn_number: i.txn_number,
                        operation:  r,
                    });
                }

                MessageFromServer::Response(operations)
            }
            FromServer::Resources(i) => {
                let mut a = Vec::new();

                for i in i {
                    let v = Cas::cast_ok::<Ch>(i);
                    a.push(OpOk(v));
                }

                MessageFromServer::Resources(a)
            }
        };

        Ok(resp)
    }
}

fn listen_to_error_actor(
    mut receiver_to_error: MpscReceiver<HashimError>,
    external_errors_signal: impl HashimSignal<String>,
) {
    Rt::spawn_local(async move {
        loop {
            let err = receiver_to_error.recv().await.unwrap();
            external_errors_signal.set(err.to_string());
        }
    });
}
