use std::io;
use std::sync::{atomic::AtomicI32, mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use crate::audience::Audience;
use crate::commands::Commands;
use crate::drops::Drops;
use crate::mobs::Mobs;
use crate::peers::Peers;
use crate::workers::Workers;
use crate::world::FlatWorld;

const SAVE_INTERVAL: Duration = Duration::from_secs(5);

pub struct Autosave {
    stop: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for Autosave {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Simulation {
    pub terrain: FlatWorld,
    pub commands: Commands,
    pub audience: Audience,
    pub peers: Peers,
    pub mobs: Mobs,
    pub drops: Drops,
    pub next_entity: AtomicI32,
}

pub struct SimulationContext<'a> {
    pub world: &'a FlatWorld,
    pub commands: &'a Commands,
    pub audience: &'a Audience,
    pub drops: &'a Drops,
    pub entity_ids: &'a AtomicI32,
}

impl Simulation {
    pub fn start(
        terrain: FlatWorld,
        commands: Commands,
        worker_count: usize,
    ) -> io::Result<Arc<Self>> {
        let simulation = Arc::new(Self {
            terrain,
            commands,
            audience: Audience::default(),
            peers: Peers::default(),
            mobs: Mobs::default(),
            drops: Drops::default(),
            next_entity: AtomicI32::new(1),
        });
        let workers = Workers::new(worker_count)?;
        let weak = Arc::downgrade(&simulation);
        thread::Builder::new()
            .name("leaf-native-world".into())
            .spawn(move || loop {
                let Some(simulation) = weak.upgrade() else {
                    break;
                };
                let wait = simulation.poll(&workers);
                drop(simulation);
                if wait < Duration::from_millis(1) {
                    thread::yield_now();
                } else {
                    thread::sleep(wait);
                }
            })?;
        Ok(simulation)
    }

    pub fn autosave(self: &Arc<Self>) -> io::Result<Autosave> {
        let (stop, receiver) = mpsc::channel();
        let simulation = Arc::clone(self);
        let thread = thread::Builder::new()
            .name("leaf-native-save".into())
            .spawn(move || loop {
                let stopping =
                    receiver.recv_timeout(SAVE_INTERVAL) != Err(mpsc::RecvTimeoutError::Timeout);
                if let Err(error) = simulation.save_world() {
                    eprintln!("leaf-native: saving block changes: {error}");
                }
                if stopping {
                    break;
                }
            })?;
        Ok(Autosave {
            stop,
            thread: Some(thread),
        })
    }

    pub fn save_world(&self) -> io::Result<usize> {
        self.terrain.save(self.commands.game_time())
    }

    fn poll(&self, workers: &Workers) -> Duration {
        let start = Instant::now();
        let next = self
            .commands
            .world
            .lock()
            .unwrap()
            .poll_tick(start, &self.audience);
        let (tick, finished_step, rules) = match next {
            Ok(tick) => tick,
            Err(delay) => return delay,
        };
        let ctx = SimulationContext {
            world: &self.terrain,
            commands: &self.commands,
            audience: &self.audience,
            drops: &self.drops,
            entity_ids: &self.next_entity,
        };
        self.mobs.tick(tick, &rules, &ctx, &self.peers, workers);
        self.drops.update(tick, &self.terrain, workers);
        let end = Instant::now();
        self.commands.world.lock().unwrap().complete_tick(
            end - start,
            end,
            finished_step,
            &self.audience,
        );
        Duration::ZERO
    }
}
