impl crate::tests::transformed::Tagged {
    /// Returns the label in uppercase.
    pub fn shout(&self) -> String {
        self.label.to_uppercase()
    }
}
