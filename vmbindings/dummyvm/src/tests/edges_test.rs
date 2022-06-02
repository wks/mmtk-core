// GITHUB-CI: MMTK_PLAN=NoGC

use atomic::{Atomic, Ordering};
use mmtk::{
    util::{Address, ObjectReference},
    vm::edge_shape::{Edge, SimpleEdge},
};

use crate::{
    edges::{CompressedOopEdge, DummyVMEdge, OffsetEdge, ValueEdge},
    tests::fixtures::{Fixture, TwoObjects},
};

lazy_static! {
    static ref FIXTURE: Fixture<TwoObjects> = Fixture::new();
}

#[test]
pub fn load_simple() {
    FIXTURE.with_fixture(|fixture| {
        let mut slot: Atomic<ObjectReference> = Atomic::new(fixture.objref1);

        let edge = SimpleEdge::from_address(Address::from_ref(&mut slot));
        let objref = edge.load();

        assert_eq!(objref, fixture.objref1);
    });
}

#[test]
pub fn store_simple() {
    FIXTURE.with_fixture(|fixture| {
        let mut slot: Atomic<ObjectReference> = Atomic::new(fixture.objref1);

        let edge = SimpleEdge::from_address(Address::from_ref(&mut slot));
        edge.store(fixture.objref2);
        assert_eq!(slot.load(Ordering::SeqCst), fixture.objref2);

        let objref = edge.load();
        assert_eq!(objref, fixture.objref2);
    });
}

#[test]
pub fn load_compressed() {
    FIXTURE.with_fixture(|fixture| {
        let usize1 = fixture.objref1.to_address().as_usize();
        if usize1 > u32::MAX as usize {
            // skip test.  Address too high.
            return;
        }
        let mut slot: Atomic<u32> = Atomic::new(usize1 as u32);

        let edge = CompressedOopEdge::from_address(Address::from_ref(&mut slot));
        let objref = edge.load();

        assert_eq!(objref, fixture.objref1);
    });
}

#[test]
pub fn store_compressed() {
    FIXTURE.with_fixture(|fixture| {
        let usize1 = fixture.objref1.to_address().as_usize();
        let usize2 = fixture.objref2.to_address().as_usize();
        if usize1 > u32::MAX as usize || usize2 > u32::MAX as usize {
            // skip test.  Address too high.
            return;
        }
        let mut slot: Atomic<u32> = Atomic::new(usize1 as u32);

        let edge = CompressedOopEdge::from_address(Address::from_ref(&mut slot));
        edge.store(fixture.objref2);
        assert_eq!(slot.load(Ordering::SeqCst), usize2 as u32);

        let objref = edge.load();
        assert_eq!(objref, fixture.objref2);
    });
}

#[test]
pub fn load_offset() {
    const OFFSET: usize = 48;
    FIXTURE.with_fixture(|fixture| {
        let addr1 = fixture.objref1.to_address();
        let mut slot: Atomic<Address> = Atomic::new(addr1 + OFFSET);

        let edge = OffsetEdge::new_with_offset(Address::from_ref(&mut slot), OFFSET);
        let objref = edge.load();

        assert_eq!(objref, fixture.objref1);
    });
}

#[test]
pub fn store_offset() {
    const OFFSET: usize = 48;
    FIXTURE.with_fixture(|fixture| {
        let addr1 = fixture.objref1.to_address();
        let addr2 = fixture.objref2.to_address();
        let mut slot: Atomic<Address> = Atomic::new(addr1 + OFFSET);

        let edge = OffsetEdge::new_with_offset(Address::from_ref(&mut slot), OFFSET);
        edge.store(fixture.objref2);
        assert_eq!(slot.load(Ordering::SeqCst), addr2 + OFFSET);

        let objref = edge.load();
        assert_eq!(objref, fixture.objref2);
    });
}

#[test]
pub fn load_value() {
    FIXTURE.with_fixture(|fixture| {
        let edge = ValueEdge::new(fixture.objref1);
        let objref = edge.load();

        assert_eq!(objref, fixture.objref1);
    });
}

#[test]
pub fn mixed() {
    const OFFSET: usize = 48;

    FIXTURE.with_fixture(|fixture| {
        let addr1 = fixture.objref1.to_address();
        let addr2 = fixture.objref2.to_address();

        let mut slot1: Atomic<ObjectReference> = Atomic::new(fixture.objref1);
        let mut slot3: Atomic<Address> = Atomic::new(addr1 + OFFSET);

        let edge1 = SimpleEdge::from_address(Address::from_ref(&mut slot1));
        let edge3 = OffsetEdge::new_with_offset(Address::from_ref(&mut slot3), OFFSET);
        let edge4 = ValueEdge::new(fixture.objref1);

        let de1 = DummyVMEdge::Simple(edge1);
        let de3 = DummyVMEdge::Offset(edge3);
        let de4 = DummyVMEdge::Value(edge4);

        let edges = vec![de1, de3, de4];
        for (i, edge) in edges.iter().enumerate() {
            let objref = edge.load();
            assert_eq!(objref, fixture.objref1, "Edge {} is not properly loaded", i);
        }

        let mutable_edges = vec![de1, de3];
        for (i, edge) in mutable_edges.iter().enumerate() {
            edge.store(fixture.objref2);
            let objref = edge.load();
            assert_eq!(
                objref, fixture.objref2,
                "Edge {} is not properly loaded after store",
                i
            );
        }

        assert_eq!(slot1.load(Ordering::SeqCst), fixture.objref2);
        assert_eq!(slot3.load(Ordering::SeqCst), addr2 + OFFSET);
    });
}
