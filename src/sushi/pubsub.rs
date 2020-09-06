use super::db::*;
use super::domain::GameEvent;
use super::GameId;
use actix_web::web::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    sync::{
        broadcast::{channel, Receiver as BroadcastReceiver, Sender as BroadcastSender},
        mpsc::UnboundedReceiver,
        oneshot, Mutex,
    },
    time::delay_for,
};

#[derive(Debug)]
pub enum Countdown {
    Start { game_id: GameId, duration: Duration },
    Cancel { game_id: GameId },
}

struct Countdowns {
    map: Mutex<HashMap<GameId, oneshot::Receiver<()>>>,
}

impl Countdowns {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    async fn insert(&self, game_id: GameId, receiver: oneshot::Receiver<()>) {
        self.map.lock().await.insert(game_id, receiver);
    }

    async fn remove(&self, game_id: GameId) -> Option<oneshot::Receiver<()>> {
        self.map.lock().await.remove(&game_id)
    }
}

pub async fn receiver(
    broadcaster: Broadcaster,
    db: Database,
    mut rx: UnboundedReceiver<Countdown>,
) {
    let countdowns = Arc::new(Countdowns::new());

    while let Some(msg) = rx.recv().await {
        debug!("received a countdown message {:?}", msg);

        match msg {
            Countdown::Start { game_id, duration } => {
                let (tx, rx) = oneshot::channel();

                tokio::spawn(countdown(
                    broadcaster.clone(),
                    db.clone(),
                    countdowns.clone(),
                    tx,
                    game_id,
                    duration,
                ));

                countdowns.insert(game_id, rx).await;

                broadcaster
                    .send(game_id, &GameEvent::CountdownStarted(duration))
                    .await;
            }

            Countdown::Cancel { game_id } => {
                if let Some(mut receiver) = countdowns.remove(game_id).await {
                    receiver.close();

                    broadcaster
                        .send(game_id, &GameEvent::CountdownCancelled)
                        .await;
                }
            }
        }
    }
}

async fn countdown(
    broadcaster: Broadcaster,
    db: Database,
    countdowns: Arc<Countdowns>,
    mut tx: oneshot::Sender<()>,
    game_id: GameId,
    duration: Duration,
) {
    tokio::select! {
        _ = &mut delay_for(duration) => {
            info!("Countdown for {} completed after {:?}", game_id, duration);

            match db.read_game(game_id) {
                Err(e) => error!("failed to read game from db because {}", e),
                Ok(None) => warn!("failed to find game with id {}", game_id),
                Ok(Some((mut game, version))) => {
                    let events = game.end_turn();

                    if let Err(e) = db.update_game(game_id, &game, version) {
                        error!("failed to update game because {}", e);
                    }

                    for event in events {
                        broadcaster.send(game_id, &event).await;
                    }
                }
            }
        },

        _ = tx.closed() => {
            // interrupted
            info!("Countdown for {} was interrupted", game_id);
        }
    }

    countdowns.remove(game_id).await;
}

#[derive(Debug, Clone)]
pub struct Broadcaster {
    senders_by_game_id: Arc<Mutex<HashMap<GameId, BroadcastSender<Bytes>>>>,
}

impl Broadcaster {
    pub fn new() -> Broadcaster {
        Self {
            senders_by_game_id: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, game_id: GameId) -> BroadcastReceiver<Bytes> {
        debug!("New subscriber for game {}", game_id);

        let rx = {
            let mut guard = self.senders_by_game_id.lock().await;

            if let Some(tx) = guard.get(&game_id) {
                debug!("Sender exists for game {}", game_id);
                tx.subscribe()
            } else {
                debug!("Sender does NOT exist for game {}", game_id);
                let (tx, rx) = channel(100);
                guard.insert(game_id, tx);
                rx
            }
        };

        rx
    }

    pub async fn send(&self, game_id: GameId, game_event: &GameEvent) {
        let tx = {
            let mut guard = self.senders_by_game_id.lock().await;

            if let Some(tx) = guard.get(&game_id) {
                tx.clone()
            } else {
                let (tx, _) = channel(100);
                guard.insert(game_id, tx.clone());
                tx
            }
        };

        let msg = serialize(game_event).expect("failed to serialize game event");

        debug!("Sending {} to {} receivers", msg, tx.receiver_count());

        if let Err(e) = tx.send(Bytes::from(msg)) {
            debug!("no one was listening for event {:?} in game {}", e, game_id);
        }
    }
}

fn serialize(event: &GameEvent) -> Result<String, serde_json::Error> {
    use GameEvent::*;

    let (event, data) = match event {
        CardsSelected(id) => ("cardsselected", serde_json::to_string(&id)?),
        CountdownStarted(d) => ("countdownstarted", serde_json::to_string(&d.as_millis())?),
        CountdownCancelled => ("countdowncancelled", serde_json::to_string(&())?),
        TurnOver => ("turnover", serde_json::to_string(&())?),
        RoundOver(obj) => ("roundover", serde_json::to_string(&obj)?),
        GameOver(winner) => ("gameover", serde_json::to_string(&winner)?),
    };

    Ok(["event: ", event, "\n", "data: ", &data, "\n\n"].concat())
}
