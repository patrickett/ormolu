pub struct FieldsFilter<P> {
    conditions: Vec<bool>,
    predicate: P,
}
