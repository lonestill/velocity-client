//! Voice connection via Songbird driver.
//! Receives VoiceMessage from gateway and connects/disconnects the driver.

use std::num::NonZeroU64;

use crate::gateway::{VoiceConnectionInfo, VoiceMessage};
use crate::voice_audio;
use dioxus::prelude::spawn;
use songbird::{driver::{DecodeMode, Driver}, CoreEvent, id::*, Config, ConnectionInfo};
use tokio::sync::mpsc;

fn parse_id(s: &str) -> u64 {
    s.parse::<u64>().unwrap_or(0)
}

/// Discord IDs in songbird use NonZeroU64. Use 1 as sentinel for missing guild (DM).
fn connection_info_from_ours(info: &VoiceConnectionInfo) -> Option<ConnectionInfo> {
    let guild_id = info
        .guild_id
        .as_ref()
        .and_then(|s| NonZeroU64::new(parse_id(s)))
        .unwrap_or_else(|| NonZeroU64::new(1).unwrap());
    let channel_id = NonZeroU64::new(parse_id(&info.channel_id))?;
    let user_id = NonZeroU64::new(parse_id(&info.user_id))?;
    Some(ConnectionInfo {
        guild_id: GuildId::from(guild_id),
        channel_id: Some(ChannelId::from(channel_id)),
        endpoint: info.endpoint.clone(),
        token: info.token.clone(),
        session_id: info.session_id.clone(),
        user_id: UserId::from(user_id),
    })
}

/// Spawn the voice task: receives Connect(info) or Leave and runs the Songbird driver.
pub fn spawn_voice_task(mut rx: mpsc::UnboundedReceiver<VoiceMessage>) {
    spawn(async move {
        eprintln!("[voice] task started, waiting for Connect/Leave");
        let mut driver: Option<Driver> = None;
        let mut mic_stream: Option<cpal::Stream> = None;
        let mut speaker_stream: Option<cpal::Stream> = None;
        while let Some(msg) = rx.recv().await {
            eprintln!("[voice] received {}", if matches!(&msg, VoiceMessage::Connect { .. }) { "Connect" } else { "Leave" });
            match msg {
                VoiceMessage::Connect { info, input_device, output_device } => {
                    eprintln!("[voice] connecting to channel {} (input_device={:?}, output_device={:?})", info.channel_id, input_device, output_device);
                    let conn = match connection_info_from_ours(&info) {
                        Some(c) => c,
                        None => {
                            eprintln!("[voice] failed to build connection info");
                            continue;
                        }
                    };
                    // Create speaker and register VoiceTick handler *before* connect so ticks are handled from first packet.
                    let (speaker_stream_opt, queue_opt) = match voice_audio::create_speaker_output(output_device.as_deref()) {
                        Some((stream, queue)) => {
                            eprintln!("[voice] speaker output created");
                            (Some(stream), Some(queue))
                        }
                        None => {
                            eprintln!("[voice] no output device available");
                            (None, None)
                        }
                    };
                    let mut d = Driver::new(Config::default().decode_mode(DecodeMode::Decode));
                    if let Some(ref queue) = queue_opt {
                        let handler = voice_audio::VoicePlayback::new(queue.clone());
                        d.add_global_event(CoreEvent::VoiceTick.into(), handler);
                        eprintln!("[voice] VoiceTick handler registered");
                    }
                    if let Err(e) = d.connect(conn).await {
                        eprintln!("[voice] connect error: {}", e);
                        continue;
                    }
                    eprintln!("[voice] driver connected");
                    if let Some(stream) = speaker_stream_opt {
                        speaker_stream = Some(stream);
                    }
                    // Start microphone capture and send it as a live raw PCM source.
                    if let Some((stream, input)) = voice_audio::create_mic_input(input_device.as_deref()) {
                        d.play_input(input);
                        mic_stream = Some(stream);
                        eprintln!("[voice] microphone input started");
                    } else {
                        eprintln!("[voice] no microphone device available");
                    }
                    driver = Some(d);
                }
                VoiceMessage::Leave => {
                    if let Some(mut d) = driver.take() {
                        d.leave();
                    }
                    mic_stream = None;
                    speaker_stream = None;
                }
            }
        }
    });
}
