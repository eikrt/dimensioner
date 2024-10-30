use crate::worldgen::{Chunk, News};

#[derive(Clone)]
pub struct RenderMsg {
    pub chunk: Chunk,
    pub news: News,
}
impl RenderMsg {
    pub fn from(chunk: Chunk, news: News) -> RenderMsg {
        RenderMsg {
            chunk: chunk,
            news: news,
        }
    }
}
