use crate::client::cache_op;
use crate::client::cache_op::DbBundle;
use crate::client::network_actor::Network;
use crate::client::network_actor::WSClient;
use crate::client::network_actor::network_actor;
use crate::client::ui_effect::Commander;
use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor::CacheActorUtils;
use crate::client::utility::cache_actor::MessageFromServer;
use crate::client::utility::cache_actor::MessageToCache;
use crate::client::utility::client_traits::CacheActorStruct;
use crate::client::utility::client_traits::OperationName;
use crate::client::utility::process_manager::process_manager_actor;
use crate::client::utility::ui_model::AllSignalTypes;
use crate::client::utility::ui_model::Model;
use crate::domain::request_response::FromClient;
use crate::domain::request_response::FromServer;
use crate::domain::request_response::MyResult;
use crate::domain::request_response::OperationsInput;
use crate::domain::request_response::OperationsOk;
use crate::domain::request_response::OperationsResult;
use crate::domain::request_response::ResourceDTO;
use crate::domain::request_response::Txn;
use crate::domain::utility::types::ADDRESS;
use crate::domain::utility::types::HashimError;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::Nonce;
use crate::utility::traits::Coding;
use crate::utility::traits::DynamicError;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::RandomNumber;
use crate::utility::traits::Receiver;
use crate::utility::traits::Regex;
use crate::utility::traits::Runtime;
use crate::utility::traits::Sender;
use crate::utility::traits::Time;
use crate::utility::utils::ReadAndSet;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::RwLock;

pub fn new<
    Rn: RandomNumber,
    Rt: Runtime,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    Ed: Coding,
    Rg: Regex,
    Ti: Time,
    Ch: Cache + 'static,
    Ws: WSClient,
    As: AllSignalTypes,
    Dbb: DbBundle<Ch>,
