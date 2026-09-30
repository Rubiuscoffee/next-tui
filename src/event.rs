use crate::analyzer::AnalyzerEvent;
use crate::runner::RunnerEvent;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Runner(RunnerEvent),
    Analyzer(AnalyzerEvent),
}

pub struct EventHandler {
    pub runner_tx: UnboundedSender<RunnerEvent>,
    runner_rx: UnboundedReceiver<RunnerEvent>,
    pub analyzer_tx: UnboundedSender<AnalyzerEvent>,
    analyzer_rx: UnboundedReceiver<AnalyzerEvent>,
    crossterm_events: EventStream,
    tick_interval: tokio::time::Interval,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (runner_tx, runner_rx) = unbounded_channel();
        let (analyzer_tx, analyzer_rx) = unbounded_channel();
        let crossterm_events = EventStream::new();
        let tick_interval = tokio::time::interval(tick_rate);

        Self {
            runner_tx,
            runner_rx,
            analyzer_tx,
            analyzer_rx,
            crossterm_events,
            tick_interval,
        }
    }

    pub async fn next(&mut self) -> Result<AppEvent, ()> {
        tokio::select! {
            _ = self.tick_interval.tick() => {
                Ok(AppEvent::Tick)
            }
            Some(runner_msg) = self.runner_rx.recv() => {
                Ok(AppEvent::Runner(runner_msg))
            }
            Some(analyzer_msg) = self.analyzer_rx.recv() => {
                Ok(AppEvent::Analyzer(analyzer_msg))
            }
            maybe_event = self.crossterm_events.next() => {
                match maybe_event {
                    Some(Ok(event)) => match event {
                        CrosstermEvent::Key(key) => Ok(AppEvent::Key(key)),
                        CrosstermEvent::Mouse(mouse) => Ok(AppEvent::Mouse(mouse)),
                        _ => Ok(AppEvent::Tick),
                    },
                    _ => Err(()),
                }
            }
        }
    }
}
