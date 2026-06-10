# Playback Controller

The Playback Controller is the interface used to interact with the audio decoding thread (Audio Pipeline) and the audio output stream (Audio Engine). 

## Audio Engine

The audio engine is meant to handle everything related to the output stream built with `cpal`, it allows to pause and resume the stream. The output stream thread receives `f32` PCM samples through a ring buffer built with `rtrb`. 

**Important:** The output callback must not block, allocate, decode or perform any sort of database queries or I/O, it must be as performant as possible since it is a critical flow to output audio to the system. If there is an underrun of samples from the ring buffer the output stream receives silence.

## Audio Pipeline

The audio pipeline is where audio decoding happens, it takes a track, extracts the samples through audio decoding, resamples them and/or converts the channels of the samples if needed, and inserts them in an audio sink that inserts the samples in the ring buffer with an external buffer in case it gets filled up.

### Architecture

The audio pipeline orchestrates a series of stages, a stage represents any individual process in the pipeline that manipulates samples and outputs a new set of samples.

Stages get grouped in two sets:

- Track stages, which live within a Audio Track Pipeline.
- Output stages, which live in the Audio Pipeline and do not pertain to any particular track.

And stages can be of these types:

- Source: Produces samples, does not receive samples (Decoder).
- Process: Receives samples and outputs them modifying them (Resampler, Channel Converter)
- Join: Takes multiple samples and mixes them together in one set of samples (Mixer).

Stages get processed in sequential order and only if they are enabled and receive a reference to the current configuration of the pipeline, first we run all the track stages (if the Audio Track Pipelines should produce samples), and then those samples go through the output stages, mixed if there is more than one Audio Track Pipeline active.

Commands are received through a `mpsc` channel and trickle down from the Audio Pipeline itself through all of the stages, as of now only the main Audio Track Pipeline receives commands (the Audio Track Pipeline that is first in the vector of pipelines), this allows stage internal implementations to react to commands without the need of having their implementation in the orchestrating Audio Pipeline.

Events get emitted through a `mpsc` channel as well at the stage layer through the emitter set in the configuration of the Audio Pipeline, internal implementation of the stages must implement their own way of buffering pending events so that the stage can consume and emit them.

Once the we reach the end of all stages, the samples get introduced in the Audio Sink buffer, the next iteration then attempts to insert as many samples to the ring buffer as possible, if the ring buffer is full, it **sleeps half of the time needed for the output stream to clear the ring buffer and tries again until the buffer empties.**