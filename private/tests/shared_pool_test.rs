//! Tests `shared_descriptor_pool`: with the shared pool emitted at the crate
//! root, both packages delegate to the same lazily-built `DescriptorPool`.

#[test]
fn packages_share_one_descriptor_pool() {
    assert!(std::ptr::eq(
        pool_proto::tests::pool::one::descriptor_pool(),
        pool_proto::tests::pool::two::descriptor_pool()
    ));
}
