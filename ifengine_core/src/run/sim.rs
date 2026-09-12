use std::collections::{HashMap, HashSet};

use iddqd::{IdHashMap, id_hash_map::Entry};

use crate::{
    Action, Game, GameError, SimEnd, View,
    core::{GameContext, GameTags, PageId, Response},
    utils::_dbg,
};

use super::Interactable;

impl<C: GameContext> Game<C> {
    /// Performs a depth-first traversal of story branches starting from the current page.
    ///
    /// # Algorithm
    /// 1. Clones the initial game state with `simulating = true`.
    /// 2. Executes each reachable branch in depth-first order by enumerating all
    ///    simulatable interactables ([`interactables_sim`](View::interactables_sim)).
    /// 3. When a tunnel boundary is encountered ([`Action::Tunnel`] or [`Response::Tunnel`]),
    ///    the tunnel's target is queued for a separate traversal pass in [`runs`](Simulation::runs).
    /// 4. Records visited pages, incoming transitions, collected tags, and terminal conditions
    ///    ([`SimEnd`]) in [`PageRecords`].
    ///
    /// # Halting and Cycles
    /// Traversal relies on story branches being acyclic outside of tunnels, or on the `visitor`
    /// callback returning `false` to prune branches. Unbounded loops without tunnels or visitor
    /// pruning will not terminate.
    ///
    /// # Arguments
    /// - `visitor`: Called before expanding each [`SimulationState`]. Returning `false` halts
    ///   further exploration along that branch.
    ///
    /// # Panics
    /// Panics if the current game state has no active page on the stack.
    pub fn simulate<F>(&self, mut visitor: F) -> Simulation
    where
        F: FnMut(&mut SimulationState<C>) -> bool,
    {
        let mut ret = Simulation::new();

        let start_page = self.pages.current().unwrap();
        let tun_id = start_page.canonical_id();
        let mut start = self.clone();
        start.simulating = true;

        let mut tunnels_queue = vec![(tun_id, start)];

        while let Some((tun_id, start)) = tunnels_queue.pop() {
            let records = ret.runs.entry(tun_id).or_insert(PageRecords::new());
            let mut queue = vec![SimulationState::new(start)];
            let _ = Self::simulate_impl(&mut queue, records, &mut tunnels_queue, &mut visitor);
        }

        ret
    }

    fn interact_sim(&mut self, e: Interactable<'_>) -> Result<(), SimEnd> {
        match e {
            Interactable::Choice(key, _, index) => {
                self.handle_choice(*key, index);
                Ok(())
            }
            Interactable::Span(_, s) => {
                let action = s.action.as_ref().unwrap();
                match action {
                    Action::Tunnel(next) => {
                        let tun_target = next.canonical_id();

                        let mut next = next.clone();
                        next.id.clear();

                        self.pages.adv_stack();
                        let _ = self.pages.push(next);
                        return Err(SimEnd::Tunnel(tun_target));
                    }
                    Action::Exit => Err(SimEnd::TunnelExit),
                    _ => self
                        .handle_action(action.clone())
                        .map(|_| {})
                        .map_err(|e| e.into()),
                }
            }
        }
    }

    fn simulate_impl<F>(
        queue: &mut Vec<SimulationState<C>>,
        records: &mut PageRecords,
        tunnels_queue: &mut Vec<(PageId, Self)>,
        visitor: &mut F,
    ) where
        F: FnMut(&mut SimulationState<C>) -> bool,
    {
        // dfs
        while let Some(mut s) = queue.pop() {
            // unimportant preflight
            let Some(mut page) = s.pages.current() else {
                return;
            };
            if page.id.is_empty() {
                s.pages.pop(); // drop the initial page for the next rendered and (possibly same) page. In particular, it will have the fully resolved name, (while i.e. the pagehandles produced by link! in handle_action don't).
            }
            _dbg!(&page);

            if !visitor(&mut s) {
                continue; // could support custom ends here
            }

            let v_res = loop {
                let r = page.call(&mut s);
                match r {
                    Response::View(view) => {
                        page.id = view.pageid.clone(); // id the page by the fully resolved name
                        _dbg!(&page.id);
                        break s.pages.push(page).map(|_| view).map_err(|e| e.into()); // only rendered pages get added to history
                    }
                    Response::Switch(next) => {
                        page = next;
                    }
                    Response::Back(n) => match s.pages.pop_n(n) {
                        Ok(p) => page = p,
                        Err(e) => break Err(e.into()),
                    },
                    Response::Tunnel(mut next) => {
                        let tun_target = next.canonical_id();
                        if records.record_outgoing_tunnel(s.pageid(), &tun_target) {
                            let mut fork = s.game.clone();
                            next.id.clear();
                            fork.pages.adv_stack();
                            let _ = fork.pages.push(next);
                            tunnels_queue.push((tun_target.clone(), fork));
                        }
                        break Err(SimEnd::Tunnel(tun_target));
                    }
                    Response::Exit => {
                        // we cannot fully distinguish between tunnel_exit and game_end by this response variant
                        break Err(SimEnd::TunnelExit);
                    }
                    Response::End => break Err(GameError::End.into()),
                }
            };

            match v_res {
                Ok(mut v) => {
                    let mut to_queue = vec![];

                    Self::simulate_view(&mut v, &mut s, records, tunnels_queue, &mut to_queue);

                    queue.extend(to_queue.into_iter().rev());
                }
                Err(e) => {
                    records.push_sim_end(s.pageid(), e);
                }
            };
        }
    }

