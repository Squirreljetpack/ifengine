use ifengine::elements::{paragraph, tun};
use ifengine::Game;
use ifengine::core::PageId;

#[derive(Clone, Debug, Default)]
struct SimState {}

#[ifengine::ifview]
fn tunnel_dest(_state: &mut SimState) {
    paragraph!("Inside tunnel destination");
}

#[ifengine::ifview]
fn tunnel_source(_state: &mut SimState) {
    paragraph!(
        tun!("First tunnel link", tunnel_dest),
        tun!("Second tunnel link", tunnel_dest),
    );
}

#[test]
fn test_tunnel_deduplication_from_same_page() {
    let game = Game::new_with_page("tunnel_source", tunnel_source);
    let sim = game.simulate(|s| s.depth <= 10);

    let source_canon = ifengine::core::resolve_page_id(tunnel_source).unwrap();
    let dest_canon = ifengine::core::resolve_page_id(tunnel_dest).unwrap();

    let source_id: PageId = source_canon.into();
    let dest_id: PageId = dest_canon.into();

    // The simulation runs should contain runs for the start page and the tunnel destination
    let start_run = sim.runs.get(&source_id).expect("source run exists");
    let dest_run = sim.runs.get(&dest_id).expect("dest run exists");

    // Check outgoing tunnels on the source page record
    let source_record = start_run.get(&source_id).expect("source record exists");
    assert_eq!(source_record.outgoing_tunnels.len(), 1);
    assert!(source_record.outgoing_tunnels.contains(&dest_id));
    assert!(source_record.ends.is_empty(), "ends and tunnels must be disjoint");

    // The destination run should only have 1 page record (visited once, not duplicated)
    assert_eq!(dest_run.len(), 1);
}