>(
    model: &'static Model<As>,
) -> Commander<Mpsc> {
    let (sender_to_network, receiver_to_network) = Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = Mpsc::channel();
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::<Rt, Ws, _>(
        MyNetwork::<Mpsc> {
            sender_to_cache: sender_to_cache.clone(),
            receiver_to_network,
            sender_to_error: sender_to_error.clone(),
            is_online: is_online.clone(),
        },
        format!("ws://{}/ws", ADDRESS),
    );

    let cache = CacheActorStruct::new::<Rt, Ed, MyCache<Mpsc, Ch, Id, Ti, Dbb>>(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager_actor::<Mpsc, As, Rt>();

    Commander::new::<As, Rt, Rn, Id, Ti, Ch, Dbb>(
        receiver_to_error,
        sender_to_process_manager,
        model,
        cache,
    )
}

struct MyNetwork<Mpsc: MultiProducerSingleConsumer> {
    sender_to_cache:
        Mpsc::Sender<MessageToCache<Mpsc, OperationName, OperationsInput, OperationsResult>>,
    receiver_to_network: Mpsc::Receiver<Vec<u8>>,
    sender_to_error:     Mpsc::Sender<HashimError>,
    is_online:           Arc<RwLock<bool>>,
}

impl<Mpsc: MultiProducerSingleConsumer> Network for MyNetwork<Mpsc> {
    async fn network_state(&mut self, is_online: bool) {
        self.is_online.put(is_online);

        if is_online {
            self.sender_to_cache.send(MessageToCache::WeAreBackOnline).await.unwrap();
        }
    }

    async fn network_sender(&mut self, data: Vec<u8>) {
        self.sender_to_cache.send(MessageToCache::DataFromServer(data)).await.unwrap();
    }

    async fn network_reciever(&mut self) -> Vec<u8> {
        self.receiver_to_network.recv().await.unwrap()
    }

    async fn send_error(&mut self, _: DynamicError) {
        self.sender_to_error.send(HashimError::ConnectionClosed).await.unwrap();
    }
}

struct MyCache<Mpsc: MultiProducerSingleConsumer, Ch: Cache, Id: RowId, Ti: Time, Dbb: DbBundle<Ch>>
{
    _ph: PhantomData<(Mpsc, Ch, Id, Ti, Dbb)>,
}

impl<Mpsc: MultiProducerSingleConsumer, Ch: Cache, Id: RowId, Ti: Time, Dbb: DbBundle<Ch>>
    CacheActorUtils for MyCache<Mpsc, Ch, Id, Ti, Dbb>
{
    type Cache = Ch;
    type ErrorFromServer = HashimError;
    type ErrorSender = Mpsc::Sender<HashimError>;
    type MessageFromServer<'de> = FromServer;
    type Mpsc = Mpsc;
    type NetworkSender = Mpsc::Sender<Vec<u8>>;
    type NetworkStatus = Arc<RwLock<bool>>;
    type OpInput = OperationsInput;
    type OpResult = OperationsResult;
    type ResourceFromServer = Vec<ResourceDTO>;
    type ResourceToStore = OperationsOk;
    type Response = MyResult;
    type SendingTxns = FromClient;
    type Subscribe = OperationName;

    async fn cache_receiver(
        receiver: &mut <Self::Mpsc as MultiProducerSingleConsumer>::Receiver<
            MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>,
        >,
    ) -> MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult> {
        receiver.recv().await.unwrap()
    }

    async fn send_to_network(sender: &mut Self::NetworkSender, data: Vec<u8>) {
        sender.send(data).await.unwrap();
    }

    async fn internal_server_error(sender: &mut Self::ErrorSender) {
        sender.send(HashimError::InternalServerError).await.unwrap();
    }

    async fn invalid_format_error(sender: &mut Self::ErrorSender) {
        sender.send(HashimError::InvalidDataFormat).await.unwrap();
    }

    async fn is_online(network_status: &Self::NetworkStatus) -> bool {
        network_status.read()
    }

    async fn new_cache() -> Self::Cache {
        cache_op::new::<Id, Ti, Dbb, Ch>().await
    }

    async fn get_all_pending_txn(cache: &Self::Cache) -> Vec<(u64, Self::OpInput)> {
        let txns = cache.get_all_txn_input().await;

        let mut result = Vec::with_capacity(txns.len());
        for txn in txns {
            result.push((txn.txn_number, txn.operation));
        }
        result
    }

    async fn clear_state_pending_txn(cache: &mut Self::Cache) {
        cache.clear_pending_txn_state().await;
    }

    async fn start_state_pending_txn(cache: &mut Self::Cache) {
        cache.start_pending_txn_state().await;
    }

    async fn prepare_txn_for_send(
        cache: &Self::Cache,
        txns: Vec<(u64, Self::OpInput)>,
    ) -> Self::SendingTxns {
        let mut jwts = Vec::new();
        for (_, txn) in &txns {
            if let Some(user_uuid) = txn.get_user_uuid::<Ti, Ch, Dbb>()
                && let Some(jwt) = cache.get_jwt(user_uuid).await
            {
                jwts.push(jwt)
            }
        }

        let mut operations1 = Vec::with_capacity(txns.len());

        for i in txns {
            operations1.push(Txn {
                txn_number: i.0,
                operation:  i.1,
            });
        }

        FromClient {
            jwts,
            nonce: Nonce(Id::generate()),
            operations: operations1,
        }
    }

    fn to(
        msg: Self::MessageFromServer<'_>,
    ) -> MessageFromServer<Self::ErrorFromServer, Self::Response, Self::ResourceFromServer> {
        match msg {
            FromServer::Error(a) => MessageFromServer::Error(a),
            FromServer::PushData(a) => MessageFromServer::Response(a),
            FromServer::Resources(a) => MessageFromServer::Resources(a),
        }
    }

    fn extract_resource_from_response(resp: &Self::Response) -> Vec<Self::ResourceToStore> {
        resp.operations.iter().flat_map(|a| a.operation.extract_resource()).collect()
    }

    fn extract_resource_from_result(data: &Self::OpResult) -> Option<Self::ResourceToStore> {
        data.extract_resource()
    }

    async fn write_resource_to_cache_from_server(
        cache: &mut Self::Cache,
        resource: &Self::ResourceToStore,
    ) {
        todo!()
    }

    async fn write_resource_to_cache_from_client(
        cache: &mut Self::Cache,
        resource: &Self::ResourceToStore,
    ) {
        todo!()
    }

    async fn delete_successful_txn_input(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if i.operation.is_ok() {
                cache.delete_txn_input(&i.txn_number).await;
            }
        }
    }

    async fn mark_txn_input_as_faild(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if !i.operation.is_ok() {
                cache.mark_txn_input_as_faild(&i.txn_number).await;
            }
        }
    }

    async fn write_faild_txn_result(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if !i.operation.is_ok() {
                cache.write_txn_result(i).await;
            }
        }
    }

    async fn get_all_response_txn_numbers(resp: &Self::Response) -> Vec<(u64, Self::OpResult)> {
        resp.operations.iter().map(|a| (a.txn_number, a.operation.clone())).collect()
    }

    async fn check_input(cache: &mut Self::Cache, data: &Self::OpInput) -> Self::OpResult {
        data.run_operation_check::<Id, Ti, Ch, Dbb>(cache).await
    }

    async fn write_input(cache: &Self::Cache, txn_number: u64, data: &Self::OpInput) {
        cache
            .write_txn_input(&Txn {
                txn_number,
                operation: data.clone(),
            })
            .await;
    }

    fn collect_subs_to_poke(
        subs_to_poke: &mut HashSet<Self::Subscribe>,
        resource: &Self::ResourceToStore,
    ) {
        let operation_name = match resource {
            OperationsOk::SignUp(_) => OperationName::SignUp,
            OperationsOk::SignIn(_) => OperationName::SignIn,
            OperationsOk::CreateCompany(_) => OperationName::CreateCompany,
            OperationsOk::CreateCompanyBranch(_) => OperationName::CreateCompanyBranch,
            OperationsOk::CreateAccount(_) => OperationName::CreateAccount,
            OperationsOk::CreateAccountForBranch(_) => OperationName::CreateAccountForBranch,
            OperationsOk::CreateJournalEntry(_) => OperationName::CreateJournalEntry,
            OperationsOk::ListCompanyAndBranch(_) => OperationName::ListCompanyAndBranch,
            OperationsOk::GetAllAccounts(_) => OperationName::GetAllAccounts,
            OperationsOk::GetAllAccountsForBranch(_) => OperationName::GetAllAccountsForBranch,
        };

        subs_to_poke.insert(operation_name);
    }

    fn convert_resource_from_server_to_resource_to_store(
        resource: &Self::ResourceFromServer,
    ) -> Vec<Self::ResourceToStore> {
        let mut a = Vec::new();

        for resource in resource {
            let b = match resource {
                ResourceDTO::CreateCompany(i) => OperationsOk::CreateCompany(i.clone()),
                ResourceDTO::CreateCompanyBranch(i) => OperationsOk::CreateCompanyBranch(i.clone()),
                ResourceDTO::CreateAccount(i) => OperationsOk::CreateAccount(i.clone()),
                ResourceDTO::CreateAccountForBranch(i) => {
                    OperationsOk::CreateAccountForBranch(i.clone())
                }
                ResourceDTO::CreateJournalEntry(i) => OperationsOk::CreateJournalEntry(i.clone()),
            };

            a.push(b);
        }

        a
    }
}