    fn simulate_view(
        v: &mut View,
        s: &mut SimulationState<C>,
        records: &mut PageRecords,
        tunnels_queue: &mut Vec<(PageId, Self)>,
        to_queue: &mut Vec<SimulationState<C>>,
    ) {
        let pageid = v.pageid.clone();
        records.insert_view(s, &pageid, v);
        s.inner.last_id = pageid.clone();

        for e in v.interactables_sim() {
            _dbg!(&e.content());
            let mut next = s.next();
            match next.interact_sim(e) {
                Ok(()) => {
                    to_queue.push(next);
                    _dbg!(to_queue.len());
                }
                Err(e) => {
                    if let SimEnd::Tunnel(tun_target) = &e {
                        _dbg!("tun");
                        if records.record_outgoing_tunnel(&pageid, tun_target) {
                            tunnels_queue.push((tun_target.clone(), next.game));
                        }
                    }
                    records.push_sim_end(&pageid, e);
                }
            }
        }
    }
}

// need a way to collapse paths which lead to the same result

/// Output of a simulation pass across reachable story branches.
#[derive(Debug, Clone)]
pub struct Simulation {
    /// History of runs, keyed by entry point page ID.
    pub runs: HashMap<PageId, PageRecords>,
}

/// The active traversal state along an in-flight branch during simulation.
#[derive(Debug, Clone)]
pub struct SimulationState<C> {
    pub game: Game<C>,
    pub depth: usize,
}

impl<C: Clone> SimulationState<C> {
    fn new(game: Game<C>) -> Self {
        Self { game, depth: 0 }
    }

    fn next(&self) -> Self {
        let mut ret = self.clone();
        ret.depth += 1;
        ret
    }
}

/// Static analysis record for an individual page encountered during simulation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageRecord {
    pub id: PageId,
    pub ends: HashSet<SimEnd>,
    pub tags: GameTags,
    pub incoming: HashSet<PageId>,
    pub min_depth: usize,
    pub outgoing_tunnels: HashSet<PageId>,
}

/// Fast lookup table mapping [`PageId`] to its [`PageRecord`].
#[derive(Debug, Clone)]
pub struct PageRecords(pub IdHashMap<PageRecord>);

impl PageRecord {
    pub fn new(id: PageId) -> Self {
        PageRecord {
            id,
            ends: Default::default(),
            tags: Default::default(),
            incoming: Default::default(),
            min_depth: usize::MAX,
            outgoing_tunnels: Default::default(),
        }
    }

    pub fn split(mut self) -> (Self, HashSet<PageId>) {
        let incoming = std::mem::take(&mut self.incoming);
        (self, incoming)
    }

    pub fn is_empty(&self) -> bool {
        self.ends.is_empty() && self.tags.is_empty() && self.outgoing_tunnels.is_empty()
    }

    pub fn compute_display_width(&self) -> usize {
        let mut max = 6;
        for end in &self.ends {
            let len = format!("{:?}", end).len();
            if len > max {
                max = len;
            }
        }
        for tag in &self.tags {
            let len = tag.len();
            if len > max {
                max = len;
            }
        }
        for tun in &self.outgoing_tunnels {
            let len = tun.len();
            if len > max {
                max = len;
            }
        }
        max
    }
}

impl PageRecords {
    // Drains the seen tags into the record, and adds an incoming edge
    pub fn insert_view<C>(&mut self, s: &SimulationState<C>, pageid: &PageId, v: &mut View) {
        match self.entry(pageid) {
            Entry::Occupied(mut occ) => {
                let mut record = occ.get_mut();

                record.tags.extend(v.tags.drain(0..v.tags.len()));
                if s.depth > 0 {
                    record.incoming.insert(s.game.inner.last_id.clone());
                }
                record.min_depth = record.min_depth.min(s.depth);
            }

            Entry::Vacant(vac) => {
                let mut record = PageRecord::new(pageid.clone());

                record.tags.extend(v.tags.drain(0..v.tags.len()));
                if s.depth > 0 {
                    record.incoming.insert(s.game.inner.last_id.clone());
                }
                record.min_depth = record.min_depth.min(s.depth);

                vac.insert(record);
            }
        }
    }

    pub fn push_sim_end(&mut self, pageid: &PageId, e: SimEnd) {
        if let Some(mut record) = self.0.get_mut(pageid) {
            record.ends.insert(e.into());
        }
    }

    /// Records an outgoing tunnel transition from `from` to `target`.
    /// Returns `true` if this is the first time entering `target` from `from`,
    /// or `false` if the tunnel was already entered from `from`.
    pub fn record_outgoing_tunnel(&mut self, from: &PageId, target: &PageId) -> bool {
        match self.entry(from) {
            Entry::Occupied(mut occ) => occ.get_mut().outgoing_tunnels.insert(target.clone()),
            Entry::Vacant(vac) => {
                let mut record = PageRecord::new(from.clone());
                let is_new = record.outgoing_tunnels.insert(target.clone());
                vac.insert(record);
                is_new
            }
        }
    }

    // this can be 0!
    pub fn depth(&self) -> usize {
        self.0.iter().map(|r| r.min_depth).max().unwrap_or(0)
    }
}

// --------------- BOILERPLATE ------------------------------

impl std::ops::Deref for PageRecords {
    type Target = IdHashMap<PageRecord>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PageRecords {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C> std::ops::Deref for SimulationState<C> {
    type Target = Game<C>;
    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl<C> std::ops::DerefMut for SimulationState<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game
    }
}

impl PageRecords {
    pub fn new() -> Self {
        PageRecords(Default::default())
    }
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            runs: Default::default(),
        }
    }
}

impl iddqd::IdHashItem for PageRecord {
    type Key<'a> = &'a PageId;

    fn key(&self) -> Self::Key<'_> {
        &self.id
    }

    iddqd::id_upcast!();
}
