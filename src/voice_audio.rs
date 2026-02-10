//! Microphone capture for Songbird driver (voice feature).

use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{traits::*, HeapRb};
use songbird::input::RawAdapter;
use std::io::{Read, Seek, SeekFrom};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use songbird::{Event, EventContext, EventHandler};

const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 1;

/// Reader that yields f32 PCM as little-endian bytes from a ring buffer.
/// Implements Read + Seek (seek unsupported) for Songbird RawAdapter.
struct MicReader<C> {
    consumer: Mutex<C>,
    pending: [u8; 4],
    pending_len: usize,
}

impl<C> Read for MicReader<C>
where
    C: Consumer<Item = f32>,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut written = 0;
        if self.pending_len > 0 {
            let take = self.pending_len.min(buf.len());
            buf[..take].copy_from_slice(&self.pending[..take]);
            self.pending.copy_within(take..self.pending_len, 0);
            self.pending_len -= take;
            written += take;
            if written >= buf.len() {
                return Ok(written);
            }
        }
        let buf = &mut buf[written..];
        let mut guard = match self.consumer.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        while buf.len() >= 4 {
            let sample = match guard.try_pop() {
                Some(s) => s,
                None => {
                    // Don't block: return what we have (or 0) so driver isn't stuck
                    break;
                }
            };
            let bytes = sample.to_le_bytes();
            let n = 4.min(buf.len());
            buf[..n].copy_from_slice(&bytes[..n]);
            written += n;
            if n < 4 {
                self.pending[..4 - n].copy_from_slice(&bytes[n..]);
                self.pending_len = 4 - n;
                break;
            }
        }
        Ok(written)
    }
}

impl<C> Seek for MicReader<C>
where
    C: Consumer<Item = f32>,
{
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "mic stream is not seekable",
        ))
    }
}

impl<C> symphonia_core::io::MediaSource for MicReader<C>
where
    C: Consumer<Item = f32> + Send,
{
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}

/// List names of available input (microphone) devices. Empty string = default.
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    let mut names = vec!["(Default)".to_string()];
    if let Ok(devices) = host.input_devices() {
        for dev in devices {
            if let Ok(name) = dev.name() {
                names.push(name);
            }
        }
    }
    names
}

/// List names of available output (speaker) devices. Empty string = default.
pub fn list_output_devices() -> Vec<String> {
    let host = cpal::default_host();
    let mut names = vec!["(Default)".to_string()];
    if let Ok(devices) = host.output_devices() {
        for dev in devices {
            if let Ok(name) = dev.name() {
                names.push(name);
            }
        }
    }
    names
}

/// Create mic capture and Songbird Input. Returns (stream_handle, Input).
/// device_name: None or "(Default)" = default device; otherwise match by name.
pub fn create_mic_input(device_name: Option<&str>) -> Option<(cpal::Stream, songbird::input::Input)> {
    let rb = HeapRb::<f32>::new(SAMPLE_RATE as usize * 2);
    let (mut producer, consumer) = rb.split();
    let reader = MicReader {
        consumer: Mutex::new(consumer),
        pending: [0u8; 4],
        pending_len: 0,
    };
    let host = cpal::default_host();
    let device = match device_name {
        None | Some("") | Some("(Default)") => host.default_input_device()?,
        Some(name) => {
            host.input_devices()
                .ok()?
                .find(|d| d.name().map(|n| n.as_str() == name).unwrap_or(false))?
        }
    };
    let dev_name = device.name().unwrap_or_else(|_| "?".into());
    eprintln!("[voice] mic device: {}", dev_name);
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for &s in data {
                    let _ = producer.try_push(s);
                }
            },
            move |err| {
                eprintln!("[voice] mic stream error: {}", err);
            },
            None,
        )
        .ok()?;
    stream.play().ok()?;
    let input: songbird::input::Input = RawAdapter::new(reader, SAMPLE_RATE, CHANNELS).into();
    Some((stream, input))
}

/// Create a speaker output stream (48kHz stereo i16) and a shared queue for samples.
/// device_name: None or "(Default)" = default device; otherwise match by name.
pub fn create_speaker_output(device_name: Option<&str>) -> Option<(cpal::Stream, Arc<Mutex<VecDeque<i16>>>)> {
    let host = cpal::default_host();
    let device = match device_name {
        None | Some("") | Some("(Default)") => host.default_output_device()?,
        Some(name) => {
            host.output_devices()
                .ok()?
                .find(|d| d.name().map(|n| n.as_str() == name).unwrap_or(false))?
        }
    };
    let dev_name = device.name().unwrap_or_else(|_| "?".into());
    eprintln!("[voice] speaker device: {}", dev_name);
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };
    let queue: Arc<Mutex<VecDeque<i16>>> = Arc::new(Mutex::new(VecDeque::new()));
    let q = queue.clone();
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut guard = match q.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                for s in data.iter_mut() {
                    *s = guard.pop_front().unwrap_or(0);
                }
                // Prevent unbounded growth if producer outruns consumer.
                const MAX_SAMPLES: usize = (SAMPLE_RATE as usize) * 2 * 2; // ~2s stereo
                if guard.len() > MAX_SAMPLES {
                    let drain = guard.len() - MAX_SAMPLES;
                    for _ in 0..drain {
                        let _ = guard.pop_front();
                    }
                }
            },
            move |err| eprintln!("[voice] speaker stream error: {}", err),
            None,
        )
        .ok()?;
    stream.play().ok()?;
    Some((stream, queue))
}

/// Songbird global event handler: mixes decoded voice and pushes to speaker queue.
#[derive(Clone)]
pub struct VoicePlayback {
    queue: Arc<Mutex<VecDeque<i16>>>,
}

impl VoicePlayback {
    pub fn new(queue: Arc<Mutex<VecDeque<i16>>>) -> Self {
        Self { queue }
    }
}

#[async_trait]
impl EventHandler for VoicePlayback {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::VoiceTick(tick) => {
                let n_speakers = tick.speaking.len();
                // Mix all speakers into one PCM buffer (i16 stereo interleaved).
                let mut mix: Option<Vec<i32>> = None;
                for (_ssrc, data) in &tick.speaking {
                    let Some(pcm) = data.decoded_voice.as_ref() else { continue };
                    let m = mix.get_or_insert_with(|| vec![0i32; pcm.len()]);
                    if m.len() != pcm.len() {
                        // Length mismatch: resize mix to max and mix what we can (e.g. mono vs stereo).
                        if pcm.len() > m.len() {
                            m.resize(pcm.len(), 0);
                        }
                        let len = m.len().min(pcm.len());
                        for (i, &s) in pcm[..len].iter().enumerate() {
                            m[i] += s as i32;
                        }
                    } else {
                        for (i, &s) in pcm.iter().enumerate() {
                            m[i] += s as i32;
                        }
                    }
                }
                if let Some(m) = mix {
                    if n_speakers > 0 && m.len() > 0 {
                        // Log occasionally so we don't flood (e.g. every ~50 ticks ≈ 1s)
                        static TICK_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                        let c = TICK_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if c % 50 == 0 {
                            eprintln!("[voice] VoiceTick: {} speaker(s), {} samples", n_speakers, m.len());
                        }
                    }
                    let mut out: Vec<i16> = Vec::with_capacity(m.len());
                    for v in m {
                        let v = v.clamp(i16::MIN as i32, i16::MAX as i32);
                        out.push(v as i16);
                    }
                    if let Ok(mut guard) = self.queue.lock() {
                        guard.extend(out);
                    }
                }
            }
            _ => {}
        }
        None
    }
}
