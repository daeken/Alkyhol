# PE32 DirectSound playback

The handwritten Rust DirectSound bridge routes original PE32 COM calls to a
software PCM mixer and the system default CoreAudio output through public
AudioQueue APIs. It does not replace FMOD or decode game audio itself. Original
game inputs remain local and unchanged. No proprietary translation component is
required, and no microphone/capture path is enabled.

## Implemented contract

`dsound32.rs` owns checked low-address COM vtables, shared IUnknown/refcounts,
parent buffer/device lifetime, default playback enumeration, dynamic DLL
references and static imports. Device caps explicitly describe a software
adapter, without hardware/3D certification. Cooperative normal/priority levels
accept actual owned windows. PCM8/16 mono/stereo at 8–96 kHz, whole/wrapped
streaming locks, partial unlock writes, play/stop/position, frequency, volume,
pan and retained event notifications are implemented. Primary-buffer format
selection requires priority cooperative level. Hardware, 3D, effects and capture
requests return explicit failures. Unsupported exports still fail explicitly.

`sound32.rs` mixes independent voices, linearly resamples, applies hundredth-dB
volume/pan, sums/clips stereo float output and tracks actual queued consumption.
Play cursors and offset/stop events follow consumed frames, not an elapsed-time
simulation. Four 512-frame buffers feed 48 kHz AudioQueue output. Queue creation,
start and enqueue failures propagate; shutdown synchronously disposes callbacks
before their Rust state. The finite queued latency is approximately 43 ms.
Exact native resampler/output waveform parity, focus muting, device-loss recovery,
spatial audio and arbitrary hardware/device configurations are not established.

The ABI follows Microsoft's documented DirectSound interfaces, including
[Lock](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/mt708932(v=vs.85)).
No third-party DirectSound implementation is incorporated.

## Evidence on physical M4, 2026-11-10

Four Rust units pass for format validation, sample decoding, resampling,
consumed versus produced positions, volume/pan, exact mute, multi-voice clipping,
looping, stop and retained event notifications. An explicit opt-in physical
AudioQueue test consumes at least 2048 frames and produces nonzero original test
PCM, then tears down. This is queued-output evidence, not a listener report.

The owned `directsound.c` PE and observer pass 63 fixed words (version 2) at normal and
rebased placements through x86_64 Rosetta/CoreAudio. They cover default-device
callback cancellation, default alias plus real adapter GUID, DLL lookup, COM/notification identity and references, caps/format/output
guards, wrapped/whole-buffer writes, invalid unlock rejection, playback cursor,
looping status, actual notification wait, stop/frozen position and stack cleanup.
The observer can also run the identical PE on native Windows with an actual
audio endpoint; that comparison has not run yet. A CI definition alone is not
a native audio result. Reproduce with:

```
python3 tools/observe_pe32_directsound.py --build
python3 tools/observe_pe32_directsound.py --runner /absolute/path/to/startup-macos
# On actual Windows, with the identical built PE and inputs.json:
python tools/observe_pe32_directsound.py --native --output /path/to/witness
```

The original game's FMOD initialization and audible playback remain unverified
at this component milestone. Software voice support is deliberately bounded;
this is not general DirectSound or game compatibility.

The first original replay calls DirectSoundEnumerateA but does not create a
voice. It skips the NULL default alias. Enumeration now also reports the same
real-backed software adapter by its own GUID when the callback continues, as
[Microsoft documents](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/ee417545(v=vs.85)).
The version-2 witness checks both cancellation and continued enumeration and
creates playback using the returned non-NULL GUID. Its PE SHA-256 is
e34dbbfa92ccf6cdc415ba899a25bf3ac127fc742f070520d1370dad4826b606.
This fixes the identified selection omission; original FMOD playback is still
pending replay.

### Primary listener state

The original FMOD DLL requests PRIMARYBUFFER|CTRL3D (0x11), confirmed from its
original instructions at the observed CreateSoundBuffer call. The primary buffer
now exposes a real IDirectSound3DListener interface sharing its canonical COM
identity/lifetime. All parameter getters, finite position/velocity/orientation,
validated distance/rolloff/doppler setters and deferred versus immediate updates
are backed by retained parameter state; CommitDeferredSettings applies that
state. Secondary spatial buffers remain rejected, and no spatialized output is
claimed. These listener parameters therefore do not alter supported 2D PCM.

The version-3 owned PE adds primary CTRL3D creation, listener QI, guarded defaults,
deferred position/commit, invalid-factor rejection and retained release. Its
71 fixed words pass normal/rebased on physical Rosetta/CoreAudio. A separate
unit checks immediate/deferred field coexistence and atomic rejection. Original
FMOD streaming after this change is still pending replay.

### Original FMOD streaming observed

Clean 7943c38's original Rosetta replay creates the primary listener and an
original FMOD PCM16 stereo streaming buffer (98,304 bytes), sets its playback
frequency to 48 kHz and calls Play(looping). Original Bink also creates real
115,712-byte PCM16 stereo output buffers. Original listener SetAllParameters and
CommitDeferredSettings execute. Physical AudioQueue reports millions of consumed
frames with increasing nonzero PCM counts while the original menu renders.
This is physical original-binary output evidence; listener audibility/quality
confirmation and exact native Windows waveform parity remain separate.
