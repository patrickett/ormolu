use crate::{Key, Table};
use sqlx::{Database, Decode, prelude::Type};
use std::marker::PhantomData;

#[repr(transparent)]
pub struct ForeignKey<Entity, const ORDINAL: usize, T> {
    _entity: PhantomData<Entity>,
    value: T,
}

impl<E, const C: usize, T> AsRef<T> for ForeignKey<E, C, T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<Entity, const ORDINAL: usize, T> Key<Entity, T> for ForeignKey<Entity, ORDINAL, T>
where
    Entity: Table,
{
    async fn get_entity(&self) -> Result<Option<Entity>, crate::OrmoluError> {
        // TODO: OrmoluError this
        let column_name = Entity::column(ORDINAL).expect("msg");

        // Entity::
        // Entity::get_by_primary_key(key)
        todo!()
    }
}

impl<DB: Database, E, const ORDINAL: usize, T> Type<DB> for ForeignKey<E, ORDINAL, T>
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database, E, const ORDINAL: usize, T> Decode<'r, DB> for ForeignKey<E, ORDINAL, T>
where
    String: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        todo!()
    }
}
