pub mod current;
pub mod diesel;
pub mod history;
pub mod sqlite;

use std::marker::PhantomData;

use error_stack::{Result, ResultExt};
pub use model::schema;
use model::{AccountIdInternal, AccountIdLight, ContentId};

use utils::ComponentError;

pub struct NoId;

pub type WriteResult<T, Err, WriteContext = T> =
    std::result::Result<T, WriteError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryWriteResult<T, Err, WriteContext = T> =
    std::result::Result<T, HistoryWriteError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct WriteError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for WriteError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for WriteError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryWriteError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for HistoryWriteError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for HistoryWriteError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

pub type ReadResult<T, Err, WriteContext = T> =
    std::result::Result<T, ReadError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryReadResult<T, Err, WriteContext = T> =
    std::result::Result<T, HistoryReadError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct ReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for ReadError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for ReadError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for HistoryReadError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for HistoryReadError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DatabaseId {
    Light(AccountIdLight),
    Internal(AccountIdInternal),
    Content(AccountIdLight, ContentId),
    Empty,
}

impl From<AccountIdLight> for DatabaseId {
    fn from(value: AccountIdLight) -> Self {
        DatabaseId::Light(value)
    }
}

impl From<AccountIdInternal> for DatabaseId {
    fn from(value: AccountIdInternal) -> Self {
        DatabaseId::Internal(value)
    }
}

impl From<(AccountIdLight, ContentId)> for DatabaseId {
    fn from(value: (AccountIdLight, ContentId)) -> Self {
        DatabaseId::Content(value.0, value.1)
    }
}

impl From<NoId> for DatabaseId {
    fn from(_: NoId) -> Self {
        DatabaseId::Empty
    }
}

pub trait ConvertCommandError<D>: Sized {
    type Err: ComponentError;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, Self::Err>;
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D> for WriteResult<D, E, CmdContext> {
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(WriteError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} write command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D>
    for HistoryWriteResult<D, E, CmdContext>
{
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(HistoryWriteError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} history write command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D> for ReadResult<D, E, CmdContext> {
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(ReadError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} read command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D>
    for HistoryReadResult<D, E, CmdContext>
{
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(HistoryReadError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} history read command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}
