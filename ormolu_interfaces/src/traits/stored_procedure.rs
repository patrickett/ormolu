use super::DatabaseObject;

pub trait StoredProcedure
where
    Self: DatabaseObject,
{
    // TODO: remove T here and use cli to generate return types for StoredProcedure
    fn execute<T>(&self) -> Result<T, String> {
        todo!()
    }
}
