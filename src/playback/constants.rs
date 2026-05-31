pub const SAMPLE_BUFFER_CAPACITY: usize = 19200;

/*
 * Ratio between chunk size and sub chunk size determines latency and quality, a higher ratio increases
 * quality and also latency, a lower ratio reduces quality but increases latency, recommended ratio is
 * 100 to 1000.
 *
 * Currently we are optimizing for highest quality.
 */
pub static RESAMPLER_CHUNK_SIZE: usize = 2048;
pub static RESAMPLER_SUB_CHUNK_SIZE: usize = 2;
