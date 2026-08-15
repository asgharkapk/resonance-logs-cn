//! Bounded control plane for the single-owner live runtime.

use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use crate::live::bootstrap_snapshot::MonitorRuntimeSnapshot;
use crate::live::ipc::models::{
    LiveBuffsPayload, LiveCombatPayload, LiveDeathsPayload, LiveFantasyPayload, LiveMonsterPayload,
    LiveScenePayload, LiveStatusPayload,
};
const CONTROL_CAPACITY: usize = 64;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum TopicBootstrap {
    Combat(LiveCombatPayload),
    Status(LiveStatusPayload),
    Buffs(LiveBuffsPayload),
    Monster(LiveMonsterPayload),
    Fantasy(LiveFantasyPayload),
    Deaths(LiveDeathsPayload),
    Scene(LiveScenePayload),
}

/// Topics that support command-side bootstrap. Minimap is push-only: its
/// payload is rebuilt from accumulated projection state on every tick and its
/// skill casts are exactly-once deltas, so a bootstrap would return either an
/// empty snapshot (scene not yet registered) or a stale one (volatile
/// positions). Keeping it out of this type makes that dead path inexpressible.
#[derive(Debug, Clone, Copy)]
pub enum BootstrapTopic {
    Combat,
    Status,
    Buffs,
    Monster,
    Fantasy,
    Deaths,
    Scene,
}

#[derive(Debug)]
pub enum RuntimeCommand {
    GetTopic {
        topic: BootstrapTopic,
        reply: oneshot::Sender<TopicBootstrap>,
    },
    ManualReset,
    TogglePause,
    ApplyMonitorConfig(MonitorRuntimeSnapshot),
    StartTraining,
    StopTraining,
    Shutdown {
        reply: oneshot::Sender<Result<(), String>>,
    },
}

#[derive(Clone, Debug)]
pub struct LiveRuntimeHandle {
    sender: mpsc::Sender<RuntimeCommand>,
}

impl LiveRuntimeHandle {
    pub fn new() -> (Self, mpsc::Receiver<RuntimeCommand>) {
        let (sender, receiver) = mpsc::channel(CONTROL_CAPACITY);
        (Self { sender }, receiver)
    }

    pub async fn topic(&self, topic: BootstrapTopic) -> Result<TopicBootstrap, String> {
        let (reply, receive) = oneshot::channel();
        self.send(RuntimeCommand::GetTopic { topic, reply }).await?;
        receive
            .await
            .map_err(|_| "live runtime stopped before replying".to_string())
    }

    pub async fn combat(&self) -> Result<LiveCombatPayload, String> {
        match self.topic(BootstrapTopic::Combat).await? {
            TopicBootstrap::Combat(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn status(&self) -> Result<LiveStatusPayload, String> {
        match self.topic(BootstrapTopic::Status).await? {
            TopicBootstrap::Status(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn buffs(&self) -> Result<LiveBuffsPayload, String> {
        match self.topic(BootstrapTopic::Buffs).await? {
            TopicBootstrap::Buffs(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn monster(&self) -> Result<LiveMonsterPayload, String> {
        match self.topic(BootstrapTopic::Monster).await? {
            TopicBootstrap::Monster(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn fantasy(&self) -> Result<LiveFantasyPayload, String> {
        match self.topic(BootstrapTopic::Fantasy).await? {
            TopicBootstrap::Fantasy(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn deaths(&self) -> Result<LiveDeathsPayload, String> {
        match self.topic(BootstrapTopic::Deaths).await? {
            TopicBootstrap::Deaths(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn scene(&self) -> Result<LiveScenePayload, String> {
        match self.topic(BootstrapTopic::Scene).await? {
            TopicBootstrap::Scene(payload) => Ok(payload),
            other => Err(format!("unexpected topic bootstrap: {other:?}")),
        }
    }

    pub async fn manual_reset(&self) -> Result<(), String> {
        self.send(RuntimeCommand::ManualReset).await
    }

    pub async fn toggle_pause(&self) -> Result<(), String> {
        self.send(RuntimeCommand::TogglePause).await
    }

    pub async fn apply_monitor_config(
        &self,
        snapshot: MonitorRuntimeSnapshot,
    ) -> Result<(), String> {
        self.send(RuntimeCommand::ApplyMonitorConfig(snapshot))
            .await
    }

    pub async fn start_training(&self) -> Result<(), String> {
        self.send(RuntimeCommand::StartTraining).await
    }

    pub async fn stop_training(&self) -> Result<(), String> {
        self.send(RuntimeCommand::StopTraining).await
    }

    /// Synchronous Tauri exit hook adapter. The runtime replies only after
    /// capture/decode drain, active-segment finalize, and the DB actor fence.
    pub fn shutdown_blocking(&self) -> Result<(), String> {
        let (reply, receive) = oneshot::channel();
        self.sender
            .blocking_send(RuntimeCommand::Shutdown { reply })
            .map_err(|_| "live runtime is unavailable".to_string())?;

        tauri::async_runtime::block_on(async move {
            tokio::time::timeout(SHUTDOWN_TIMEOUT, receive)
                .await
                .map_err(|_| "timed out stopping live runtime".to_string())?
                .map_err(|_| "live runtime stopped without replying".to_string())?
        })
    }

    async fn send(&self, command: RuntimeCommand) -> Result<(), String> {
        self.sender
            .send(command)
            .await
            .map_err(|_| "live runtime is unavailable".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn topic_uses_a_request_reply_without_shared_state() {
        let (handle, mut commands) = LiveRuntimeHandle::new();
        let task = tokio::spawn(async move {
            let RuntimeCommand::GetTopic { topic, reply } = commands.recv().await.unwrap() else {
                panic!("unexpected command")
            };
            assert!(matches!(topic, BootstrapTopic::Combat));
            reply
                .send(TopicBootstrap::Combat(LiveCombatPayload::default()))
                .unwrap();
        });

        assert_eq!(handle.combat().await.unwrap().revision, 0);
        task.await.unwrap();
    }
}
