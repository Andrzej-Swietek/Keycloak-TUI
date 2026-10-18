//! The runtime: terminal, event sources, and the loop that ties `update` to `view`.

mod clipboard;
mod effects;
mod files;
mod requests;
mod scim;
mod tokens;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use kct_core::catalog::Resource;
use kct_core::ports::Gateway;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::effect::Effect;
use crate::model::{Model, Session};
use crate::msg::Msg;
use crate::{update, view};

/// How often toasts are aged out and the token countdown refreshed.
const TICK: Duration = Duration::from_millis(500);

pub async fn run(
    session: Session,
    gateway: Arc<dyn Gateway>,
    tokens: Arc<dyn kct_core::ports::TokenSource>,
    start: &'static Resource,
    mascots: crate::view::Scan,
) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let outcome = drive(&mut terminal, session, gateway, tokens, start, mascots).await;
    ratatui::restore();
    outcome
}

async fn drive(
    terminal: &mut ratatui::DefaultTerminal,
    session: Session,
    gateway: Arc<dyn Gateway>,
    tokens: Arc<dyn kct_core::ports::TokenSource>,
    start: &'static Resource,
    mascots: crate::view::Scan,
) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    spawn_input(tx.clone());
    spawn_ticker(tx.clone());

    let realm = session.realm.to_string();
    let prefs = session.prefs;
    let mut model = Model {
        prefs: prefs.clamped(mascots.found.len()),
        mascots,
        ..Model::new(session, start)
    };

    for effect in [
        Effect::DetectCapabilities {
            realm: realm.clone(),
        },
        Effect::DetectUserProfile {
            realm: realm.clone(),
        },
        Effect::DetectRealmFlags {
            realm: realm.clone(),
        },
        Effect::DetectAuthFlows { realm },
        Effect::ListRealms,
    ] {
        spawn(effect, &gateway, &tokens, &tx);
    }
    for effect in update::reload(model.clone()).effects {
        spawn(effect, &gateway, &tokens, &tx);
    }
    model = update::reload(model).model;

    while !model.quit {
        let rows = view::viewport_rows(terminal.size()?.height);
        if rows != model.viewport_rows {
            model = update::update(model, Msg::Viewport(rows)).model;
        }

        terminal.draw(|frame| view::render(&model, frame))?;

        let Some(first) = rx.recv().await else { break };

        // Drain whatever else is queued before redrawing: a burst of keystrokes or log
        // messages should cost one frame, not one frame each.
        let mut pending = vec![first];
        while let Ok(next) = rx.try_recv() {
            pending.push(next);
        }

        for msg in pending {
            let step = update::update(model, msg);
            model = step.model;
            for effect in step.effects {
                spawn(effect, &gateway, &tokens, &tx);
            }
        }
    }

    Ok(())
}

fn spawn(
    effect: Effect,
    gateway: &Arc<dyn Gateway>,
    tokens: &Arc<dyn kct_core::ports::TokenSource>,
    tx: &UnboundedSender<Msg>,
) {
    if matches!(effect, Effect::Quit) {
        return;
    }
    let gateway = Arc::clone(gateway);
    let tokens = Arc::clone(tokens);
    let tx = tx.clone();
    tokio::spawn(async move { effects::run(effect, gateway, tokens, &tx).await });
}

fn spawn_input(tx: UnboundedSender<Msg>) {
    tokio::spawn(async move {
        let mut events = crossterm::event::EventStream::new();
        while let Some(Ok(event)) = events.next().await {
            let msg = match event {
                crossterm::event::Event::Key(key) => Msg::Key(key),
                crossterm::event::Event::Resize(_, height) => {
                    Msg::Viewport(view::viewport_rows(height))
                }
                _ => continue,
            };
            if tx.send(msg).is_err() {
                break;
            }
        }
    });
}

fn spawn_ticker(tx: UnboundedSender<Msg>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(TICK);
        loop {
            ticker.tick().await;
            if tx.send(Msg::Tick).is_err() {
                break;
            }
        }
    });
}
