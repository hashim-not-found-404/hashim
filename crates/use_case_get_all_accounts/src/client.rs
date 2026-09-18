use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use kernel::client::Cache;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::cache::CacheUtility;
use utility::cache::CachingStrategy;
use utility::cache::OpError;
use utility::cache::OpErrorTrait;
use utility::cache::OpInput;
use utility::cache::OpInputTrait;
use utility::cache::OpOk;
use utility::cache::OpOkTrait;
use utility::cache::OpResult;
use utility::cache::Subscribe;
use utility::dtos::OperationsError;
use utility::dtos::OperationsInput;
use utility::dtos::OperationsOk;
use utility::dtos::TxnNumber;

impl<Ch: Cache> OpOkTrait<Ch> for Ok {
    fn into_serde(self: Box<Self>) -> Box<dyn OperationsOk> {
        todo!()
    }

    fn apply_to_cache(&self, cache: &mut Ch) -> Pin<Box<dyn Future<Output = ()>>> {
        todo!()
    }

    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

impl OpErrorTrait for Error {
    fn into_serde(self: Box<Self>) -> Box<dyn OperationsError> {
        todo!()
    }

    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct WrapperInput<Ch, DBReader>
where
    Ch: Cache + Debug + Clone,
    DBReader: DatabaseRead<Db = Ch, Input = ReadInput, Output = ReadOutput> + Debug + Clone,
{
    inner: Input,
    _ph:   PhantomData<(DBReader, Ch)>,
}

impl<Ch, DBReader> OpInputTrait<Ch> for WrapperInput<Ch, DBReader>
where
    Ch: Cache + Debug + Clone,
    DBReader: DatabaseRead<Db = Ch, Input = ReadInput, Output = ReadOutput> + Debug + Clone,
{
    fn into_serde(self: Box<Self>) -> Box<dyn OperationsInput> {
        Box::new(self.inner)
    }

    fn check_input<'a>(
        &'a self,
        cache: &'a mut Ch,
    ) -> Pin<Box<dyn Future<Output = OpResult<Ch>> + 'a>> {
        Box::pin(async {
            let mut errr = self.inner.state_less_check();

            if errr.is_there_error() {
                return Err(OpError(Box::new(errr)));
            }

            let ok = self.inner.state_full_operation::<DBReader>(cache).await.unwrap();

            Ok(OpOk(Box::new(ok)))
        })
    }

    fn user_uuid(&self) -> Option<[u8; 16]> {
        Some(*self.inner.user_uuid.deref().deref())
    }
}

pub async fn fetch<Ch, DBReader>(
    selected_company: CompanyUuid,
    user_uuid: UserUuid,
    mut cache: CacheStruct<Ch>,
) where
    Ch: Cache + Debug + Clone,
    DBReader:
        DatabaseRead<Db = Ch, Input = ReadInput, Output = ReadOutput> + Debug + Clone + 'static,
{
    let input = Input {
        user_uuid,
        company_uuid: selected_company,
    };

    let input = WrapperInput {
        inner: input,
        _ph:   PhantomData::<(DBReader, Ch)>,
    };

    let input: OpInput<Ch> = OpInput(Arc::new(input));

    let txn_number = TxnNumber::default();

    cache.send_to_cache_actor(CachingStrategy::ReadServerOnly, txn_number, input).await;
}
