pub trait FrameSource: Send + Sync{
    /// Get the next frame from this source. The provided frame must be stored
    /// in the provided buffer in 32-bit RGBA pixel format.
    fn next_frame(&self, width: u32, height: u32, timestamp: i64, data: &mut [u8]);
}
