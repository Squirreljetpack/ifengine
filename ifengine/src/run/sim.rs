use std::{
    collections::{HashMap, HashSet},
    thread::current,
};

use iddqd::{IdHashMap, id_hash_map::Entry};

use crate::{
    Action, Game, GameError, SimEnd, View,
    core::{GameContext, GameTags, PageHandle, PageId, PageStack, Response, game_state::GameState},
    view::Object,
};

use super::Interactable;

impl<C: GameContext> Game<C> {
    /// The user must ensure that all cycles must be modelled by tunnels. We guarantee to never visit the same tunnel from the same location twice, but the presence of other loops will result in failure to halt. Although certain types of elements generated from proc_macros
    /// Panics if current game state is not a view
    /// F:
    pub fn simulate<F>(&self, mut visitor: F) -> Simulation
    where
        F: FnMut(&mut SimulationState<C>) -> bool,
    {
        let mut ret = Simulation::new();

        // A tunnel categorized by the function which it enters into, which does not necessarily must respond with a view
        let tun_id = self
            .pages
            .current()
            .unwrap()
            .id
            .rsplit("::")
            .next()
            .unwrap()
            .into(); // all tunnels with the same basename are grouped the same. This is because the pagehandles contained by tunnel cannot be guaranteed to have the same import style. Although this too, is still very error_prone (i.e. renames) as well as runs the risk of collisions.
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

    fn interact_sim(&mut self, e: Interactable<'_>, pageid: &PageId) -> Result<(), SimEnd> {
        match e {
            Interactable::Choice(key, _, index) => {
                self.handle_choice((pageid.clone(), *key), index);
                Ok(())
            }
            Interactable::Span(_, s) => {
                let action = s.action.as_ref().unwrap();
                match action {
                    Action::Tunnel(next) => {
                        let fork_name = next.id.rsplit("::").next().unwrap().into();

                        let mut next = next.clone();
                        next.id.clear();

                        self.pages = PageStack::new_with_page(next);
                        return Err(SimEnd::Tunnel(fork_name));
                    }
                    Action::Exit => Err(SimEnd::TunnelExit),
                    _ => self.handle_action(action.clone()).map_err(|e| e.into()),
                }
            }
        }
    }

    fn simulate_impl<F>(
        queue: &mut Vec<SimulationState<C>>,
        records: &mut PageRecords,
        tunnels_queue: &mut Vec<(String, Self)>,
        visitor: &mut F,
    ) where
        F: FnMut(&mut SimulationState<C>) -> bool,
    {
        while let Some(mut s) = queue.pop() {
            // unimportant preflight
            let Some(mut page) = s.pages.current() else {
                return;
            };
            if page.id.is_empty() {
                s.pages.pop(); // drop the initial page for the next rendered and (possibly same) page. In particular, it will have the fully resolved name, (while i.e. the pagehandles produced by link! in handle_action don't).
            }

            if visitor(&mut s) {
                continue; // could support custom ends here
            }

            let v_res = loop {
                let r = page.call(&mut s);
                match r {
                    Response::View(view) => {
                        page.id = view.pageid.clone(); // id the page by the fully resolved name
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
                        let mut fork = s.game.clone();
                        let fork_name = next.id.rsplit("::").next().unwrap().to_string(); // note: why compiler can't infer into_string() here
                        next.id.clear();
                        fork.pages = PageStack::new_with_page(next);
                        tunnels_queue.push((fork_name.clone(), fork));
                        break Err(SimEnd::Tunnel(fork_name));
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
                    let curr_id = v.pageid.clone();
                    records.insert_view(&s, &mut v);
                    let mut to_queue = vec![];

                    for e in v.interactables_sim() {
                        let mut next = s.next(curr_id.clone());
                        match next.interact_sim(e, &curr_id) {
                            Ok(()) => {
                                to_queue.push(next);
                            }
                            Err(e) => {
                                if let SimEnd::Tunnel(fork_name) = &e {
                                    tunnels_queue.push((fork_name.clone(), next.game));
                                }
                                records.push_sim_end(&curr_id, e.into());
                            }
                        }
                    }

                    queue.extend(to_queue.into_iter().rev());
                }
                Err(e) => {
                    if let Some(last) = s.last.as_ref() {
                        records.push_sim_end(last, e)
                    }
                }
            };
        }
    }
}

// need a way to collapse paths which lead to the same result

#[derive(Debug, Clone)]
pub struct Simulation {
    /// A history of runs, one for each starting point. Starts consist of tunnel entrances and the initial game start.
    pub runs: HashMap<String, PageRecords>,
}

#[derive(Debug, Clone)]
pub struct SimulationState<C> {
    pub game: Game<C>,
    pub depth: usize,
    pub last: Option<PageId>,
}

impl<C: Clone> SimulationState<C> {
    fn new(game: Game<C>) -> Self {
        Self {
            game,
            depth: 0,
            last: None,
        }
    }

    fn next(&self, curr_id: PageId) -> Self {
        let mut ret = self.clone();
        ret.depth += 1;
        ret.last = Some(curr_id);
        ret
    }
}

// note: outgoing_tunnels not currently implemented
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
        self.ends.is_empty() && self.tags.is_empty()
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
        max
    }
}

impl PageRecords {
    // Drains the seen tags into the record, and adds an incoming edge
    pub fn insert_view<C>(&mut self, s: &SimulationState<C>, v: &mut View) {
        let pageid = v.pageid.clone();
        let prev = s.last.clone();

        match self.entry(&pageid) {
            Entry::Occupied(mut occ) => {
                let mut record = occ.get_mut();

                record.tags.extend(v.tags.drain(0..v.tags.len()));
                if let Some(prev) = prev {
                    record.incoming.insert(prev);
                }
                record.min_depth = record.min_depth.min(s.depth);
            }

            Entry::Vacant(vac) => {
                let mut record = PageRecord::new(pageid);

                record.tags.extend(v.tags.drain(0..v.tags.len()));
                if let Some(prev) = prev {
                    record.incoming.insert(prev);
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
