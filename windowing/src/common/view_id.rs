slotmap::new_key_type! {
    /// A small unique identifier for an instance of a [View](crate::View).
    ///
    /// This id is how you can access and modify a view, including accessing children views and updating state.
   pub struct ViewId;
}
